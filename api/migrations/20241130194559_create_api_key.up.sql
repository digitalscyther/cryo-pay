CREATE TABLE api_key (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    api_key VARCHAR(255) NOT NULL,
    created TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    last_used TIMESTAMP,
    CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Create an index on the user_id column for better performance on lookups.
CREATE INDEX idx_api_key_user_id ON api_key (user_id);

-- Create a unique index on the api_key column to ensure quick lookups and uniqueness.
CREATE UNIQUE INDEX idx_api_key_api_key ON api_key (api_key);
