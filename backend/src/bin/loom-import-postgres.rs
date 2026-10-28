use std::borrow::Cow;
use std::error::Error;
use std::path::Path;
use std::process::ExitCode;

use chrono::{DateTime, Utc};
use futures::TryStreamExt;
use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use sqlx::{Connection, PgConnection, SqliteConnection};
use uuid::Uuid;

const V0_5_MIGRATION: i64 = 20260913174500;
const V0_6_MIGRATION: i64 = 20261028120000;
const SQLITE_BASELINE_MIGRATION: i64 = 20261028180000;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

fn baseline_migrator() -> Migrator {
    Migrator {
        migrations: Cow::Owned(
            MIGRATOR
                .iter()
                .filter(|migration| migration.version <= SQLITE_BASELINE_MIGRATION)
                .cloned()
                .collect(),
        ),
        ..Migrator::DEFAULT
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let [_, postgres_url, sqlite_path] = args.as_slice() else {
        eprintln!("usage: loom-import-postgres <postgres-url> <sqlite-path>");
        return ExitCode::from(2);
    };

    match import(postgres_url, Path::new(sqlite_path)).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("import failed: {e}");
            ExitCode::FAILURE
        }
    }
}

async fn import(postgres_url: &str, sqlite_path: &Path) -> Result<(), Box<dyn Error>> {
    if sqlite_path.exists() {
        return Err(format!(
            "{} already exists, the import only writes a new database",
            sqlite_path.display()
        )
        .into());
    }

    let mut pg = PgConnection::connect(postgres_url).await?;

    let version: Option<i64> =
        sqlx::query_scalar("SELECT MAX(version) FROM _sqlx_migrations WHERE success")
            .fetch_one(&mut pg)
            .await?;
    let stations_query = match version {
        Some(V0_6_MIGRATION) => "SELECT id, ip, reported_label, label_override FROM stations",
        Some(V0_5_MIGRATION) => "SELECT id, ip, NULL::text, NULL::text FROM stations",
        _ => {
            let version = version.map_or_else(|| "none".to_owned(), |v| v.to_string());
            return Err(format!(
                "postgres schema is at migration {version}, expected {V0_5_MIGRATION} (v0.5.0) \
                 or {V0_6_MIGRATION} (v0.6.0); run loom v0.6.0 against it once before importing"
            )
            .into());
        }
    };

    let result = copy(&mut pg, stations_query, sqlite_path).await;
    if result.is_err() {
        for suffix in ["", "-wal", "-shm"] {
            let mut path = sqlite_path.as_os_str().to_owned();
            path.push(suffix);
            let _ = std::fs::remove_file(path);
        }
    }
    result
}

async fn copy(
    pg: &mut PgConnection,
    stations_query: &str,
    sqlite_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut sqlite = SqliteConnection::connect_with(
        &SqliteConnectOptions::new()
            .filename(sqlite_path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal),
    )
    .await?;
    baseline_migrator().run(&mut sqlite).await?;

    let mut tx = sqlite.begin().await?;

    let mut rows = sqlx::query_as::<_, (String, String, Option<DateTime<Utc>>, DateTime<Utc>)>(
        "SELECT id, name, start_time, end_time FROM contests",
    )
    .fetch(&mut *pg);
    while let Some((id, name, start_time, end_time)) = rows.try_next().await? {
        sqlx::query!(
            "INSERT INTO contests (id, name, start_time, end_time) VALUES ($1, $2, $3, $4)",
            id,
            name,
            start_time,
            end_time
        )
        .execute(&mut *tx)
        .await?;
    }
    drop(rows);

    let mut rows =
        sqlx::query_as::<_, (i32, String)>("SELECT id, name FROM contest_map").fetch(&mut *pg);
    while let Some((id, name)) = rows.try_next().await? {
        sqlx::query!(
            "INSERT INTO contest_map (id, name) VALUES ($1, $2)",
            id,
            name
        )
        .execute(&mut *tx)
        .await?;
    }
    drop(rows);

    let mut rows =
        sqlx::query_as::<_, (i64, String, Option<String>, Option<String>)>(stations_query)
            .fetch(&mut *pg);
    while let Some((id, ip, reported_label, label_override)) = rows.try_next().await? {
        sqlx::query!(
            "INSERT INTO stations (id, ip, reported_label, label_override) VALUES ($1, $2, $3, $4)",
            id,
            ip,
            reported_label,
            label_override
        )
        .execute(&mut *tx)
        .await?;
    }
    drop(rows);

    let mut rows = sqlx::query_as::<_, (String, String, Option<i64>)>(
        "SELECT id, name, team_station FROM teams",
    )
    .fetch(&mut *pg);
    while let Some((id, name, team_station)) = rows.try_next().await? {
        sqlx::query!(
            "INSERT INTO teams (id, name, team_station) VALUES ($1, $2, $3)",
            id,
            name,
            team_station
        )
        .execute(&mut *tx)
        .await?;
    }
    drop(rows);

    let mut rows =
        sqlx::query_as::<_, (String, String)>("SELECT contest_id, team_id FROM contest_teams")
            .fetch(&mut *pg);
    while let Some((contest_id, team_id)) = rows.try_next().await? {
        sqlx::query!(
            "INSERT INTO contest_teams (contest_id, team_id) VALUES ($1, $2)",
            contest_id,
            team_id
        )
        .execute(&mut *tx)
        .await?;
    }
    drop(rows);

    let mut rows =
        sqlx::query_as::<_, (String, i32)>("SELECT contest_id, map_id FROM contest_map_contest")
            .fetch(&mut *pg);
    while let Some((contest_id, map_id)) = rows.try_next().await? {
        sqlx::query!(
            "INSERT INTO contest_map_contest (contest_id, map_id) VALUES ($1, $2)",
            contest_id,
            map_id
        )
        .execute(&mut *tx)
        .await?;
    }
    drop(rows);

    let mut rows = sqlx::query_as::<_, (Uuid, i32, String, serde_json::Value, Option<i64>)>(
        "SELECT id, map_id, element_type, props, station_id FROM map_element",
    )
    .fetch(&mut *pg);
    while let Some((id, map_id, element_type, props, station_id)) = rows.try_next().await? {
        let props = props.to_string();
        sqlx::query!(
            "INSERT INTO map_element (id, map_id, element_type, props, station_id)
             VALUES ($1, $2, $3, $4, $5)",
            id,
            map_id,
            element_type,
            props,
            station_id
        )
        .execute(&mut *tx)
        .await?;
    }
    drop(rows);

    let mut rows = sqlx::query_as::<_, (i32, Vec<u8>, String, String)>(
        "SELECT id, data, mime_type, text_color FROM wallpapers",
    )
    .fetch(&mut *pg);
    while let Some((id, data, mime_type, text_color)) = rows.try_next().await? {
        sqlx::query!(
            "INSERT INTO wallpapers (id, data, mime_type, text_color) VALUES ($1, $2, $3, $4)",
            id,
            data,
            mime_type,
            text_color
        )
        .execute(&mut *tx)
        .await?;
    }
    drop(rows);

    let mut rows = sqlx::query_as::<_, (String, i32)>(
        "SELECT contest_id, wallpaper_id FROM contest_wallpaper",
    )
    .fetch(&mut *pg);
    while let Some((contest_id, wallpaper_id)) = rows.try_next().await? {
        sqlx::query!(
            "INSERT INTO contest_wallpaper (contest_id, wallpaper_id) VALUES ($1, $2)",
            contest_id,
            wallpaper_id
        )
        .execute(&mut *tx)
        .await?;
    }
    drop(rows);

    let default_wallpaper_id: Option<i32> =
        sqlx::query_scalar("SELECT default_wallpaper_id FROM system_settings")
            .fetch_optional(&mut *pg)
            .await?
            .flatten();
    sqlx::query!(
        "UPDATE system_settings SET default_wallpaper_id = $1",
        default_wallpaper_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    MIGRATOR.run(&mut sqlite).await?;
    sqlite.close().await?;

    println!("imported into {}", sqlite_path.display());
    Ok(())
}
