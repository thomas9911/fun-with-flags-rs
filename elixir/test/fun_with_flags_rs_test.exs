defmodule FunWithFlagsRsTest do
  use ExUnit.Case
  doctest FunWithFlagsRs

  test "greets the world" do
    assert FunWithFlagsRs.hello() == :world
  end
end
