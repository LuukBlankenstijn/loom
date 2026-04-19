-- Add migration script here
ALTER TABLE contests ALTER COLUMN start_time DROP NOT NULL;
