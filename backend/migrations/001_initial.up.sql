-- Schema matching existing Ent-managed database.
-- Table and column names follow Ent's naming conventions exactly.

CREATE TABLE IF NOT EXISTS stations (
    id BIGSERIAL PRIMARY KEY,
    ip VARCHAR NOT NULL UNIQUE,
    connected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    disconnected_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS contests (
    id VARCHAR PRIMARY KEY,
    name VARCHAR NOT NULL,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS teams (
    id VARCHAR PRIMARY KEY,
    name VARCHAR NOT NULL,
    -- Ent names this FK column after the edge: team -> station = team_station
    team_station BIGINT UNIQUE REFERENCES stations(id) ON DELETE SET NULL
);

-- Ent M2M join table for contest <-> team
CREATE TABLE IF NOT EXISTS contest_teams (
    contest_id VARCHAR NOT NULL REFERENCES contests(id) ON DELETE CASCADE,
    team_id VARCHAR NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    PRIMARY KEY (contest_id, team_id)
);

CREATE TABLE IF NOT EXISTS wallpapers (
    id BIGSERIAL PRIMARY KEY,
    image_data BYTEA NOT NULL,
    mime_type VARCHAR NOT NULL,
    color VARCHAR NOT NULL DEFAULT '#ffffff',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    contest_id VARCHAR NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS contest_area_maps (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR NOT NULL
);

-- Ent stores contest->map association in a separate table
CREATE TABLE IF NOT EXISTS contest_maps (
    id BIGSERIAL PRIMARY KEY,
    contest_id VARCHAR NOT NULL UNIQUE,
    map_id BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS wall_elements (
    id UUID PRIMARY KEY,
    x_start BIGINT NOT NULL,
    y_start BIGINT NOT NULL,
    x_end BIGINT NOT NULL,
    y_end BIGINT NOT NULL,
    -- Ent FK: contest_area_map edge "walls" -> contest_area_map_walls
    contest_area_map_walls BIGINT NOT NULL REFERENCES contest_area_maps(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS door_elements (
    id UUID PRIMARY KEY,
    x BIGINT NOT NULL,
    y BIGINT NOT NULL,
    rotation VARCHAR NOT NULL DEFAULT '0',
    -- Ent FK: contest_area_map edge "doors" -> contest_area_map_doors
    contest_area_map_doors BIGINT NOT NULL REFERENCES contest_area_maps(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS table_elements (
    id UUID PRIMARY KEY,
    x BIGINT NOT NULL,
    y BIGINT NOT NULL,
    rotation VARCHAR NOT NULL DEFAULT '0',
    -- Ent FK: contest_area_map edge "tables" -> contest_area_map_tables
    contest_area_map_tables BIGINT NOT NULL REFERENCES contest_area_maps(id) ON DELETE CASCADE,
    -- Ent FK: table_element edge "station" -> table_element_station
    table_element_station BIGINT UNIQUE REFERENCES stations(id) ON DELETE SET NULL
);
