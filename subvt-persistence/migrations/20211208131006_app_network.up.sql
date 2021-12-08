CREATE TABLE IF NOT EXISTS app_network
(
    id SERIAL PRIMARY KEY,
    hash VARCHAR(66) NOT NULL,
    name VARCHAR(50) NOT NULL,
    app_service_url VARCHAR(150),
    live_network_status_service_url VARCHAR(150),
    report_service_url VARCHAR(150),
    validator_details_service_url VARCHAR(150),
    validator_list_service_url VARCHAR(150),
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT network_u_hash UNIQUE (hash),
    CONSTRAINT network_u_name UNIQUE (name)
);

INSERT INTO app_network(hash, name) VALUES('0xb0a8d493285c2df73290dfb7e61f870f17b41801197a149ca93654499ea3dafe', 'Kusama');
INSERT INTO app_network(hash, name) VALUES('0x91b171bb158e2d3848fa23a9f1c25182fb8e20313b2c1eb49219da7a70ce90c3', 'Polkadot');