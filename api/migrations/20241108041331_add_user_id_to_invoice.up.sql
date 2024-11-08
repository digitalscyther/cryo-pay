ALTER TABLE invoice
ADD COLUMN user_id VARCHAR(255);

CREATE INDEX idx_invoice_user_id ON invoice(user_id);