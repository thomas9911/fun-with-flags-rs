use crate::models::{FeatureFlag, GroupSet, RawFeatureFlag};

use postgres::types::ToSql;
use postgres::NoTls;
use postgres::Row;

use r2d2_postgres::PostgresConnectionManager;

pub type DB = ();
pub type DBConnection = Connection;
pub type SetOutput = Result<FeatureFlag, Error>;
pub type GetOutput = Result<FeatureFlag, Error>;

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

impl From<postgres::Error> for Error {
    fn from(e: postgres::Error) -> Self {
        Error(e.to_string())
    }
}

pub struct Connection {
    pub pool: r2d2::Pool<PostgresConnectionManager<NoTls>>,
}

impl Connection {
    pub fn establish(url: &str) -> Result<Connection, ()> {
        let manager = PostgresConnectionManager::new(url.parse().unwrap(), NoTls);
        let pool = r2d2::Pool::new(manager).unwrap();
        Ok(Connection { pool })
    }
}

pub struct Backend {}

impl Backend {
    pub fn set(conn: &DBConnection, flag: FeatureFlag) -> SetOutput {
        let mut conn = Self::create_conn(conn)?;

        use FeatureFlag::*;

        let insertable = flag.to_row();
        let db_result = match flag {
            Percentage { .. } | Time { .. } => {
                let result = if insertable.enabled {
                    let update_enable = r#"UPDATE "fun_with_flags_toggles" 
                SET "target" = $1, "enabled" = $2 
                WHERE "fun_with_flags_toggles"."flag_name" = $3 AND "fun_with_flags_toggles"."gate_type" = $4 
                RETURNING "fun_with_flags_toggles"."id", "fun_with_flags_toggles"."flag_name", "fun_with_flags_toggles"."gate_type", "fun_with_flags_toggles"."target", "fun_with_flags_toggles"."enabled""#;
                    let arg_enable1: Vec<&(dyn ToSql + Sync)> = vec![
                        &insertable.target,
                        &insertable.enabled,
                        &insertable.flag_name,
                        &insertable.gate_type,
                    ];
                    conn.query_opt(update_enable, &arg_enable1)?
                } else {
                    let update_disable = r#"UPDATE "fun_with_flags_toggles" 
                SET "enabled" = $1 
                WHERE "fun_with_flags_toggles"."flag_name" = $2 AND "fun_with_flags_toggles"."gate_type" = $3 
                RETURNING "fun_with_flags_toggles"."id", "fun_with_flags_toggles"."flag_name", "fun_with_flags_toggles"."gate_type", "fun_with_flags_toggles"."target", "fun_with_flags_toggles"."enabled""#;
                    let arg_disable1: Vec<&(dyn ToSql + Sync)> = vec![
                        &insertable.enabled,
                        &insertable.flag_name,
                        &insertable.gate_type,
                    ];
                    conn.query_opt(update_disable, &arg_disable1)?
                };

                if result.is_none() {
                    let insert = r#"INSERT INTO "fun_with_flags_toggles" ("flag_name", "gate_type", "target", "enabled") 
                VALUES ($1, $2, $3, $4) 
                RETURNING "fun_with_flags_toggles"."id", "fun_with_flags_toggles"."flag_name", "fun_with_flags_toggles"."gate_type", "fun_with_flags_toggles"."target", "fun_with_flags_toggles"."enabled""#;
                    let arg2: Vec<&(dyn ToSql + Sync)> = vec![
                        &insertable.flag_name,
                        &insertable.gate_type,
                        &insertable.target,
                        &insertable.enabled,
                    ];
                    conn.query_one(insert, &arg2)?
                } else {
                    result.unwrap()
                }
            }
            _ => {
                let insert = r#"INSERT INTO "fun_with_flags_toggles" ("flag_name", "gate_type", "target", "enabled") 
            VALUES ($1, $2, $3, $4) ON CONFLICT ("flag_name", "gate_type", "target") 
            DO UPDATE SET "enabled" = $5 
            RETURNING "fun_with_flags_toggles"."id", "fun_with_flags_toggles"."flag_name", "fun_with_flags_toggles"."gate_type", "fun_with_flags_toggles"."target", "fun_with_flags_toggles"."enabled""#;
                let args: Vec<&(dyn ToSql + Sync)> = vec![
                    &insertable.flag_name,
                    &insertable.gate_type,
                    &insertable.target,
                    &insertable.enabled,
                    &insertable.enabled,
                ];
                conn.query_one(insert, &args)?
            }
        };

        Ok(FeatureFlag::from_row(db_result))
    }

    pub fn get(conn: &DBConnection, flag: FeatureFlag) -> GetOutput {
        let mut conn = Self::create_conn(conn).unwrap();

        use FeatureFlag::*;

        let insertable = flag.to_row();
        let db_result = match flag {
            Time { .. } | Percentage { .. } | Group { .. } => {
                let query = r#"SELECT "fun_with_flags_toggles"."id", "fun_with_flags_toggles"."flag_name", "fun_with_flags_toggles"."gate_type", "fun_with_flags_toggles"."target", "fun_with_flags_toggles"."enabled" 
                FROM "fun_with_flags_toggles" WHERE "fun_with_flags_toggles"."flag_name" = $1 AND "fun_with_flags_toggles"."gate_type" = $2"#;
                let arguments: Vec<&(dyn ToSql + Sync)> =
                    vec![&insertable.flag_name, &insertable.gate_type];
                conn.query_one(query, &arguments)?
            }

            _ => {
                let query = r#"SELECT "fun_with_flags_toggles"."id", "fun_with_flags_toggles"."flag_name", "fun_with_flags_toggles"."gate_type", "fun_with_flags_toggles"."target", "fun_with_flags_toggles"."enabled" 
                FROM "fun_with_flags_toggles" WHERE "fun_with_flags_toggles"."flag_name" = $1 AND "fun_with_flags_toggles"."gate_type" = $2 AND "fun_with_flags_toggles"."target" = $3"#;
                let arguments: Vec<&(dyn ToSql + Sync)> = vec![
                    &insertable.flag_name,
                    &insertable.gate_type,
                    &insertable.target,
                ];
                conn.query_one(query, &arguments)?
            }
        };

        Ok(FeatureFlag::from_row(db_result))
    }

    pub fn backend_name() -> &'static str {
        "postgres"
    }

    pub fn create_conn(
        pool: &DBConnection,
    ) -> Result<r2d2::PooledConnection<PostgresConnectionManager<NoTls>>, r2d2::Error> {
        let pool = pool.pool.clone();
        pool.get()
    }
}

impl FeatureFlag {
    pub fn to_row(&self) -> RawFeatureFlag {
        use FeatureFlag::*;

        match self {
            Boolean { name, enabled } => RawFeatureFlag {
                flag_name: name.to_string(),
                gate_type: "boolean".to_string(),
                target: "_fwf_none".to_string(),
                enabled: *enabled,
            },
            Actor {
                name,
                target,
                enabled,
            } => RawFeatureFlag {
                flag_name: name.to_string(),
                gate_type: "actor".to_string(),
                target: target.to_string(),
                enabled: *enabled,
            },
            Group {
                name,
                target,
                enabled,
            } => RawFeatureFlag {
                flag_name: name.to_string(),
                gate_type: "group".to_string(),
                target: target.to_optional_string().unwrap_or("").to_string(),
                enabled: *enabled,
            },
            Time {
                name,
                target,
                enabled,
            } => RawFeatureFlag {
                flag_name: name.to_string(),
                gate_type: "percentage".to_string(),
                target: format!("time/{}", target),
                enabled: *enabled,
            },
            Percentage {
                name,
                target,
                enabled,
            } => RawFeatureFlag {
                flag_name: name.to_string(),
                gate_type: "percentage".to_string(),
                target: format!("actors/{}", target),
                enabled: *enabled,
            },
        }
    }

    pub fn from_row(row: Row) -> FeatureFlag {
        let flag_name: String = row.get("flag_name");
        let gate_type: String = row.get("gate_type");
        let target: String = row.get("target");
        let enabled: bool = row.get("enabled");

        match gate_type.as_ref() {
            "boolean" => FeatureFlag::Boolean {
                name: flag_name,
                enabled,
            },
            "actor" => FeatureFlag::Actor {
                name: flag_name,
                target,
                enabled,
            },
            "group" => FeatureFlag::Group {
                name: flag_name,
                target: GroupSet::new(target),
                enabled,
            },
            "percentage" if target.starts_with("time/") => FeatureFlag::Time {
                name: flag_name,
                target: parse_float(target, "time/"),
                enabled,
            },
            "percentage" if target.starts_with("actors/") => FeatureFlag::Percentage {
                name: flag_name,
                target: parse_float(target, "actors/"),
                enabled,
            },
            _ => panic!("this gate is not supported"),
        }
    }
}

fn parse_float(mut value: String, prefix: &'static str) -> f64 {
    value
        .split_off(prefix.len())
        .parse()
        .expect("db contains invalid data")
}
