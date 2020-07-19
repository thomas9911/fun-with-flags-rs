// use diesel::deserialize::Queryable;
// use diesel::prelude::Queryable;

#[derive(Debug, FromSqlRow)]
pub struct RawFeatureFlag {
    pub id: i64,
    pub flag_name: String,
    pub gate_type: String,
    pub target: String,
    pub enabled: bool,
}

#[derive(Debug)]
pub enum FeatureFlag {
    Boolean {
        name: String,
        enabled: bool,
    },
    Actor {
        name: String,
        target: String,
        enabled: bool,
    },
    Group {
        name: String,
        target: String,
        enabled: bool,
    },
    Time {
        name: String,
        target: f64,
        enabled: bool,
    },
    Percentage {
        name: String,
        target: f64,
        enabled: bool,
    },
}

impl FeatureFlag {
    pub fn enabled<'a>(&'a self) -> &'a bool {
        use FeatureFlag::*;

        match self {
            Boolean { enabled, .. } => enabled,
            Actor { enabled, .. } => enabled,
            Group { enabled, .. } => enabled,
            Time { enabled, .. } => enabled,
            Percentage { enabled, .. } => enabled,
        }
    }
    // pub fn to_insertable<'a>(&'a self) -> NewFeatureFlag<'a> {
    //     use FeatureFlag::*;

    //     match self {
    //         Boolean { name, enabled } => NewFeatureFlag {
    //             flag_name: name,
    //             gate_type: "boolean",
    //             target: "_fwf_none".to_string(),
    //             enabled: enabled,
    //         },
    //         Actor {
    //             name,
    //             target,
    //             enabled,
    //         } => NewFeatureFlag {
    //             flag_name: name,
    //             gate_type: "actor",
    //             target: target.to_string(),
    //             enabled: enabled,
    //         },
    //         Group {
    //             name,
    //             target,
    //             enabled,
    //         } => NewFeatureFlag {
    //             flag_name: name,
    //             gate_type: "group",
    //             target: target.to_string(),
    //             enabled: enabled,
    //         },
    //         Time {
    //             name,
    //             target,
    //             enabled,
    //         } => NewFeatureFlag {
    //             flag_name: name,
    //             gate_type: "percentage",
    //             target: format!("time/{}", target),
    //             enabled: enabled,
    //         },
    //         Percentage {
    //             name,
    //             target,
    //             enabled,
    //         } => NewFeatureFlag {
    //             flag_name: name,
    //             gate_type: "percentage",
    //             target: format!("actors/{}", target),
    //             enabled: enabled,
    //         },
    //     }
    // }

    // // pub fn to_filter<'a>(&'a self) -> (&'a str, &'a str, String) {
    // pub fn to_filter<'a>(
    //     &'a self,
    // ) -> Box<dyn BoxableExpression<fun_with_flags_toggles::table, DB, SqlType = Bool> + 'a> {
    //     use crate::schema::fun_with_flags_toggles::dsl::*;
    //     use FeatureFlag::*;

    //     let insertable = self.to_insertable();
    //     match self {
    //         Time { .. } | Percentage { .. } => Box::new(
    //             flag_name
    //                 .eq(insertable.flag_name)
    //                 .and(gate_type.eq(insertable.gate_type)),
    //         ),

    //         _ => Box::new(
    //             flag_name
    //                 .eq(insertable.flag_name)
    //                 .and(gate_type.eq(insertable.gate_type))
    //                 .and(target.eq(insertable.target)),
    //         ),
    //     }
    // }
}

// impl Insertable<fun_with_flags_toggles::table> for FeatureFlag {
//     // type Values = (String, String, String, bool);
//     type Values = NewFeatureFlag;
//     // type Values = NewFeatureFlag<'a>;

//     fn values(self) -> Self::Values {
//         use FeatureFlag::*;

//         println!("{:?}", self);
//         match self {
//             Boolean { name, enabled } => NewFeatureFlag {
//                 flag_name: name,
//                 gate_type: "boolean".to_string(),
//                 target: "_fwf_none".to_string(),
//                 enabled: enabled,
//             },

//             _ => unreachable!(),
//         }
//     }
// }

// impl diesel::insertable::CanInsertInSingleQuery<DB> for FeatureFlag{

// }

// impl<DB> FromSql<Integer, DB> for FeatureFlag
// where
//     DB: Backend,
//     i32: FromSql<Integer, DB>,
// {
//     fn from_sql(bytes: Option<backend::RawValue<DB>>) -> deserialize::Result<Self> {
//         match i32::from_sql(bytes)? {
//             1 => Ok(MyEnum::A),
//             2 => Ok(MyEnum::B),
//             x => Err(format!("Unrecognized variant {}", x).into()),
//         }
//     }
// }

// #[derive(Insertable)]
// #[table_name = "fun_with_flags_toggles"]
// pub struct NewFeatureFlag {
//     pub flag_name: String,
//     pub gate_type: String,
//     pub target: String,
//     pub enabled: bool,
// }
