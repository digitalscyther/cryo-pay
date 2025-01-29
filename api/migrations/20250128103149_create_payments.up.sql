CREATE TABLE payments (
    id UUID PRIMARY KEY,
    data JSONB NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    paid_at TIMESTAMP
);

CREATE INDEX idx_payments_data_category ON payments USING GIN (data);
