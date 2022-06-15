CREATE TABLE IF NOT EXISTS app_network
(
    id                                      INTEGER PRIMARY KEY,
    hash                                    VARCHAR(66) NOT NULL,
    chain                                   VARCHAR(50) NOT NULL,
    display                                 VARCHAR(50) NOT NULL,
    ss58_prefix                             INTEGER NOT NULL,
    token_ticker                            VARCHAR(16) NOT NULL,
    token_decimal_count                     INTEGER NOT NULL,
    network_status_service_host             VARCHAR(150),
    network_status_service_port             INTEGER,
    report_service_host                     VARCHAR(150),
    report_service_port                     INTEGER,
    validator_details_service_host          VARCHAR(150),
    validator_details_service_port          INTEGER,
    active_validator_list_service_host      VARCHAR(150),
    active_validator_list_service_port      INTEGER,
    inactive_validator_list_service_host    VARCHAR(150),
    inactive_validator_list_service_port    INTEGER,
    created_at                              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    updated_at                              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT app_network_u_hash UNIQUE (hash),
    CONSTRAINT app_network_u_chain UNIQUE (chain),
    CONSTRAINT app_network_u_display UNIQUE (display)
);

INSERT INTO app_network(id, hash, chain, display, ss58_prefix, token_ticker, token_decimal_count)
VALUES(1, '0xB0A8D493285C2DF73290DFB7E61F870F17B41801197A149CA93654499EA3DAFE', 'kusama', 'Kusama', 2, 'KSM', 12)
ON CONFLICT(id) DO NOTHING;
INSERT INTO app_network(id, hash, chain, display, ss58_prefix, token_ticker, token_decimal_count)
VALUES(2, '0x91B171BB158E2D3848FA23A9F1C25182FB8E20313B2C1EB49219DA7A70CE90C3', 'polkadot', 'Polkadot', 0, 'DOT', 10)
ON CONFLICT(id) DO NOTHING;
--INSERT INTO app_network(id, hash, chain, display, ss58_prefix, token_ticker, token_decimal_count) VALUES(3, '0xE143F23803AC50E8F6F8E62695D1CE9E4E1D68AA36C1CD2CFD15340213F3423E', 'westend', 'Westend', 42, 'WND', 12) ON CONFLICT(id) DO NOTHING;