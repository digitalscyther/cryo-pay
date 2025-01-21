-- Drop the indexes created
DROP INDEX IF EXISTS idx_callback_urls_user_id;

-- Drop the table and its associated constraints
DROP TABLE IF EXISTS callback_urls;
