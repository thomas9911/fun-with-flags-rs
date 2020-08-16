use crate::models::{FeatureFlag, RawOptionalFeatureFlag, RawOptionalFeatureFlags};
use crate::Error;
use std::collections::HashSet;
use std::sync::Mutex;
use state::Storage;
use redis::Commands;

pub type DB = ();
pub type DBConnection = Connection;
pub type SetOutput = Result<FeatureFlag, Error>;
pub type GetOutput = Result<FeatureFlag, Error>;
pub type ConnectionResult = Result<PooledConnection, Error>;
type PooledConnection = r2d2::PooledConnection<redis::Client>;
type Pool = r2d2::Pool<redis::Client>;

const NAMESPACE: &str = "fun_with_flags";

lazy_static::lazy_static! {
    static ref GLOBAL_POOL: Storage<Mutex<Pool>> = Storage::new();
}

///
/// redis contains a fun_with_flags set field with all keys that are used
///
/// each field is namespaced with fun_with_flags
/// each field is of type hash (map)
///
///
pub struct Backend {}

pub struct Connection {
    pub config: String
}

impl Connection {
    pub fn establish(url: &str) -> Result<Connection, Error> {
        if GLOBAL_POOL.try_get().is_some() {
            Ok(Connection{config: url.to_string()})
        } else{ 
            let manager = redis::Client::open(url)?;
            let pool = r2d2::Pool::builder().max_size(15).build(manager)?;
            GLOBAL_POOL.set(Mutex::new(pool));
            Self::establish(url)
        }
    }
}

impl Backend {
    pub fn set(pool: &DBConnection, flag: FeatureFlag) -> SetOutput {
        let mut conn = Self::create_conn(pool)?;

        let (k, v) = flag.to_redis_value();
        let key = flag_key(&flag);

        let _: () = redis::pipe()
            .atomic()
            .sadd(NAMESPACE, flag.name())
            .ignore()
            .hset(&key, k, v)
            .ignore()
            .query(&mut *conn)?;

        let flag =  Self::priv_get(conn, flag)?;

        Ok(flag)
    }

    pub fn get(pool: &DBConnection, flag: FeatureFlag) -> GetOutput {
        let conn = Self::create_conn(pool)?;
        Self::priv_get(conn, flag)
    }

    fn priv_get(mut conn: PooledConnection, flag: FeatureFlag) -> GetOutput {
        let mut map: RawOptionalFeatureFlags = conn.hgetall(flag_key(&flag))?;

        Self::post_processing(&flag, &mut map);

        match map.find(&flag) {
            Some(x) => Ok(x),
            None => Ok(FeatureFlag::Empty),
        }
    }

    pub fn all_flags_names(pool: &DBConnection) -> Result<HashSet<String>, Error> {
        let mut conn = Self::create_conn(pool)?;

        let set = conn.smembers(NAMESPACE)?;
        Ok(set)
    }

    pub fn clean_all(pool: &DBConnection) -> Result<(), Error> {
        let flag_names = Self::all_flags_names(pool)?;

        for flag_name in flag_names {
            Self::clean(pool, &flag_name)?
        }

        Ok(())
    }

    pub fn clean(pool: &DBConnection, flag_name: &str) -> Result<(), Error> {
        let mut conn = Self::create_conn(pool)?;

        let key = flag_key_from_str(flag_name);

        let _: () = redis::pipe()
            .atomic()
            .srem(NAMESPACE, flag_name)
            .ignore()
            .del(&key)
            .ignore()
            .query(&mut *conn)?;

        Ok(())
    }

    fn post_processing(original_flag: &FeatureFlag, output: &mut RawOptionalFeatureFlags) {
        output.set_flag_name(original_flag.name().to_string());
        output.update_flag_name();
    }

    pub fn backend_name() -> &'static str {
        "redis"
    }

    pub fn create_conn(config: &DBConnection) -> ConnectionResult {
        if let Some(pool) = GLOBAL_POOL.try_get() {
            let locked_pool = pool.lock().unwrap();
            let cloned_pool = locked_pool.clone();
            let conn = cloned_pool.get()?;
            Ok(conn)
        } else{ 
            DBConnection::establish(&config.config)?;
            Self::create_conn(config)
        }
    }
}

fn flag_key(flag: &FeatureFlag) -> String {
    flag_key_from_str(flag.name())
}

fn flag_key_from_str(flag_name: &str) -> String {
    format!("{}:{}", NAMESPACE, flag_name)
}

impl redis::FromRedisValue for RawOptionalFeatureFlags {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<RawOptionalFeatureFlags> {
        use redis::{from_redis_value, ErrorKind, RedisError, Value};

        match *v {
            Value::Bulk(ref items) => {
                let mut iter = items.iter();
                let mut feature_flags = RawOptionalFeatureFlags::default();
                while let (Some(k), Some(v)) = (iter.next(), iter.next()) {
                    let key: String = from_redis_value(k)?;
                    let value: String = from_redis_value(v)?;

                    let feature_flag = match key {
                        x if x.starts_with("actor") => {
                            let target = x.rsplit("/").next().unwrap();
                            let enabled: bool = value.parse().unwrap();

                            RawOptionalFeatureFlag {
                                target: target.to_string(),
                                gate_type: "actor".to_string(),
                                enabled,
                                flag_name: None,
                            }
                        }

                        x if x == "percentage" => {
                            let mut fields = value.split("/");
                            let gate_type = fields.next().unwrap().to_string();
                            let target = fields.next().unwrap().to_string();
                            RawOptionalFeatureFlag {
                                target,
                                gate_type,
                                enabled: true,
                                flag_name: None,
                            }
                        }

                        x if x == "boolean" => {
                            let enabled: bool = value.parse().unwrap();
                            RawOptionalFeatureFlag {
                                gate_type: "boolean".to_string(),
                                target: String::new(),
                                enabled,
                                flag_name: None,
                            }
                        }

                        x if x.starts_with("group") => {
                            let target = x.rsplit("/").next().unwrap();
                            let enabled: bool = value.parse().unwrap();

                            RawOptionalFeatureFlag {
                                target: target.to_string(),
                                gate_type: "group".to_string(),
                                enabled,
                                flag_name: None,
                            }
                        }

                        _ => unimplemented!(),
                    };

                    feature_flags.add(feature_flag);
                }
                Ok(feature_flags)
            }
            _ => Err(RedisError::from((
                ErrorKind::TypeError,
                "Response type not hashmap compatible",
            ))),
        }
    }
}

impl FeatureFlag {
    pub fn to_redis_value(&self) -> (String, String) {
        use FeatureFlag::*;

        let x = match self {
            Boolean { enabled, .. } => ("boolean".to_string(), enabled.to_string()),
            Actor {
                target, enabled, ..
            } => (format!("actor/{}", target), enabled.to_string()),
            Group {
                target, enabled, ..
            } => (
                format!("group/{}", target.get_first_unsafe()),
                enabled.to_string(),
            ),
            Time { target, .. } => ("percentage".to_string(), format!("time/{}", target)),
            Percentage { target, .. } => ("percentage".to_string(), format!("actors/{}", target)),
            Empty => panic!("can not set this value"),
        };
        x
    }
}
