#[macro_use]
#[cfg(feature = "postgres-backend")]
extern crate diesel;
extern crate dotenv;

// use diesel::prelude::*;
use dotenv::dotenv;
use std::env;

#[cfg(feature = "postgres-backend")]
#[allow(unused_imports)]
use diesel::Connection;

pub use models::FeatureFlag;
pub use traits::Actor;
pub mod backend;
pub mod models;

#[cfg(feature = "postgres-backend")]
pub mod schema;

pub mod traits;
pub use backend::{Backend, DBConnection, SetOutput as Output};

pub fn establish_connection() -> DBConnection {
    dotenv().ok();

    let database_name = env::var("DATABASE_NAME").expect("DATABASE_NAME must be set");
    establish_connection_to_database(&database_name)
}

pub fn establish_connection_to_database(database_name: &str) -> DBConnection {
    dotenv().ok();
    let database_url = env::var("DATABASE_ADDRESS").expect("DATABASE_URL must be set");

    DBConnection::establish(&format!("{}/{}", database_url, database_name))
        .expect(&format!("Error connecting to {}", database_url))
}

pub fn enable(flag: &str) -> Output {
    let conn = establish_connection();
    Backend::set(
        &conn,
        FeatureFlag::Boolean {
            name: flag.to_string(),
            enabled: true,
        },
    )
}

pub fn enable_for<T: Actor>(flag: &str, actor: &T) -> Output {
    let conn = establish_connection();
    Backend::set(
        &conn,
        FeatureFlag::Actor {
            name: flag.to_string(),
            target: actor.feature_flag_id(),
            enabled: true,
        },
    )
}

pub fn disable(flag: &str) -> Output {
    let conn = establish_connection();
    Backend::set(
        &conn,
        FeatureFlag::Boolean {
            name: flag.to_string(),
            enabled: false,
        },
    )
}

pub fn disable_for<T: Actor>(flag: &str, actor: &T) -> Output {
    let conn = establish_connection();
    Backend::set(
        &conn,
        FeatureFlag::Actor {
            name: flag.to_string(),
            target: actor.feature_flag_id(),
            enabled: false,
        },
    )
}

pub fn enabled(flag: &str) -> bool {
    let conn = establish_connection();

    if let Ok(x) = Backend::get(
        &conn,
        FeatureFlag::Boolean {
            name: flag.to_string(),
            enabled: true,
        },
    ) {
        return *x.enabled();
    };

    if let Ok(FeatureFlag::Time {
        target,
        enabled: true,
        ..
    }) = Backend::get(
        &conn,
        FeatureFlag::Time {
            name: flag.to_string(),
            enabled: true,
            target: 0.0,
        },
    ) {
        return target > generate_0_1();
    };

    false
}

pub fn disabled(flag: &str) -> bool {
    !enabled(flag)
}

pub fn enabled_for<T: Actor>(flag: &str, actor: &T) -> bool {
    let conn = establish_connection();

    // match Backend::get(
    //     &conn,
    //     FeatureFlag::Actor {
    //         name: flag.to_string(),
    //         target: actor.feature_flag_id(),
    //         enabled: true,
    //     },
    // ) {
    //     Ok(x) => *x.enabled(),
    //     Err(_) => false,
    // }

    if let Ok(x) = Backend::get(
        &conn,
        FeatureFlag::Actor {
            name: flag.to_string(),
            target: actor.feature_flag_id(),
            enabled: true,
        },
    ) {
        return *x.enabled();
    };

    if let Ok(FeatureFlag::Percentage {
        target,
        enabled: true,
        ..
    }) = Backend::get(
        &conn,
        FeatureFlag::Percentage {
            name: flag.to_string(),
            enabled: true,
            target: 0.0,
        },
    ) {
        // return target > generate_0_1();
        return target > score(flag, actor)
    };

    false
}

pub fn disabled_for<T: Actor>(flag: &str, actor: &T) -> bool {
    !enabled_for(flag, actor)
}

pub fn enable_percentage_of_time(flag: &str, percentage: f64) -> Output {
    let conn = establish_connection();
    Backend::set(
        &conn,
        FeatureFlag::Time {
            name: flag.to_string(),
            target: percentage,
            enabled: true,
        },
    )
}

pub fn disable_percentage_of_time(flag: &str) -> Output {
    let conn = establish_connection();
    Backend::set(
        &conn,
        FeatureFlag::Time {
            name: flag.to_string(),
            target: 0.0,
            enabled: false,
        },
    )
}

pub fn enable_percentage_of_actors(flag: &str, percentage: f64) -> Output {
    let conn = establish_connection();
    Backend::set(
        &conn,
        FeatureFlag::Percentage {
            name: flag.to_string(),
            target: percentage,
            enabled: true,
        },
    )
}

pub fn disable_percentage_of_actors(flag: &str) -> Output {
    let conn = establish_connection();
    Backend::set(
        &conn,
        FeatureFlag::Percentage {
            name: flag.to_string(),
            target: 0.0,
            enabled: false,
        },
    )
}

pub fn score<T: Actor>(flag: &str, actor: &T) -> f64 {
    let blob = format!("{}{}", actor.feature_flag_id(), flag);
    hash(&blob)
}

fn hash(input: &str) -> f64 {
    use sha2::{Digest, Sha256};

    let result = Sha256::digest(input.as_bytes());
    let num = u16::from_ne_bytes([result[0], result[1]]);
    num as f64 / 65_536f64
}

fn generate_0_1() -> f64 {
    use rand::distributions::OpenClosed01;
    use rand::{thread_rng, Rng};

    thread_rng().sample(OpenClosed01)
}

#[cfg(test)]
mod tests {
    use crate::{enable, enable_for, Backend, FeatureFlag};
    use serial_test::serial;

    #[test]
    #[serial]
    fn enable_test() {
        let _mock = Backend::default();

        let ctx = Backend::set_context();
        ctx.expect().returning(|_, _| {
            Ok(vec![FeatureFlag::Boolean {
                name: "oke".to_string(),
                enabled: true,
            }])
        });

        assert!(enable("oke").is_ok());
    }

    #[test]
    #[serial]
    fn enable_for_test() {
        let _mock = Backend::default();

        let ctx = Backend::set_context();
        ctx.expect().returning(|_, _| {
            Ok(vec![FeatureFlag::Actor {
                name: "oke".to_string(),
                target: "testing".to_string(),
                enabled: true,
            }])
        });

        assert!(enable_for("oke", &"testing").is_ok());
    }
}
