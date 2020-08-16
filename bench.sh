case "$1" in
    redis)
        # DATABASE_ADDRESS=redis://localhost DATABASE_NAME=0 cargo run --features "rayon postgres-backend" --example works
        DATABASE_ADDRESS=redis://localhost DATABASE_NAME=0 cargo bench --features "bench redis-backend"
    ;;
    redis-c)
        DATABASE_ADDRESS=redis://localhost DATABASE_NAME=0 cargo bench --features "bench cached redis-backend"
    ;;
    postgres)
        # DATABASE_ADDRESS=postgres://username:password@localhost DATABASE_NAME=fun_with_flags_repo cargo run --features "postgres-backend rayon" --example works
        DATABASE_ADDRESS=postgres://username:password@localhost DATABASE_NAME=fun_with_flags_repo cargo bench --features "bench postgres-backend"
    ;;
    postgres-c)
        DATABASE_ADDRESS=postgres://username:password@localhost DATABASE_NAME=fun_with_flags_repo cargo bench --features "bench cached postgres-backend"
    ;;
    *)
        echo "run benchmark"
        echo "options are 'redis', 'redis-c', 'postgres', postgres-c'"
esac
