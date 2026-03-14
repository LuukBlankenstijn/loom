-- Contests
CREATE TABLE contests (
    id         TEXT        PRIMARY KEY,
    name       TEXT        NOT NULL,
    start_time TIMESTAMPTZ NOT NULL,
    end_time   TIMESTAMPTZ NOT NULL
);

-- Maps
CREATE TABLE contest_map (
    id   SERIAL PRIMARY KEY,
    name TEXT   NOT NULL
);

-- Contest → Map assignment (one active map per contest)
CREATE TABLE contest_map_contest (
    contest_id TEXT    PRIMARY KEY REFERENCES contests(id)    ON DELETE CASCADE,
    map_id     INTEGER NOT NULL    REFERENCES contest_map(id) ON DELETE CASCADE
);

-- Map elements (walls, doors, seats)
CREATE TABLE map_element (
    id           UUID    PRIMARY KEY,
    map_id       INTEGER NOT NULL REFERENCES contest_map(id) ON DELETE CASCADE,
    element_type TEXT    NOT NULL,
    props        JSONB   NOT NULL
);

-- Stations (physical contest machines, keyed by IP)
CREATE TABLE stations (
    id BIGSERIAL PRIMARY KEY,
    ip TEXT      NOT NULL UNIQUE
);

-- Teams (local mode; in ICPC mode teams come from the API)
CREATE TABLE teams (
    id            TEXT   PRIMARY KEY,
    name          TEXT   NOT NULL,
    team_station  BIGINT REFERENCES stations(id) ON DELETE SET NULL
);

-- Contest → Team membership
CREATE TABLE contest_teams (
    contest_id TEXT NOT NULL REFERENCES contests(id) ON DELETE CASCADE,
    team_id    TEXT NOT NULL REFERENCES teams(id)    ON DELETE CASCADE,
    PRIMARY KEY (contest_id, team_id)
);

-- Wallpapers (one per contest)
CREATE TABLE wallpapers (
    contest_id TEXT        PRIMARY KEY REFERENCES contests(id) ON DELETE CASCADE,
    data       BYTEA       NOT NULL,
    mime_type  TEXT        NOT NULL,
    text_color TEXT        NOT NULL DEFAULT '#ffffff'
);
