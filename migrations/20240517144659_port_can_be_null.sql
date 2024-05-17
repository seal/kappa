-- Add migration script here
ALTER TABLE "container" ALTER COLUMN port DROP NOT NULL;
