-- Drop the indexes created
DROP INDEX IF EXISTS idx_api_key_api_key;
DROP INDEX IF EXISTS idx_api_key_user_id;

-- Drop the table and its associated constraints
DROP TABLE IF EXISTS api_key;
