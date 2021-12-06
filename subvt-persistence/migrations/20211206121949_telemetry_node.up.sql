CREATE TABLE IF NOT EXISTS telemetry_node (
   id bigint PRIMARY KEY,
   controller_account_id character varying(66),
   name text NOT NULL,
   client_implementation text                   NOT NULL,
   client_version text                          NOT NULL,
   best_block_number bigint,
   best_block_hash VARCHAR(66),
   finalized_block_number bigint,
   finalized_block_hash VARCHAR(66),
   startup_time bigint,
   last_updated TIMESTAMP WITHOUT TIME ZONE     NOT NULL DEFAULT now()
);

CREATE INDEX telemetry_node_idx_controller_account_id
    ON telemetry_node (controller_account_id);
CREATE INDEX telemetry_node_idx_name
    ON telemetry_node (name);