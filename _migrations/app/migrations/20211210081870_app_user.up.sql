CREATE TABLE IF NOT EXISTS app_user
(
    id              SERIAL PRIMARY KEY,
    public_key_hex  VARCHAR(68),
    registration_ip VARCHAR(64),
    created_at      TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    updated_at      TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMP WITHOUT TIME ZONE,
    CONSTRAINT app_user_u_public_key UNIQUE (public_key_hex)
);

CREATE INDEX IF NOT EXISTS app_user_idx_public_key_hex
    ON app_user (public_key_hex);

CREATE INDEX IF NOT EXISTS app_user_idx_registration_ip_created_at
    ON app_user (registration_ip, created_at);