use crate::models::{FeatureFlag, RawOptionalFeatureFlag, RawOptionalFeatureFlags};
use std::collections::HashSet;

pub type DB = ();
pub type DBConnection = Connection;
pub type SetOutput = Result<FeatureFlag, ()>;
pub type GetOutput = Result<FeatureFlag, ()>;
pub type BackendError = Error;

const NAMESPACE: &str = "fun_with_flags";

use redis::Commands;

///
/// redis contains a fun_with_flags set field with all keys that are used
///
/// each field is namespaced with fun_with_flags
/// each field is of type hash (map)
///
///
pub struct Backend {}

pub struct Connection {
    pub pool: r2d2::Pool<redis::Client>,
}

#[derive(Debug)]
pub struct Error(String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<r2d2::Error> for Error {
    fn from(e: r2d2::Error) -> Self {
        Error(e.to_string())
    }
}

impl From<redis::RedisError> for Error {
    fn from(e: redis::RedisError) -> Self {
        Error(e.to_string())
    }
}

impl Connection {
    pub fn establish(url: &str) -> Result<Connection, Error> {
        let manager = redis::Client::open(url)?;
        let pool = r2d2::Pool::builder().max_size(15).build(manager)?;
        Ok(Connection { pool })
    }
}

impl Backend {
    pub fn set(pool: &DBConnection, flag: FeatureFlag) -> SetOutput {
        let mut conn = Self::create_conn(pool).unwrap();

        let (k, v) = flag.to_redis_value();
        let key = flag_key(&flag);

        let _: () = redis::pipe()
            .atomic()
            .sadd(NAMESPACE, flag.name())
            .ignore()
            .hset(&key, k, v)
            .ignore()
            .query(&mut *conn)
            .expect("handle error");

        let flag = Self::get(pool, flag)?;

        Ok(flag)
    }

    pub fn get(pool: &DBConnection, flag: FeatureFlag) -> GetOutput {
        let mut conn = Self::create_conn(pool).unwrap();

        let mut map: RawOptionalFeatureFlags = conn.hgetall(flag_key(&flag)).expect("handle error");

        Self::post_processing(&flag, &mut map);

        match map.find(&flag) {
            Some(x) => Ok(x),
            None => Err(()),
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

    pub fn create_conn(
        pool: &DBConnection,
    ) -> Result<r2d2::PooledConnection<redis::Client>, r2d2::Error> {
        let pool = pool.pool.clone();
        pool.get()
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
            ))), // _ => invalid_type_error!(v, "Response type not hashmap compatible"),
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
        };
        x
    }
}
