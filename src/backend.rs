use crate::FeatureFlag;

cfg_if::cfg_if! {
    if #[cfg(test)] {
        pub mod null;
        pub use null::{MockBackend as Backend, DBConnection, GetOutput, SetOutput, DB};

    } else if #[cfg(feature = "redis-backend")]{
        pub mod redis;
        pub use self::redis::{Backend, DBConnection, GetOutput, SetOutput, DB};
    } else if #[cfg(feature = "postgres-backend")] {
        pub mod postgres;
        pub use postgres::{Backend, DBConnection, GetOutput, SetOutput, DB};
    } else {
        pub mod null;
        pub use null::{MockBackend as Backend, DBConnection, GetOutput, SetOutput, DB};
    }
}

pub fn set(conn: &DBConnection, flag: FeatureFlag) -> SetOutput {
    Backend::set(conn, flag)
}

pub fn get(conn: &DBConnection, flag: FeatureFlag) -> GetOutput {
    Backend::get(conn, flag)
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn connection_name() {
//         assert_eq!("mock", Backend::backend_name());
//     }

//     #[test]
//     fn get_returns_error() {
//         use crate::establish_connection;

//         let conn = establish_connection();
//         let ff = FeatureFlag::Boolean {
//             name: "something".to_string(),
//             enabled: true,
//         };
//         assert!(Backend::get(&conn, ff).is_err());
//     }

//     #[test]
//     fn set_returns_error() {
//         use crate::establish_connection;

//         let conn = establish_connection();
//         let ff = FeatureFlag::Boolean {
//             name: "something".to_string(),
//             enabled: true,
//         };
//         assert!(Backend::set(&conn, ff).is_err());
//     }

// }
