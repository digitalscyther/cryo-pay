CREATE TABLE IF NOT EXISTS network_monitor (
    id SERIAL PRIMARY KEY,
    network VARCHAR(255) NOT NULL UNIQUE,
    block_number BIGINT NOT NULL CHECK (block_number >= 0)
);