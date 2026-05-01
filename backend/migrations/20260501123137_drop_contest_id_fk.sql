-- Add migration script here
ALTER TABLE contest_map_contest
DROP CONSTRAINT contest_map_contest_contest_id_fkey;
