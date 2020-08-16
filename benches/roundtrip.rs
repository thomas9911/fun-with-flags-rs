use criterion::{criterion_group, criterion_main, Criterion};
use fun_with_flags::{enable, enabled};

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

            cfg_if::cfg_if! {
                if #[cfg(feature = "cached")] {
                    fun_with_flags::backend::cached::flush_cache()
                }
            }

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

            cfg_if::cfg_if! {
                if #[cfg(feature = "cached")] {
                    fun_with_flags::backend::cached::flush_cache()
                }
            }
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

fn enabling() -> bool {
    enable("oke").unwrap();
    enabled("oke") == true
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut _ctx = TestContext::new();
    c.bench_function("enabled", |b| b.iter(|| enabling()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
