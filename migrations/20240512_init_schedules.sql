CREATE TABLE IF NOT EXISTS schedules
(
    id              BIGSERIAL PRIMARY KEY,
    client_id       BIGINT NOT NULL,
    filename        TEXT NOT NULL,
    vendor          TEXT NOT NULL,
    target          TEXT NOT NULL,
    cron_expression TEXT NOT NULL,
    next_run        TIMESTAMP WITH TIME ZONE NOT NULL
);
