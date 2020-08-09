defmodule FunWithFlagsRs do
  def run do
    # create a bunch of flags

    FunWithFlags.enable(:boolean_one)
    FunWithFlags.enable(:boolean_two)
    FunWithFlags.enable(:boolean_three)
    FunWithFlags.enable(:actor_one, for_actor: "user_1")
    FunWithFlags.enable(:actor_one, for_actor: "user_2")
    FunWithFlags.enable(:actor_two, for_actor: "user_1")
    FunWithFlags.enable(:actor_two, for_actor: "user_3")
    FunWithFlags.enable(:actor_two, for_actor: "user_3")
    FunWithFlags.enable(:group_one, for_group: "group_1")
    FunWithFlags.enable(:time_one, for_percentage_of: {:time, 0.05})
    FunWithFlags.enable(:actor_percentage_one, for_percentage_of: {:actors, 0.60})
  
    FunWithFlags.enable(:mixed)
    FunWithFlags.enable(:mixed, for_actor: "user_1")
    FunWithFlags.enable(:mixed, for_percentage_of: {:time, 0.05})
  
  end
end

defimpl FunWithFlags.Actor, for: BitString do
  def id(str) do
    "#{str}"
  end
end



defmodule FunWithFlagsRs.HashTest do
  @moduledoc false

  alias FunWithFlags.Actor

  # Combine an actor id and a flag name to get
  # a score. The flag name must be included to
  # ensure that the same actors get different
  # scores for different flags, but with
  # deterministic and predictable results.
  #
  @spec score(term, atom) :: float
  def score(actor, flag_name) do
    blob = Actor.id(actor) <> to_string(flag_name)
    _actor_score(blob)
  end

  # first 16 bits:
  # 2 ** 16 = 65_536
  #
  # %_ratio : 1.0 = 16_bits : 65_536
  #
  defp _actor_score(string) do
    IO.puts(string)
    hash = :crypto.hash(:sha256, string) |> IO.inspect()
    # IO.inspect({hash[0], hash[1]})
    <<score :: size(16), _rest :: binary>> = hash
    IO.inspect(score)
    score / 65_536
  end

end