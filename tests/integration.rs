use serial_test::serial;

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
            "johns-group" => self.name == "john",
            _ => false,
        }
    }
}

#[cfg(feature = "postgres-backend")]
mod postgres_test_context {
    use fun_with_flags;
    use postgres::NoTls;

    use r2d2_postgres::PostgresConnectionManager;

    type PostgresClient = r2d2::PooledConnection<PostgresConnectionManager<NoTls>>;

    const MAIN_DATABASE: &'static str = "postgres";

    pub struct TestContext {
        is_dropped: bool,
    }

    impl TestContext {
        pub fn new() -> Self {
            println!("Set up resources");
            let conn = fun_with_flags::establish_connection_to_database(MAIN_DATABASE);
            Self::create_db(&conn);

            let conn = fun_with_flags::establish_connection().unwrap();

            Self::migrate(&conn);
            println!("Set up resources, done");

            Self { is_dropped: false }
        }

        fn clean(&mut self) {
            println!("Clean up resources");
            let conn = fun_with_flags::establish_connection_to_database(MAIN_DATABASE);
            Self::drop_db(&conn);

            self.is_dropped = true;
        }

        fn get_client(conn: &fun_with_flags::DBConnection) -> PostgresClient {
            fun_with_flags::Backend::create_conn(conn).unwrap()
        }

        fn drop_db(conn: &fun_with_flags::DBConnection) {
            let mut client = Self::get_client(conn);

            client.batch_execute("SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = 'fun_with_flags_repo';")
                .unwrap();
            client
                .batch_execute("DROP DATABASE fun_with_flags_repo")
                .expect("Couldn't drop database fun_with_flags_repo");

        }

        fn create_db(conn: &fun_with_flags::DBConnection) {
            let mut client = Self::get_client(conn);
            client
                .batch_execute("CREATE DATABASE fun_with_flags_repo;")
                .unwrap();
        }

        fn migrate(conn: &fun_with_flags::DBConnection) {
            let mut client = Self::get_client(conn);
            let migration = include_str!("../migrations/postgres/up.sql");
            client.batch_execute(migration).unwrap();
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
            let db = fun_with_flags::establish_connection().unwrap();

            fun_with_flags::Backend::clean_all(&db).unwrap();

            TestContext {}
        }
    }

    impl Drop for TestContext {
        fn drop(&mut self) {
            let db = fun_with_flags::establish_connection().unwrap();

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

#[test]
#[serial]
fn enable_for_group_specific() {
    let mut _ctx = TestContext::new();

    let flag_name = "group2_flag";

    let john = Person {
        name: String::from("john"),
    };

    let pete = Person {
        name: String::from("pete"),
    };

    assert_eq!(false, fun_with_flags::enabled_for(flag_name, &john));
    assert_eq!(false, fun_with_flags::enabled_for(flag_name, &pete));
    fun_with_flags::enable_for_group(flag_name, "johns-group").unwrap();
    assert_eq!(true, fun_with_flags::enabled_for(flag_name, &john));
    assert_eq!(false, fun_with_flags::enabled_for(flag_name, &pete));
    fun_with_flags::disable_for_group(flag_name, "johns-group").unwrap();
    assert_eq!(false, fun_with_flags::enabled_for(flag_name, &john));
    assert_eq!(false, fun_with_flags::enabled_for(flag_name, &pete));
}
