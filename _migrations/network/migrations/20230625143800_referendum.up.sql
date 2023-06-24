CREATE TABLE IF NOT EXISTS sub_referendum
(
    post_id             INTEGER PRIMARY KEY,
    proposer_account_id VARCHAR(66) NOT NULL,
    type                TEXT NOT NULL,
    track_id            SMALLINT NOT NULL,
    title               TEXT,
    method              TEXT,
    status              TEXT NOT NULL,
    pa_created_at       TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    created_at          TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    updated_at          TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS sub_referendum_idx_proposer_account_id
    ON sub_referendum (proposer_account_id);