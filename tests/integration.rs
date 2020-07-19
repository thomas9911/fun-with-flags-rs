extern crate diesel;

use diesel::connection::SimpleConnection;
use diesel::RunQueryDsl;
use serial_test::serial;

#[macro_use]
extern crate diesel_migrations;
use diesel_migrations::MigrationConnection;

use fun_with_flags;

const MAIN_DATABASE: &'static str = "postgres";

embed_migrations!();

struct TestContext {
    is_dropped: bool,
}

impl TestContext {
    fn new() -> Self {
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

struct Person {
    name: String,
}

impl fun_with_flags::Actor for Person {
    fn feature_flag_id(&self) -> String {
        format!("person-{}", self.name)
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

    assert_eq!(false, fun_with_flags::enabled_for(flag_name, &john));
    fun_with_flags::enable_for(flag_name, &john).unwrap();
    assert_eq!(true, fun_with_flags::enabled_for(flag_name, &john));
    fun_with_flags::disable_for(flag_name, &john).unwrap();
    assert_eq!(false, fun_with_flags::enabled_for(flag_name, &john));
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



