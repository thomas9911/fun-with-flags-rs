//!  
//! # Simple usage
//!
//! ## Boolean gate
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
//! ## Actor gate
//! ```rust
//! use fun_with_flags::{enable_for, enabled_for};
//! # use fun_with_flags::{Backend, FeatureFlag};
//! # let _mock = Backend::default();
//! # let ctx = Backend::set_context();
//! # ctx.expect().returning(|_, _| {
//! #    Ok(vec![FeatureFlag::Actor {
//! #        name: "testing".to_string(),
//! #        target: "person-test".to_string(),
//! #        enabled: true,
//! #    }])
//! # });
//! # let ctx = Backend::get_context();
//! # ctx.expect().returning(|_, _| {
//! #    Ok(FeatureFlag::Actor {
//! #        name: "testing".to_string(),
//! #        target: "person-test".to_string(),
//! #        enabled: true,
//! #    })
//! # });
//!
//! struct Person {
//!     name: String
//! }
//!
//! // implement Actor trait
//! impl fun_with_flags::Actor for Person {
//!     fn feature_flag_id(&self) -> String {
//!         format!("person-{}", self.name)
//!     }
//! }
//!
//! // don't implement Group, by default always returns false
//! impl fun_with_flags::Group for Person {}
//!
//! let test = Person{name: String::from("test")};
//!
//! // enable actor gate `testing`
//! // returns a result
//! enable_for("testing", &test).is_ok();
//!
//! // enabled will return true if the flag is enabled otherwise returns false
//! if enabled_for("testing", &test) {
//!     // do something
//! } else {
//!     // do something else
//! #    panic!()
//! }
//! ```
//!
//! ## Group gate
//! ```rust
//! use fun_with_flags::{enable_for_group, disable_for_group, enabled_for};
//! # use fun_with_flags::{Backend, FeatureFlag};
//! # let _mock = Backend::default();
//! # let ctx = Backend::set_context();
//! # ctx.expect().returning(|_, _| {
//! #    Ok(vec![FeatureFlag::Actor {
//! #        name: "testing".to_string(),
//! #        target: "person-test".to_string(),
//! #        enabled: true,
//! #    }])
//! # });
//! # let ctx = Backend::get_context();
//! # ctx.expect().returning(|_, _| {
//! #    Ok(FeatureFlag::Actor {
//! #        name: "testing".to_string(),
//! #        target: "person-test".to_string(),
//! #        enabled: true,
//! #    })
//! # });
//!
//! struct Person {
//!     name: String
//! }
//!
//! // implement Actor trait
//! impl fun_with_flags::Actor for Person {
//!     fn feature_flag_id(&self) -> String {
//!         format!("person-{}", self.name)
//!     }
//! }
//! 
//! // implement Group trait
//! impl fun_with_flags::Group for Person {
//!     fn is_in_group(&self, group_name: &str) -> bool {
//!         match group_name {
//!             "tests" => true,
//!             // you can ofcourse do a match on the property of your struct
//!             name if name == self.name => true,
//!             _ => false,
//!         }
//!     }
//! }
//!
//! let test = Person{name: String::from("Johnny Test")};
//!
//! // enable feature flag `testing` for group `tests`
//! // returns a result
//! enable_for_group("testing", "tests").is_ok();
//!
//! // enabled will return true if the flag is enabled otherwise returns false
//! if enabled_for("testing", &test) {
//!     // do something
//! } else {
//!     // do something else
//! #    panic!()
//! };
//!
//! // disable for `tests` group
//! disable_for_group("testing", "tests").is_ok();
//! // enable for `Johnny Test` or the name of the Person
//! enable_for_group("testing", "Johnny Test").is_ok();
//!
//! if enabled_for("testing", &test) {
//!     // do something
//! } else {
//!     // do something else
//! #    panic!()
//! }
//! ```
//!
//! ## Percentage of time gate
//! ```rust
//! use fun_with_flags::{enable_percentage_of_time, enabled};
//! # use fun_with_flags::{Backend, FeatureFlag};
//! # let _mock = Backend::default();
//! # let ctx = Backend::set_context();
//! # ctx.expect().returning(|_, _| {
//! #    Ok(vec![FeatureFlag::Time {
//! #        name: "testing".to_string(),
//! #        target: 0.05,
//! #        enabled: true,
//! #    }])
//! # });
//! # let ctx = Backend::get_context();
//! # ctx.expect().returning(|_, _| {
//! #    Ok(FeatureFlag::Time {
//! #        name: "testing".to_string(),
//! #        target: 0.05,
//! #        enabled: true,
//! #    })
//! # });
//!
//! // enable flag `testing` of 5% of the time
//! // returns a result
//! enable_percentage_of_time("testing", 0.05).is_ok();
//!
//! // enabled will return true for 5% of the time the other 95% returns false
//! if enabled("testing") {
//!     // do 5% of the time
//! } else {
//!     // do 95% of the time
//! }
//! ```
//!
//! ## Percentage of actors gate
//! ```rust
//! use fun_with_flags::{enable_percentage_of_actors, enabled_for};
//! # use fun_with_flags::{Backend, FeatureFlag};
//! # let _mock = Backend::default();
//! # let ctx = Backend::set_context();
//! # ctx.expect().returning(|_, _| {
//! #    Ok(vec![FeatureFlag::Percentage {
//! #        name: "testing".to_string(),
//! #        target: 0.05,
//! #        enabled: true,
//! #    }])
//! # });
//! # let ctx = Backend::get_context();
//! # ctx.expect().returning(|_, _| {
//! #    Ok(FeatureFlag::Percentage {
//! #        name: "testing".to_string(),
//! #        target: 0.05,
//! #        enabled: true,
//! #    })
//! # });
//!
//! struct Person {
//!     name: String
//! }
//!
//! // implement Actor trait
//! impl fun_with_flags::Actor for Person {
//!     fn feature_flag_id(&self) -> String {
//!         format!("person-{}", self.name)
//!     }
//! }
//!
//! // don't implement Group, by default always returns false
//! impl fun_with_flags::Group for Person {}
//!
//! let test = Person{name: String::from("test")};
//!
//! // enable flag `testing` of 5% of the time based on the Actor.feature_flag_id
//! // returns a result
//! enable_percentage_of_actors("testing", 0.05).is_ok();
//!
//! // enabled will return true for 5% of the time based on the actor given the other 95% returns false
//! if enabled_for("testing", &test) {
//!     // do 5% of the time based on `test`
//! # panic!()
//! } else {
//!     // do 95% of the time
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
//! For Postgres you also need to add the fun_with_flags_toggles table to your database. The migration can be found in [migrations/postgres/up.sql](../master/migrations/postgres/up.sql).
//!
//! For more explanation look at the fun-with-flags elixir project.
//!
//! After choosing your backend you must set the DATABASE_URL and DATABASE_NAME enviroment variables. This can also be set in a `.env` file.
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
//!
//! Or use the `fun-with-flags.toml` configuration file, you can omit the configuration for the backend you don't intend to use.
//! ```toml
//! [general]
//! # pick your backend
//! backend = "redis"
//!
//! [redis]
//! # redis configurations
//! # 'url' can be anything that can be parsed by redis: `https://docs.rs/redis/latest/redis/fn.parse_redis_url.html`
//! url = "redis://localhost"
//!
//! [postgres]
//! # postgres configurations
//! # 'url' can be anything that can be parsed by postgres: `https://docs.rs/postgres/latest/postgres/config/struct.Config.html`
//! url = "postgres://username:password@localhost"
//! ```

extern crate dotenv;

#[macro_use]
extern crate serde_derive;

pub use models::FeatureFlag;
pub use traits::{Actor, Group};
pub mod backend;
pub mod models;

pub mod config;
pub mod functions;
pub mod traits;

pub use backend::{Backend, DBConnection, SetOutput as Output};
pub use functions::*;

#[cfg(test)]
mod tests {
    use crate::{
        enable, enable_for, enable_for_group, enable_percentage_of_actors, enabled_for, Actor,
        Backend, FeatureFlag, Group,
    };

    use crate::models::GroupSet;

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

    #[test]
    #[serial]
    fn percentage_enable_for_test() {
        let _mock = Backend::default();

        let ctx = Backend::set_context();
        ctx.expect().returning(|_, _| {
            Ok(vec![FeatureFlag::Percentage {
                name: "testing".to_string(),
                target: 0.40,
                enabled: true,
            }])
        });
        let ctx = Backend::get_context();
        ctx.expect().returning(|_, _| {
            Ok(FeatureFlag::Percentage {
                name: "testing".to_string(),
                target: 0.40,
                enabled: true,
            })
        });

        enable_percentage_of_actors("testing", 0.40).unwrap();

        assert!(enabled_for("testing", &"test"));
    }

    #[test]
    #[serial]
    fn enable_for_group_test() {
        let _mock = Backend::default();

        let ctx = Backend::set_context();
        ctx.expect().returning(|_, _| {
            Ok(vec![FeatureFlag::Group {
                name: "testing".to_string(),
                target: GroupSet::new("tests".to_string()),
                enabled: true,
            }])
        });
        let ctx = Backend::get_context();
        ctx.expect().returning(|_, _| {
            Ok(FeatureFlag::Group {
                name: "testing".to_string(),
                target: GroupSet::new("tests".to_string()),
                enabled: true,
            })
        });

        struct Test;

        impl Group for Test {
            fn is_in_group(&self, group_name: &str) -> bool {
                match group_name {
                    "tests" => true,
                    _ => false,
                }
            }
        }

        impl Actor for Test {
            fn feature_flag_id(&self) -> String {
                String::from("TESTING")
            }
        }

        enable_for_group("testing", "tests").unwrap();

        assert!(enabled_for("testing", &Test {}));
    }
}
