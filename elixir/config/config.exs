import Config

# config :fun_with_flags_rs, FunWithFlagsRs.Repo,
#   database: "fun_with_flags_rs_repo",
#   username: "username",
#   password: "password",
#   hostname: "localhost"

# config :fun_with_flags_rs, ecto_repos: [FunWithFlagsRs.Repo]

# config :fun_with_flags, :cache, enabled: false

# config :fun_with_flags, :cache_bust_notifications, enabled: false

# config :fun_with_flags, :persistence,
#   adapter: FunWithFlags.Store.Persistent.Ecto,
#   repo: FunWithFlagsRs.Repo,
#   ecto_table_name: "fun_with_flags_toggles"


config :fun_with_flags, :redis,
  host: "localhost",
  port: 6379,
  database: 0

config :fun_with_flags, :persistence,
  [adapter: FunWithFlags.Store.Persistent.Redis]

config :fun_with_flags, :cache, enabled: false

config :fun_with_flags, :cache_bust_notifications, enabled: false
