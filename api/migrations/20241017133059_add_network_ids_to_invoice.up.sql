ALTER TABLE Invoice
ADD COLUMN networks integer[] NOT NULL DEFAULT '{}';