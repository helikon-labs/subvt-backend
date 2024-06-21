CREATE TABLE IF NOT EXISTS sub_kline_historical
(
    id                      SERIAL PRIMARY KEY,
    open_time               BIGINT NOT NULL,
    source_ticker           VARCHAR(16) NOT NULL,
    target_ticker           VARCHAR(16) NOT NULL,
    "open"                  NUMERIC NOT NULL,
    high                    NUMERIC NOT NULL,
    low                     NUMERIC NOT NULL,
    "close"                 NUMERIC NOT NULL,
    volume                  NUMERIC NOT NULL,
    close_time              BIGINT NOT NULL,
    quote_volume            NUMERIC NOT NULL,
    "count"                 INTEGER NOT NULL,
    taker_buy_volume        NUMERIC NOT NULL,
    taker_buy_quote_volume  NUMERIC NOT NULL,
    created_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT sub_kline_historical_u_open_time_source_ticker_target_ticker
    UNIQUE (open_time, source_ticker, target_ticker)
);

CREATE INDEX IF NOT EXISTS sub_event_referendum_timed_out_idx_open_time_source_ticker_target_ticker
    ON sub_kline_historical (open_time, source_ticker, target_ticker);