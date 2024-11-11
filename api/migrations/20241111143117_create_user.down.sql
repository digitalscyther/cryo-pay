-- Step 1: Add back the original user_id column to invoice
ALTER TABLE invoice
    ADD COLUMN old_user_id TEXT;

-- Step 2: Populate old_user_id in invoice with the firebase_user_id from user
UPDATE invoice
SET old_user_id = "users".firebase_user_id
FROM "users"
WHERE invoice.user_id = "users".id;

-- Step 3: Remove the new user_id column
ALTER TABLE invoice
    DROP COLUMN user_id;

-- Step 4: Rename old_user_id back to user_id
ALTER TABLE invoice
    RENAME COLUMN old_user_id TO user_id;

-- Step 5: Drop the user table
DROP TABLE IF EXISTS "users";
