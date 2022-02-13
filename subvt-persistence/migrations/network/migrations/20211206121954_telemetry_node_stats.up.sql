CREATE TABLE IF NOT EXISTS sub_telemetry_node_stats (
    time            TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    node_id         bigint NOT NULL,
    peer_count      integer NOT NULL,
    queued_tx_count integer NOT NULL
);

ALTER TABLE sub_telemetry_node_stats
    ADD CONSTRAINT sub_telemetry_node_stats_fk_telemetry_node
    FOREIGN KEY (node_id)
        REFERENCES sub_telemetry_node (id)
        ON DELETE CASCADE
        ON UPDATE CASCADE;

SELECT create_hypertable('sub_telemetry_node_stats', 'time');
SELECT set_chunk_time_interval('sub_telemetry_node_stats', INTERVAL '1 hours');
SELECT add_retention_policy('sub_telemetry_node_stats', INTERVAL '1 days');

--SELECT time_bucket('3 minutes', TNS.time) as "bucket", TNS.node_id, TN.name, avg(TNS.peer_count)
--FROM sub_telemetry_node_stats TNS, sub_telemetry_node TN
--WHERE TNS.time > (now() AT TIME ZONE 'UTC') - INTERVAL '60 minutes'
--AND TNS.node_id = TN.id
--AND TN.name LIKE '%kon%'
--GROUP BY bucket, TNS.node_id, TN.name
--ORDER BY bucket DESC;

--SELECT show_chunks('sub_telemetry_node_stats');

--SELECT * FROM timescaledb_information.job_stats;
--SELECT alter_job(1008, schedule_interval => INTERVAL '2 minutes');
--SELECT drop_chunks('sub_telemetry_node_stats', INTERVAL '3 minutes');