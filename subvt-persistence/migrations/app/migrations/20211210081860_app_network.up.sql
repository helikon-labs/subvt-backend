CREATE TABLE IF NOT EXISTS app_network
(
    id                                  integer PRIMARY KEY,
    hash                                VARCHAR(66) NOT NULL,
    name                                VARCHAR(50) NOT NULL,
    ss58_prefix                         integer NOT NULL,
    network_status_service_url          VARCHAR(150),
    report_service_url                  VARCHAR(150),
    validator_details_service_url       VARCHAR(150),
    active_validator_list_service_url   VARCHAR(150),
    inactive_validator_list_service_url VARCHAR(150),
    created_at                          TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    updated_at                          TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT app_network_u_hash UNIQUE (hash),
    CONSTRAINT app_network_u_name UNIQUE (name)
);

INSERT INTO app_network(id, hash, name, ss58_prefix) VALUES(1, '0xB0A8D493285C2DF73290DFB7E61F870F17B41801197A149CA93654499EA3DAFE', 'Kusama', 2);
INSERT INTO app_network(id, hash, name, ss58_prefix) VALUES(2, '0x91B171BB158E2D3848FA23A9F1C25182FB8E20313B2C1EB49219DA7A70CE90C3', 'Polkadot', 0);
INSERT INTO app_network(id, hash, name, ss58_prefix) VALUES(3, '0x401A1F9DCA3DA46F5C4091016C8A2F26DCEA05865116B286F60F668207D1474B', 'Moonriver', 1285);