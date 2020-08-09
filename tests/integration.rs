use serial_test::serial;

#[cfg(feature = "postgres-backend")]
#[macro_use]
extern crate diesel_migrations;

struct Person {
    name: String,
}

impl fun_with_flags::Actor for Person {
    fn feature_flag_id(&self) -> String {
        format!("person-{}", self.name)
    }
}

impl fun_with_flags::Group for Person {
    fn is_in_group(&self, group_name: &str) -> bool {
        match group_name {
            "test" => true,
            _ => false,
        }
    }
}

#[cfg(feature = "postgres-backend")]
mod postgres_test_context {
    extern crate diesel;

    use diesel::connection::SimpleConnection;
    use diesel::RunQueryDsl;

    use diesel_migrations::MigrationConnection;

    use fun_with_flags;

    const MAIN_DATABASE: &'static str = "postgres";

    embed_migrations!();

    pub struct TestContext {
        is_dropped: bool,
    }

    impl TestContext {
        pub fn new() -> Self {
            println!("Set up resources");
            let conn = fun_with_flags::establish_connection_to_database(MAIN_DATABASE);
            Self::create_db(&conn);

            let conn = fun_with_flags::establish_connection();

            Self::migrate(&conn);
            println!("Set up resources, done");

            Self { is_dropped: false }
        }

        fn clean(&mut self) {
            println!("Clean up resources");
            let conn = fun_with_flags::establish_connection_to_database(MAIN_DATABASE);

            let disconnect_users = "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = 'fun_with_flags_repo';";

            diesel::sql_query(disconnect_users).execute(&conn).unwrap();

            let _query = diesel::sql_query("DROP DATABASE fun_with_flags_repo")
                .execute(&conn)
                .expect("Couldn't drop database fun_with_flags_repo");

            self.is_dropped = true;
        }

        fn create_db<C: SimpleConnection>(conn: &C) {
            conn.batch_execute("CREATE DATABASE fun_with_flags_repo;")
                .unwrap();
        }

        fn migrate<C: MigrationConnection>(conn: &C) {
            embedded_migrations::run(conn).unwrap();
        }
    }

    impl Drop for TestContext {
        fn drop(&mut self) {
            if !self.is_dropped {
                self.clean()
            }
        }
    }
}

#[cfg(feature = "redis-backend")]
mod redis_test_context {
    pub struct TestContext;

    impl TestContext {
        pub fn new() -> Self {
            let db = fun_with_flags::establish_connection();

            fun_with_flags::Backend::clean_all(&db).unwrap();

            TestContext {}
        }
    }

    impl Drop for TestContext {
        fn drop(&mut self) {
            let db = fun_with_flags::establish_connection();

            fun_with_flags::Backend::clean_all(&db).unwrap();
        }
    }
}

mod empty_test_context {
    #[allow(dead_code)]
    pub struct TestContext;

    impl TestContext {
        #[allow(dead_code)]
        pub fn new() -> Self {
            TestContext {}
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "redis-backend")]{
        use redis_test_context::TestContext;
    } else if #[cfg(feature = "postgres-backend")] {
        use postgres_test_context::TestContext;
    } else {
        use empty_test_context::TestContext;
    }
}

#[test]
#[serial]
fn enable() {
    let mut _ctx = TestContext::new();

    let flag_name = "bool_flag";

    assert_eq!(false, fun_with_flags::enabled(flag_name));
    fun_with_flags::enable(flag_name).unwrap();
    assert_eq!(true, fun_with_flags::enabled(flag_name));
    fun_with_flags::disable(flag_name).unwrap();
    assert_eq!(false, fun_with_flags::enabled(flag_name));
}

#[test]
#[serial]
fn enable_for() {
    let mut _ctx = TestContext::new();

    let flag_name = "actor_flag";

    let john = Person {
        name: String::from("john"),
    };

    let pete = Person {
        name: String::from("pete"),
    };

    assert_eq!(false, fun_with_flags::enabled_for(flag_name, &john));
    assert_eq!(false, fun_with_flags::enabled_for(flag_name, &pete));
    fun_with_flags::enable_for(flag_name, &john).unwrap();
    assert_eq!(true, fun_with_flags::enabled_for(flag_name, &john));
    assert_eq!(false, fun_with_flags::enabled_for(flag_name, &pete));
    fun_with_flags::disable_for(flag_name, &john).unwrap();
    assert_eq!(false, fun_with_flags::enabled_for(flag_name, &john));
    assert_eq!(false, fun_with_flags::enabled_for(flag_name, &pete));
}

#[test]
#[serial]
fn enable_chance() {
    let mut _ctx = TestContext::new();

    let flag_name = "times_of_flag";

    assert_eq!(false, fun_with_flags::enabled(flag_name));
    fun_with_flags::enable_percentage_of_time(flag_name, 0.50).unwrap();
    // chance of getting 40 times false in a row for 50% is very small (0.000000000090949470177292823792%)
    let result = (0..40)
        .map(|_x| fun_with_flags::enabled(flag_name))
        .any(|x| x);

    assert_eq!(true, result);

    fun_with_flags::disable_percentage_of_time(flag_name).unwrap();
    let result = (0..40)
        .map(|_x| fun_with_flags::enabled(flag_name))
        .any(|x| x);
    assert_eq!(false, result);
}

#[test]
#[serial]
fn enable_percentage_of_actors() {
    let mut _ctx = TestContext::new();

    let flag_name = "actor_percentage_flag";

    let john = Person {
        name: String::from("john"),
    };

    let pete = Person {
        name: String::from("pete"),
    };

    assert_eq!(false, fun_with_flags::enabled_for(flag_name, &john));
    assert_eq!(false, fun_with_flags::enabled_for(flag_name, &pete));

    fun_with_flags::enable_percentage_of_actors(flag_name, 0.4).unwrap();
    // score for john is about 0.44
    // score for pete is about 0.34
    assert_eq!(false, fun_with_flags::enabled_for(flag_name, &john));
    assert_eq!(true, fun_with_flags::enabled_for(flag_name, &pete));
    fun_with_flags::disable_percentage_of_actors(flag_name).unwrap();
    assert_eq!(false, fun_with_flags::enabled_for(flag_name, &john));
    assert_eq!(false, fun_with_flags::enabled_for(flag_name, &pete));
}

#[test]
#[serial]
fn enable_for_group() {
    let mut _ctx = TestContext::new();

    let flag_name = "group_flag";

    let john = Person {
        name: String::from("john"),
    };

    let pete = Person {
        name: String::from("pete"),
    };

    assert_eq!(false, fun_with_flags::enabled_for(flag_name, &john));
    assert_eq!(false, fun_with_flags::enabled_for(flag_name, &pete));
    fun_with_flags::enable_for_group(flag_name, "test").unwrap();
    assert_eq!(true, fun_with_flags::enabled_for(flag_name, &john));
    assert_eq!(true, fun_with_flags::enabled_for(flag_name, &pete));
    fun_with_flags::disable_for_group(flag_name, "test").unwrap();
    assert_eq!(false, fun_with_flags::enabled_for(flag_name, &john));
    assert_eq!(false, fun_with_flags::enabled_for(flag_name, &pete));
}
