defmodule FunWithFlagsRs.Repo.Migrations.CreateFeatureFlagsTable do
  use Ecto.Migration

  # This migration assumes the default table name of "fun_with_flags_toggles"
  # is being used. If you have overriden that via configuration, you should
  # change this migration accordingly.

  def up do
    create table(:fun_with_flags_toggles, primary_key: false) do
      add :id, :bigserial, primary_key: true
      add :flag_name, :string, null: false
      add :gate_type, :string, null: false
      add :target, :string, null: false
      add :enabled, :boolean, null: false
    end

    create index(
      :fun_with_flags_toggles,
      [:flag_name, :gate_type, :target],
      [unique: true, name: "fwf_flag_name_gate_target_idx"]
    )
  end

  def down do
    drop table(:fun_with_flags_toggles)
  end
end

# create_feature_flags_table

# C:\Program Files\PostgreSQL\12\lib
# cargo install diesel_cli --no-default-features --features postgres
# set path="%path%;C:\Program Files\PostgreSQL\12\lib"


# Path=C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v10.2\bin;C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v10.2\libnvvp;C:\Program Files (x86)\Common Files\Oracle\Java\javapath;C:\Python38\Scripts\;C:\Python38\;C:\Windows\system32;C:\Windows;C:\Windows\System32\Wbem;C:\Windows\System32\WindowsPowerShell\v1.0\;C:\Windows\System32\OpenSSH\;C:\Program Files\Microsoft VS Code\bin;C:\Program Files (x86)\NVIDIA Corporation\PhysX\Common;C:\ProgramData\chocolatey\bin;C:\Program Files\nodejs\;C:\Program Files\Git\cmd;C:\Program Files\Git\mingw64\bin;C:\Program Files\Git\usr\bin;C:\Program Files\Java\jdk1.8.0_211\bin;;C:\ProgramData\chocolatey\lib\Elixir/bin;C:\Program Files\NVIDIA Corporation\Nsight Compute 2019.5.0\;C:\Users\thoma\.cargo\bin;C:\Users\thoma\AppData\Local\Microsoft\WindowsApps;C:\Program Files\Docker Toolbox;C:\Users\thoma\AppData\Roaming\npm;C:\Users\thoma\.windows-build-tools\python27\;D:\MinGW\bin;C:\Program Files (x86)\FAHClient;C:\Program Files\Oracle\VirtualBox;C:\Program Files\LLVM\bin;C:\Program Files\PostgreSQL\12\lib