![Rust](https://github.com/thomas9911/fun-with-flags-rs/workflows/Rust/badge.svg)

# fun-with-flags


## simple usage

```rust
use fun_with_flags::{enable, enabled};

// enable boolean flag `testing`
// returns a result, Ok when the flag got set otherwise returns an Error
enable("testing").is_ok();

// enabled will return true if the flag is enabled otherwise returns false
if enabled("testing") {
    // do something
} else {
    // do something else
}
```

## Backends

Currently supports two backends Postgres and Redis.

For redis add: `features = ["redis-backend"]` to Cargo.toml

For Postgres add: `features = ["postgres-backend"]` to Cargo.toml

For Postgres you also need to add the fun_with_flags_toggles table to your database. The migration can be found in [migrations/postgres/up.sql](../master/migrations/postgres/up.sql).

For more explanation look at the fun-with-flags elixir project.

After choosing your backend you must set the DATABASE_URL and DATABASE_NAME enviroment variables. This can also be set in a `.env` file.

```bash
# for postgres
export DATABASE_ADDRESS=postgres://username:password@localhost
export DATABASE_NAME=fun_with_flags_repo
```

```bash
# for redis
export DATABASE_ADDRESS=redis://localhost
export DATABASE_NAME=0
```

Or use the `fun-with-flags.toml` configuration file, you can omit the configuration for the backend you don't intend to use.
```toml
[general]
# pick your backend
backend = "redis"

[redis]
# redis configurations
# 'url' can be anything that can be parsed by redis: `https://docs.rs/redis/latest/redis/fn.parse_redis_url.html`
url = "redis://localhost"

[postgres]
# postgres configurations
# 'url' can be anything that can be parsed by postgres: `https://docs.rs/postgres/latest/postgres/config/struct.Config.html`
url = "postgres://username:password@localhost"
```

Current version: 0.1.0

License: Unlicense
