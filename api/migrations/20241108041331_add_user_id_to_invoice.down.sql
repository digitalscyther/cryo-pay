DROP INDEX IF EXISTS idx_invoice_user_id;

ALTER TABLE invoice
DROP COLUMN IF EXISTS user_id;