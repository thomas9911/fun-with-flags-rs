defmodule FunWithFlagsRs.Repo do
  use Ecto.Repo,
    otp_app: :fun_with_flags_rs,
    adapter: Ecto.Adapters.Postgres
end
