-- Add migration script here
ALTER TABLE map_element 
ADD COLUMN station_id bigint 
REFERENCES stations (id) 
ON DELETE SET NULL;
