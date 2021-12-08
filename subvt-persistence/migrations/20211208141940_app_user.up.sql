CREATE TABLE IF NOT EXISTS app_user
(
    id SERIAL PRIMARY KEY,
    public_key text NOT NULL,
    apns_token VARCHAR(200),
    firebase_token VARCHAR(200),
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT user_u_public_key UNIQUE (public_key),
    CONSTRAINT user_u_apns_token UNIQUE (apns_token),
    CONSTRAINT user_u_firebase_token UNIQUE (firebase_token)
);