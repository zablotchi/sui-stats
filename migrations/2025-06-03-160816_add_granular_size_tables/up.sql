-- Your SQL goes here

CREATE TABLE IF NOT EXISTS transaction_sizes (
    tx_digest               TEXT NOT NULL,
    cp_sequence_number      BIGINT NOT NULL,
    tx_size_bytes          BIGINT NOT NULL,
    PRIMARY KEY (tx_digest, cp_sequence_number)
);

CREATE TABLE IF NOT EXISTS object_sizes (
    object_id              TEXT NOT NULL,
    cp_sequence_number     BIGINT NOT NULL,
    object_size_bytes      BIGINT NOT NULL,
    is_input              BOOLEAN NOT NULL,  -- true if input object, false if output object
    PRIMARY KEY (object_id, cp_sequence_number, is_input)
);

CREATE TABLE IF NOT EXISTS effect_sizes (
    tx_digest              TEXT NOT NULL,
    cp_sequence_number     BIGINT NOT NULL,
    effect_size_bytes      BIGINT NOT NULL,
    PRIMARY KEY (tx_digest, cp_sequence_number)
);

-- Create indexes for common query patterns
CREATE INDEX idx_transaction_sizes_cp ON transaction_sizes (cp_sequence_number);
CREATE INDEX idx_object_sizes_cp ON object_sizes (cp_sequence_number);
CREATE INDEX idx_effect_sizes_cp ON effect_sizes (cp_sequence_number);
