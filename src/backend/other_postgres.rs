use crate::models::{FeatureFlag, GroupSet, RawFeatureFlag, RawOptionalFeatureFlag};

use postgres::types::ToSql;
use postgres::Row;
use postgres::{Client, NoTls};

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
        // use crate::schema::fun_with_flags_toggles::dsl::*;

        // match flag {
        //     FeatureFlag::Percentage { .. } | FeatureFlag::Time { .. } => {
        //         let insertable = flag.to_insertable();

        //         let filter = fun_with_flags_toggles.filter(
        //             flag_name
        //                 .eq(insertable.flag_name)
        //                 .and(gate_type.eq(insertable.gate_type)),
        //         );

        //         let fetch_result = if *insertable.enabled {
        //             diesel::update(filter)
        //                 .set((target.eq(insertable.target), enabled.eq(insertable.enabled)))
        //                 .get_results::<FeatureFlag>(conn)
        //         } else {
        //             diesel::update(filter)
        //                 .set(enabled.eq(insertable.enabled))
        //                 .get_results::<FeatureFlag>(conn)
        //         }?;

        //         if fetch_result.is_empty() {
        //             diesel::insert_into(fun_with_flags_toggles)
        //                 .values(flag.to_insertable())
        //                 .get_results::<FeatureFlag>(conn)
        //         } else {
        //             Ok(fetch_result)
        //         }
        //     }
        //     _ => diesel::insert_into(fun_with_flags_toggles)
        //         .values(flag.to_insertable())
        //         .on_conflict((flag_name, gate_type, target))
        //         .do_update()
        //         .set(enabled.eq(flag.enabled()))
        //         .get_results::<FeatureFlag>(conn),
        // }
        let mut conn = Self::create_conn(conn)?;

        use FeatureFlag::*;

        let insertable = flag.to_row();
        let db_result = match flag {
            Percentage { .. } | Time { .. } => {
                let (update, arg1) = if insertable.enabled {
                    let update_enable = r#"UPDATE "fun_with_flags_toggles" 
                SET "target" = $1, "enabled" = $2 
                WHERE "fun_with_flags_toggles"."flag_name" = $3 AND "fun_with_flags_toggles"."gate_type" = $4 
                RETURNING "fun_with_flags_toggles"."id", "fun_with_flags_toggles"."flag_name", "fun_with_flags_toggles"."gate_type", "fun_with_flags_toggles"."target", "fun_with_flags_toggles"."enabled""#;
                    let arg_enable1: Vec<&(dyn ToSql + Sync)> = vec![
                        &insertable.target,
                        &insertable.enabled.to_string(),
                        &insertable.flag_name,
                        &insertable.gate_type,
                    ];
                    (update_enable, arg_enable1)
                } else {
                    let update_disable = r#"UPDATE "fun_with_flags_toggles" 
                SET "enabled" = $1 
                WHERE "fun_with_flags_toggles"."flag_name" = $2 AND "fun_with_flags_toggles"."gate_type" = $3 
                RETURNING "fun_with_flags_toggles"."id", "fun_with_flags_toggles"."flag_name", "fun_with_flags_toggles"."gate_type", "fun_with_flags_toggles"."target", "fun_with_flags_toggles"."enabled""#;
                    let arg_disable1: Vec<&(dyn ToSql + Sync)> = vec![
                        &insertable.enabled.to_string(),
                        &insertable.flag_name,
                        &insertable.gate_type,
                    ];
                    (update_disable, arg_disable1)
                };

                let result = conn.query_opt(update, &arg1)?;

                if result.is_none() {
                    let insert = r#"INSERT INTO "fun_with_flags_toggles" ("flag_name", "gate_type", "target", "enabled") 
                VALUES ($1, $2, $3, $4) 
                RETURNING "fun_with_flags_toggles"."id", "fun_with_flags_toggles"."flag_name", "fun_with_flags_toggles"."gate_type", "fun_with_flags_toggles"."target", "fun_with_flags_toggles"."enabled""#;
                    let arg2: Vec<&(dyn ToSql + Sync)> = vec![
                        &insertable.flag_name,
                        &insertable.gate_type,
                        &insertable.target,
                        &insertable.enabled.to_string(),
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
                    &insertable.enabled.to_string(),
                    &insertable.enabled.to_string(),
                ];
                conn.query_one(insert, &args)?
            }
        };

        Ok(FeatureFlag::from_row(db_result))
    }

    pub fn get(conn: &DBConnection, flag: FeatureFlag) -> GetOutput {
        // use crate::schema::fun_with_flags_toggles::dsl::*;

        // fun_with_flags_toggles
        //     .filter(flag.to_filter())
        //     .first::<FeatureFlag>(conn)
        let mut conn = Self::create_conn(conn).unwrap();

        // "SELECT id, flag_name, gate_type, target, enabled"

        let (query, arguments) = flag.to_select_sql_and_params();

        // conn.query(
        //     r#"SELECT "fun_with_flags_toggles"."id", "fun_with_flags_toggles"."flag_name", "fun_with_flags_toggles"."gate_type", "fun_with_flags_toggles"."target", "fun_with_flags_toggles"."enabled" FROM "fun_with_flags_toggles" WHERE "fun_with_flags_toggles"."flag_name" = $1 AND "fun_with_flags_toggles"."gate_type" = $2 AND "fun_with_flags_toggles"."target" = $3 LIMIT $4"#,
        //     &[]
        // );

        let db_result = conn.query_one(query, arguments)?;
        Ok(FeatureFlag::from_row(db_result))

        // CREATE TABLE fun_with_flags_toggles (
        //     id BIGSERIAL PRIMARY KEY,
        //     flag_name VARCHAR NOT NULL,
        //     gate_type VARCHAR NOT NULL,
        //     target VARCHAR NOT NULL,
        //     enabled BOOLEAN NOT NULL
        // );
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
    pub fn to_row(&self) -> RawFeatureFlag{
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
        let feature_flag = RawOptionalFeatureFlag{
            flag_name: Some(row.get("flag_name")),
            gate_type: row.get("gate_type"),
            target: row.get("target"),
            enabled: row.get("enabled"),    
        };

        FeatureFlag::from(feature_flag)
    }

    pub fn to_select_sql_and_params(&self) -> (&str, &[&(dyn ToSql + Sync)]) {
        use FeatureFlag::*;

        let insertable = self.to_row();
        match self {
            Time { .. } | Percentage { .. } | Group { .. } => (
                r#"SELECT "fun_with_flags_toggles"."id", "fun_with_flags_toggles"."flag_name", "fun_with_flags_toggles"."gate_type", "fun_with_flags_toggles"."target", "fun_with_flags_toggles"."enabled" 
                FROM "fun_with_flags_toggles" WHERE "fun_with_flags_toggles"."flag_name" = $1 AND "fun_with_flags_toggles"."gate_type" = $2"#,
                &[
                    &insertable.flag_name,
                    &insertable.gate_type,
                ],
            ),

    // Box::new(
    //     flag_name
    //         .eq(insertable.flag_name)
    //         .and(gate_type.eq(insertable.gate_type)),
    // ),
            _ => (
                r#"SELECT "fun_with_flags_toggles"."id", "fun_with_flags_toggles"."flag_name", "fun_with_flags_toggles"."gate_type", "fun_with_flags_toggles"."target", "fun_with_flags_toggles"."enabled" 
                FROM "fun_with_flags_toggles" WHERE "fun_with_flags_toggles"."flag_name" = $1 AND "fun_with_flags_toggles"."gate_type" = $2 AND "fun_with_flags_toggles"."target" = $3"#,
                &[
                    &insertable.flag_name,
                    &insertable.gate_type,
                    &insertable.target,
                ],
            ),
    // Box::new(
    //     flag_name
    //         .eq(insertable.flag_name)
    //         .and(gate_type.eq(insertable.gate_type))
    //         .and(target.eq(insertable.target)),
    // ),
        }
    }

    // pub fn to_insert_sql_and_params(&self) -> (&str, &[&(dyn ToSql + Sync)]) {

    // }
    // }
}

