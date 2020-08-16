# ./test_unit.sh && ./test_redis.sh && ./test_postgres.sh && ./test_cached_redis.sh && ./test_cached_postgres.sh

function redis_test {
    DATABASE_ADDRESS=redis://localhost DATABASE_NAME=0 cargo test --features redis-backend --test integration
}

function redis_cached_test {
    DATABASE_ADDRESS=redis://localhost DATABASE_NAME=0 cargo test --features "redis-backend cached" --test integration
}

function postgres_test {
    DATABASE_ADDRESS=postgres://username:password@localhost DATABASE_NAME=fun_with_flags_repo cargo test --features postgres-backend --test integration
}

function postgres_cached_test {
    DATABASE_ADDRESS=postgres://username:password@localhost DATABASE_NAME=fun_with_flags_repo cargo test --features "postgres-backend cached" --test integration
}

function unit_test {
    DATABASE_ADDRESS="" DATABASE_NAME="" cargo test --lib && \
    DATABASE_ADDRESS="" DATABASE_NAME="" cargo test --doc
}

function all_test {
    unit_test
    redis_test
    postgres_test
    redis_cached_test
    postgres_cached_test
}

case "$1" in
    unit)
        unit_test
    ;;
    redis)
        redis_test
    ;;
    redis-c)
        redis_cached_test
    ;;
    postgres)
        postgres_test
    ;;
    postgres-c)
        postgres_cached_test
    ;;
    help)
        echo "run tests"
        echo "options are 'unit', 'redis', 'redis-c', 'postgres', postgres-c'"
    ;;
    *)
        all_test
esac
