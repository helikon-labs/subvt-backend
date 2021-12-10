CREATE TABLE IF NOT EXISTS app_user
(
    id                  SERIAL PRIMARY KEY,
    public_key_hex      text NOT NULL,
    created_at          TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    updated_at          TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT app_user_u_public_key UNIQUE (public_key_hex)
);