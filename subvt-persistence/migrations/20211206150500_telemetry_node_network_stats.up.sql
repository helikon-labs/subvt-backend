CREATE TABLE IF NOT EXISTS telemetry_node_network_stats (
   time TIMESTAMP WITHOUT TIME ZONE     NOT NULL DEFAULT now(),
   node_id bigint                       NOT NULL,
   download_bandwidth double PRECISION  NOT NULL,
   upload_bandwidth double PRECISION    NOT NULL,
   CONSTRAINT telemetry_node_network_stats_u_time_node_id
              UNIQUE (time, node_id),
   CONSTRAINT telemetry_node_network_stats_fk_telemetry_node
              FOREIGN KEY (node_id)
                  REFERENCES telemetry_node (id)
                  ON DELETE CASCADE
                  ON UPDATE CASCADE
);

SELECT create_hypertable('telemetry_node_network_stats', 'time');
SELECT set_chunk_time_interval('telemetry_node_network_stats', INTERVAL '1 hours');
SELECT add_retention_policy('telemetry_node_network_stats', INTERVAL '1 days');

--SELECT time_bucket('10 seconds', TNNS.time) as "bucket", TNNS.node_id, TN.name, avg(TNNS.download_bandwidth)
--FROM telemetry_node_network_stats TNNS, telemetry_node TN
--WHERE TNNS.time > (now() AT TIME ZONE 'UTC') - INTERVAL '10 minutes'
--AND TNNS.node_id = TN.id
--AND TN.name LIKE '%kon%'
--GROUP BY bucket, TNNS.node_id, TN.name
--ORDER BY bucket DESC;