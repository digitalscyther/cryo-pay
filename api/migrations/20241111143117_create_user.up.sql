-- Step 1: Create the user table
CREATE TABLE "users" (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    firebase_user_id TEXT NOT NULL UNIQUE,  -- Store Firebase user ID here
    email VARCHAR(255),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- Step 2: Insert unique Firebase user IDs from the invoice table into the user table
INSERT INTO "users" (firebase_user_id)
SELECT DISTINCT user_id FROM invoice WHERE user_id IS NOT NULL;

-- Step 3: Add a new user_id column in invoice to reference user.id
ALTER TABLE invoice
    ADD COLUMN new_user_id UUID REFERENCES "users"(id);

-- Step 4: Update new_user_id in invoice based on the matching Firebase ID in user
UPDATE invoice
SET new_user_id = "users".id
FROM "users"
WHERE invoice.user_id = "users".firebase_user_id;

-- Step 5: Drop the old user_id column from invoice
ALTER TABLE invoice
    DROP COLUMN user_id;

-- Step 6: Rename new_user_id to user_id in invoice
ALTER TABLE invoice
    RENAME COLUMN new_user_id TO user_id;
