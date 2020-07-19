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
  end
end

defimpl FunWithFlags.Actor, for: BitString do
  def id(str) do
    "#{str}"
  end
end

