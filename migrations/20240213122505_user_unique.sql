-- Add migration script here
ALTER TABLE "user"
ADD CONSTRAINT unique_username UNIQUE (username);
