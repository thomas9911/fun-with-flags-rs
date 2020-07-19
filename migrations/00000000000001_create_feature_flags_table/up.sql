CREATE TABLE fun_with_flags_toggles (
    id BIGSERIAL PRIMARY KEY,
    flag_name VARCHAR NOT NULL,
    gate_type VARCHAR NOT NULL,
    target VARCHAR NOT NULL,
    enabled BOOLEAN NOT NULL
);

CREATE UNIQUE INDEX fwf_flag_name_gate_target_idx ON fun_with_flags_toggles (flag_name, gate_type, target);