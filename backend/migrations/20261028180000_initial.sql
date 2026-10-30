CREATE TABLE contests (
    id         TEXT     PRIMARY KEY NOT NULL,
    name       TEXT     NOT NULL,
    start_time DATETIME,
    end_time   DATETIME NOT NULL
);

CREATE TABLE contest_map (
    id   INTEGER PRIMARY KEY NOT NULL,
    name TEXT    NOT NULL
);

CREATE TABLE contest_map_contest (
    contest_id TEXT    PRIMARY KEY NOT NULL,
    map_id     INTEGER NOT NULL REFERENCES contest_map (id) ON DELETE CASCADE
);

CREATE TABLE stations (
    id             INTEGER PRIMARY KEY NOT NULL,
    ip             TEXT    NOT NULL UNIQUE,
    reported_label TEXT,
    label_override TEXT
);

CREATE TABLE map_element (
    id           BLOB    PRIMARY KEY NOT NULL,
    map_id       INTEGER NOT NULL REFERENCES contest_map (id) ON DELETE CASCADE,
    element_type TEXT    NOT NULL,
    props        TEXT    NOT NULL,
    station_id   INTEGER REFERENCES stations (id) ON DELETE SET NULL
);

CREATE TABLE teams (
    id           TEXT    PRIMARY KEY NOT NULL,
    name         TEXT    NOT NULL,
    team_station INTEGER REFERENCES stations (id) ON DELETE SET NULL
);

CREATE TABLE contest_teams (
    contest_id TEXT NOT NULL REFERENCES contests (id) ON DELETE CASCADE,
    team_id    TEXT NOT NULL REFERENCES teams (id) ON DELETE CASCADE,
    PRIMARY KEY (contest_id, team_id)
);

CREATE TABLE wallpapers (
    id         INTEGER PRIMARY KEY NOT NULL,
    data       BLOB    NOT NULL,
    mime_type  TEXT    NOT NULL,
    text_color TEXT    NOT NULL DEFAULT '#ffffff'
);

CREATE TABLE contest_wallpaper (
    contest_id   TEXT    PRIMARY KEY NOT NULL,
    wallpaper_id INTEGER NOT NULL REFERENCES wallpapers (id) ON DELETE CASCADE
);

CREATE TABLE system_settings (
    id                   INTEGER PRIMARY KEY NOT NULL DEFAULT 1 CHECK (id = 1),
    default_wallpaper_id INTEGER REFERENCES wallpapers (id) ON DELETE SET NULL
);

INSERT INTO system_settings (id) VALUES (1);
