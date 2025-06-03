CREATE TABLE IF NOT EXISTS sizes (
    cp_sequence_number          BIGINT PRIMARY KEY,
    cp_summary_bytes            BIGINT NOT NULL,
    cp_signatures_bytes         BIGINT NOT NULL,
    cp_contents_bytes           BIGINT NOT NULL,
    tx_count                    BIGINT NOT NULL,
    tx_bytes                    BIGINT NOT NULL,
    fx_bytes                    BIGINT NOT NULL,
    ev_bytes                    BIGINT NOT NULL,
    obj_count                   BIGINT NOT NULL,
    obj_bytes                   BIGINT NOT NULL
);
