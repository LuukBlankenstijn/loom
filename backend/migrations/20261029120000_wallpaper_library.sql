CREATE TABLE wallpaper_library (
    id         INTEGER PRIMARY KEY NOT NULL,
    name       TEXT    NOT NULL,
    kind       TEXT    NOT NULL CHECK (kind IN ('image', 'youtube')),
    data       BLOB,
    mime_type  TEXT,
    url        TEXT,
    text_color TEXT    NOT NULL DEFAULT '#ffffff',
    CHECK (
        (data IS NOT NULL AND mime_type IS NOT NULL AND url IS NULL)
        OR (data IS NULL AND mime_type IS NULL AND url IS NOT NULL)
    ),
    CHECK (kind <> 'youtube' OR url IS NOT NULL)
);

INSERT INTO wallpaper_library (id, name, kind, data, mime_type, text_color)
SELECT w.id,
       COALESCE(
           (SELECT COALESCE(c.name, cw.contest_id)
              FROM contest_wallpaper cw
              LEFT JOIN contests c ON c.id = cw.contest_id
             WHERE cw.wallpaper_id = w.id
             ORDER BY c.start_time IS NULL, c.start_time DESC
             LIMIT 1),
           CASE WHEN EXISTS (SELECT 1 FROM system_settings s WHERE s.default_wallpaper_id = w.id)
                THEN 'Default' END,
           'Wallpaper ' || w.id),
       'image', w.data, w.mime_type, w.text_color
  FROM wallpapers w;

CREATE TEMP TABLE kept_contest_wallpaper AS SELECT contest_id, wallpaper_id FROM contest_wallpaper;
CREATE TEMP TABLE kept_default_wallpaper AS SELECT default_wallpaper_id FROM system_settings;

DROP TABLE wallpapers;
ALTER TABLE wallpaper_library RENAME TO wallpapers;

INSERT OR REPLACE INTO contest_wallpaper (contest_id, wallpaper_id)
SELECT contest_id, wallpaper_id FROM kept_contest_wallpaper;
UPDATE system_settings SET default_wallpaper_id = (SELECT default_wallpaper_id FROM kept_default_wallpaper);

DROP TABLE kept_contest_wallpaper;
DROP TABLE kept_default_wallpaper;
