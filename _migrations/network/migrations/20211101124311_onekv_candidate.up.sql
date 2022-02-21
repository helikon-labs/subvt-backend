CREATE TABLE IF NOT EXISTS sub_onekv_candidate
(
    id                      SERIAL PRIMARY KEY,
    onekv_id                VARCHAR(128) NOT NULL,
    validator_account_id    VARCHAR(66) NOT NULL,
    kusama_account_id       VARCHAR(66),
    discovered_at           bigint NOT NULL,
    inclusion               real NOT NULL,
    last_valid              bigint,
    nominated_at            bigint,
    offline_accumulated     bigint NOT NULL,
    offline_since           bigint NOT NULL,
    online_since            bigint NOT NULL,
    name                    text NOT NULL,
    location                text,
    rank                    bigint,
    version                 VARCHAR(256),
    is_valid                boolean,
    score_updated_at        bigint,
    score_total             double precision,
    score_aggregate         double precision,
    score_inclusion         double precision,
    score_discovered        double precision,
    score_nominated         double precision,
    score_rank              double precision,
    score_unclaimed         double precision,
    score_bonded            double precision,
    score_faults            double precision,
    score_offline           double precision,
    score_randomness        double precision,
    score_span_inclusion    double precision,
    score_location          double precision,
    score_council_stake     double precision,
    score_democracy         double precision,
    created_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);

ALTER TABLE sub_onekv_candidate
    ADD CONSTRAINT sub_onekv_candidate_fk_validator_account_id
    FOREIGN KEY (validator_account_id)
        REFERENCES sub_account (id)
        ON DELETE RESTRICT
        ON UPDATE CASCADE;
