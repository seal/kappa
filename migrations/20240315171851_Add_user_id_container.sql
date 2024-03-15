-- Add migration script here
ALTER TABLE "container"
ADD COLUMN user_id UUID REFERENCES "user"(user_id);
