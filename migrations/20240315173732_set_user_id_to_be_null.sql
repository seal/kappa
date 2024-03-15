-- Add migration script here
ALTER TABLE "container"
ALTER COLUMN user_id SET NOT NULL;
