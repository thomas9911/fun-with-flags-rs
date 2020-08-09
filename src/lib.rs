//!  
//! # simple usage
//!
//! ```rust
//! use fun_with_flags::{enable, enabled};
//! # use fun_with_flags::{Backend, FeatureFlag};
//! # let _mock = Backend::default();
//! # let ctx = Backend::set_context();
//! # ctx.expect().returning(|_, _| {
//! #    Ok(vec![FeatureFlag::Boolean {
//! #        name: "testing".to_string(),
//! #        enabled: true,
//! #    }])
//! # });
//! # let ctx = Backend::get_context();
//! # ctx.expect().returning(|_, _| {
//! #    Ok(FeatureFlag::Boolean {
//! #        name: "testing".to_string(),
//! #        enabled: true,
//! #    })
//! # });
//!
//! // enable boolean flag `testing`
//! // returns a result, Ok when the flag got set otherwise returns an Error
//! enable("testing").is_ok();
//!
//! // enabled will return true if the flag is enabled otherwise returns false
//! if enabled("testing") {
//!     // do something
//! } else {
//!     // do something else
//! #    panic!()
//! }
//! ```
//!
//! # Backends
//!
//! Currently supports two backends Postgres and Redis.
//!
//! For redis add: `features = ["redis-backend"]` to Cargo.toml
//!
//! For Postgres add: `features = ["postgres-backend"]` to Cargo.toml
//!
//! For Postgres you also need to add the fun_with_flags_toggles table to your database. The migration can be found in [migrations/00000000000001_create_feature_flags_table/up.sql](../master/migrations/00000000000001_create_feature_flags_table/up.sql).
//!
//! For more explanation look at the fun-with-flags elixir project.
//!
//! After choosing your backend you must set the DATABASE_URL and DATABASE_NAME enviroment variables. This can also be set in a `.env` file. In the future it can also be set in a config file.
//!
//! ```bash
//! # for postgres
//! export DATABASE_ADDRESS=postgres://username:password@localhost
//! export DATABASE_NAME=fun_with_flags_repo
//! ```
//!
//! ```bash
//! # for redis
//! export DATABASE_ADDRESS=redis://localhost
//! export DATABASE_NAME=0
//! ```

#[macro_use]
#[cfg(feature = "postgres-backend")]
extern crate diesel;
extern crate dotenv;

pub use models::FeatureFlag;
pub use traits::{Actor, Group};
pub mod backend;
pub mod models;

#[cfg(feature = "postgres-backend")]
pub mod schema;

pub mod functions;
pub mod traits;

pub use backend::{Backend, DBConnection, SetOutput as Output};
pub use functions::*;

#[cfg(test)]
mod tests {
    use crate::{enable, enable_for, Backend, FeatureFlag};
    use serial_test::serial;

    mod scores {
        // tested to work the same as the elixir implementation
        use crate::score;

        macro_rules! assert_near_equal {
            ($left:expr, $right:expr) => {
                if !float_cmp::approx_eq!(f64, $left, $right) {
                    panic!(
                        r#"assertion failed: `(left == right)`
    left: `{:?}`,
    right: `{:?}`"#,
                        $left, $right
                    )
                }
            };
        }

        #[test]
        fn test_1() {
            let expected: f64 = 0.6754302978515625;
            let score = score("testing", &"test1234");

            assert_near_equal!(expected, score)
        }

        #[test]
        fn test_2() {
            let expected: f64 = 0.3940582275390625;
            let score = score("testing", &"123456789");

            assert_near_equal!(expected, score)
        }
    }

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
