use crate::models::FeatureFlag;
use crate::schema::fun_with_flags_toggles;
use diesel::prelude::*;
use diesel::sql_types::Bool;

pub type DB = diesel::pg::Pg;
pub type DBConnection = PgConnection;
pub type SetOutput = Result<Vec<FeatureFlag>, diesel::result::Error>;
pub type GetOutput = Result<FeatureFlag, diesel::result::Error>;

pub struct Backend {}

impl Backend {
    pub fn set(
        conn: &DBConnection,
        flag: FeatureFlag,
    ) -> Result<Vec<FeatureFlag>, diesel::result::Error> {
        use crate::schema::fun_with_flags_toggles::dsl::*;

        match flag {
            FeatureFlag::Percentage { .. } | FeatureFlag::Time { .. } => {
                let insertable = flag.to_insertable();

                let filter = fun_with_flags_toggles.filter(
                    flag_name
                        .eq(insertable.flag_name)
                        .and(gate_type.eq(insertable.gate_type)),
                );

                let fetch_result = if *insertable.enabled {
                    diesel::update(filter)
                        .set((target.eq(insertable.target), enabled.eq(insertable.enabled)))
                        .get_results::<FeatureFlag>(conn)
                } else {
                    diesel::update(filter)
                        .set(enabled.eq(insertable.enabled))
                        .get_results::<FeatureFlag>(conn)
                }?;

                if fetch_result.is_empty() {
                    diesel::insert_into(fun_with_flags_toggles)
                        .values(flag.to_insertable())
                        .get_results::<FeatureFlag>(conn)
                } else {
                    Ok(fetch_result)
                }
            }
            _ => diesel::insert_into(fun_with_flags_toggles)
                .values(flag.to_insertable())
                .on_conflict((flag_name, gate_type, target))
                .do_update()
                .set(enabled.eq(flag.enabled()))
                .get_results::<FeatureFlag>(conn),
        }
    }

    pub fn get(
        conn: &DBConnection,
        flag: FeatureFlag,
    ) -> Result<FeatureFlag, diesel::result::Error> {
        use crate::schema::fun_with_flags_toggles::dsl::*;

        fun_with_flags_toggles
            .filter(flag.to_filter())
            .first::<FeatureFlag>(conn)
    }

    pub fn backend_name() -> &'static str {
        "postgres"
    }
}

impl Queryable<fun_with_flags_toggles::SqlType, DB> for FeatureFlag {
    type Row = (i64, String, String, String, bool);

    fn build(row: Self::Row) -> Self {
        match row.2.as_ref() {
            "boolean" => FeatureFlag::Boolean {
                name: row.1,
                enabled: row.4,
            },
            "actor" => FeatureFlag::Actor {
                name: row.1,
                target: row.3,
                enabled: row.4,
            },
            "group" => FeatureFlag::Group {
                name: row.1,
                target: row.3,
                enabled: row.4,
            },
            "percentage" if row.3.starts_with("time/") => FeatureFlag::Time {
                name: row.1,
                target: parse_float(row.3, "time/"),
                enabled: row.4,
            },
            "percentage" if row.3.starts_with("actors/") => FeatureFlag::Percentage {
                name: row.1,
                target: parse_float(row.3, "actors/"),
                enabled: row.4,
            },
            _ => panic!("this gate is not supported"),
        }
    }
}

impl FeatureFlag {
    pub fn to_insertable<'a>(&'a self) -> NewFeatureFlag<'a> {
        use FeatureFlag::*;

        match self {
            Boolean { name, enabled } => NewFeatureFlag {
                flag_name: name,
                gate_type: "boolean",
                target: "_fwf_none".to_string(),
                enabled: enabled,
            },
            Actor {
                name,
                target,
                enabled,
            } => NewFeatureFlag {
                flag_name: name,
                gate_type: "actor",
                target: target.to_string(),
                enabled: enabled,
            },
            Group {
                name,
                target,
                enabled,
            } => NewFeatureFlag {
                flag_name: name,
                gate_type: "group",
                target: target.to_string(),
                enabled: enabled,
            },
            Time {
                name,
                target,
                enabled,
            } => NewFeatureFlag {
                flag_name: name,
                gate_type: "percentage",
                target: format!("time/{}", target),
                enabled: enabled,
            },
            Percentage {
                name,
                target,
                enabled,
            } => NewFeatureFlag {
                flag_name: name,
                gate_type: "percentage",
                target: format!("actors/{}", target),
                enabled: enabled,
            },
        }
    }

    // pub fn to_filter<'a>(&'a self) -> (&'a str, &'a str, String) {
    pub fn to_filter<'a>(
        &'a self,
    ) -> Box<dyn BoxableExpression<fun_with_flags_toggles::table, DB, SqlType = Bool> + 'a> {
        use crate::schema::fun_with_flags_toggles::dsl::*;
        use FeatureFlag::*;

        let insertable = self.to_insertable();
        match self {
            Time { .. } | Percentage { .. } => Box::new(
                flag_name
                    .eq(insertable.flag_name)
                    .and(gate_type.eq(insertable.gate_type)),
            ),

            _ => Box::new(
                flag_name
                    .eq(insertable.flag_name)
                    .and(gate_type.eq(insertable.gate_type))
                    .and(target.eq(insertable.target)),
            ),
        }
    }
}

#[derive(Insertable, AsChangeset)]
#[table_name = "fun_with_flags_toggles"]
pub struct NewFeatureFlag<'a> {
    pub flag_name: &'a str,
    pub gate_type: &'a str,
    pub target: String,
    pub enabled: &'a bool,
}

fn parse_float(mut value: String, prefix: &'static str) -> f64 {
    value
        .split_off(prefix.len())
        .parse()
        .expect("db contains invalid data")
}
