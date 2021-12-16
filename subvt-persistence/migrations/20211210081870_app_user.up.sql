CREATE TABLE IF NOT EXISTS app_user
(
    id                  SERIAL PRIMARY KEY,
    public_key_hex      VARCHAR(66),
    created_at          TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    updated_at          TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT app_user_u_public_key UNIQUE (public_key_hex)
);

CREATE INDEX app_user_idx_public_key_hex
    ON app_user (public_key_hex);