CREATE TABLE IF NOT EXISTS sub_telemetry_node (
   id                       bigint PRIMARY KEY,
   controller_account_id    character varying(66),
   name                     text NOT NULL,
   client_implementation    text NOT NULL,
   client_version           text NOT NULL,
   best_block_number        bigint,
   best_block_hash          VARCHAR(66),
   finalized_block_number   bigint,
   finalized_block_hash     VARCHAR(66),
   startup_time             bigint,
   location                 text,
   latitude                 double precision,
   longitude                double precision,
   created_at               TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
   updated_at               TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS sub_telemetry_node_idx_controller_account_id
    ON sub_telemetry_node (controller_account_id);
CREATE INDEX IF NOT EXISTS sub_telemetry_node_idx_name
    ON sub_telemetry_node (name);