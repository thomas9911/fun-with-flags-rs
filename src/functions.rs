use crate::config::{fetch_config, ConfigError};
use crate::models::GroupSet;
use crate::{Actor, Backend, DBConnection, FeatureFlag, Group, Output};

pub fn establish_connection() -> Result<DBConnection, ConfigError> {
    let config = fetch_config().expect("database config is not correctly set");

    let url = config.to_url().ok_or(ConfigError::Message(
        "database config can not find correct url".into(),
    ))?;
    DBConnection::establish(&url).or(Err(ConfigError::Message(format!(
        "Error connecting to {}",
        url
    ))))
}

/// only used for debug purposes
pub fn establish_connection_to_database(database_name: &str) -> DBConnection {
    let config = fetch_config().unwrap();

    if let Some((database_url, _name)) = config.parts() {
        DBConnection::establish(&format!("{}/{}", database_url, database_name))
            .expect(&format!("Error connecting to {}", database_url))
    } else {
        panic!("DATABASE_URL must be set")
    }
}

pub fn enable(flag: &str) -> Output {
    let conn = establish_connection().unwrap();
    Backend::set(
        &conn,
        FeatureFlag::Boolean {
            name: flag.to_string(),
            enabled: true,
        },
    )
}

pub fn enable_for<T: Actor>(flag: &str, actor: &T) -> Output {
    let conn = establish_connection().unwrap();
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
    let conn = establish_connection().unwrap();
    Backend::set(
        &conn,
        FeatureFlag::Boolean {
            name: flag.to_string(),
            enabled: false,
        },
    )
}

pub fn disable_for<T: Actor>(flag: &str, actor: &T) -> Output {
    let conn = establish_connection().unwrap();
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
    let conn = establish_connection().unwrap();

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

pub fn enabled_for<T: Actor + Group>(flag: &str, actor: &T) -> bool {
    let conn = establish_connection().unwrap();

    if let Ok(FeatureFlag::Actor { enabled: true, .. }) = Backend::get(
        &conn,
        FeatureFlag::Actor {
            name: flag.to_string(),
            target: actor.feature_flag_id(),
            enabled: true,
        },
    ) {
        return true;
    };

    if let Ok(FeatureFlag::Group {
        target,
        enabled: true,
        ..
    }) = Backend::get(
        &conn,
        FeatureFlag::Group {
            name: flag.to_string(),
            target: GroupSet::default(),
            enabled: true,
        },
    ) {
        return target.check(actor);
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
        let score = score(flag, actor);
        println!("{}", score);
        return target > score;
    };

    false
}

pub fn disabled_for<T: Actor + Group>(flag: &str, actor: &T) -> bool {
    !enabled_for(flag, actor)
}

pub fn enable_percentage_of_time(flag: &str, percentage: f64) -> Output {
    let conn = establish_connection().unwrap();
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
    let conn = establish_connection().unwrap();
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
    let conn = establish_connection().unwrap();
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
    let conn = establish_connection().unwrap();
    Backend::set(
        &conn,
        FeatureFlag::Percentage {
            name: flag.to_string(),
            target: 0.0,
            enabled: false,
        },
    )
}

pub fn enable_for_group(flag: &str, group_name: &str) -> Output {
    let conn = establish_connection().unwrap();
    Backend::set(
        &conn,
        FeatureFlag::Group {
            name: flag.to_string(),
            target: GroupSet::new(group_name.to_string()),
            enabled: true,
        },
    )
}

pub fn disable_for_group(flag: &str, group_name: &str) -> Output {
    let conn = establish_connection().unwrap();
    Backend::set(
        &conn,
        FeatureFlag::Group {
            name: flag.to_string(),
            target: GroupSet::new(group_name.to_string()),
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
    let first_byte = result[0] as u16;
    let second_byte = result[1] as u16;

    let num = first_byte.wrapping_shl(8) + second_byte;
    num as f64 / 65_536f64
}

fn generate_0_1() -> f64 {
    use rand::distributions::OpenClosed01;
    use rand::{thread_rng, Rng};

    thread_rng().sample(OpenClosed01)
}


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
            Ok(FeatureFlag::Boolean {
                name: "oke".to_string(),
                enabled: true,
            })
        });

        assert!(enable("oke").is_ok());
    }

    #[test]
    #[serial]
    fn enable_for_test() {
        let _mock = Backend::default();

        let ctx = Backend::set_context();
        ctx.expect().returning(|_, _| {
            Ok(FeatureFlag::Actor {
                name: "oke".to_string(),
                target: "testing".to_string(),
                enabled: true,
            })
        });

        assert!(enable_for("oke", &"testing").is_ok());
    }

    #[test]
    #[serial]
    fn percentage_enable_for_test() {
        let _mock = Backend::default();

        let ctx = Backend::set_context();
        ctx.expect().returning(|_, _| {
            Ok(FeatureFlag::Percentage {
                name: "testing".to_string(),
                target: 0.40,
                enabled: true,
            })
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
            Ok(FeatureFlag::Group {
                name: "testing".to_string(),
                target: GroupSet::new("tests".to_string()),
                enabled: true,
            })
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
