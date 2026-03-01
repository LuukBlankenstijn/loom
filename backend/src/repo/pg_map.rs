use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{Door, FullMap, Map, MapRepository, Rotation, Table, Wall};
use crate::error::AppError;

#[derive(sqlx::FromRow)]
struct MapRow {
    id: i64,
    name: String,
}

impl From<MapRow> for Map {
    fn from(r: MapRow) -> Self {
        Self {
            id: r.id as i32,
            name: r.name,
        }
    }
}

pub struct PgMapRepo(PgPool);

impl PgMapRepo {
    pub fn new(pool: PgPool) -> Self {
        Self(pool)
    }
}

#[derive(sqlx::FromRow)]
struct WallRow {
    id: Uuid,
    x_start: i64,
    y_start: i64,
    x_end: i64,
    y_end: i64,
}
#[derive(sqlx::FromRow)]
struct DoorRow {
    id: Uuid,
    x: i64,
    y: i64,
    rotation: String,
}
#[derive(sqlx::FromRow)]
struct TableRow {
    id: Uuid,
    x: i64,
    y: i64,
    rotation: String,
}

impl From<WallRow> for Wall {
    fn from(r: WallRow) -> Self {
        Self {
            id: r.id,
            x_start: r.x_start as i32,
            y_start: r.y_start as i32,
            x_end: r.x_end as i32,
            y_end: r.y_end as i32,
        }
    }
}

impl From<DoorRow> for Door {
    fn from(r: DoorRow) -> Self {
        Self {
            id: r.id,
            x: r.x as i32,
            y: r.y as i32,
            rotation: Rotation::parse(&r.rotation),
        }
    }
}

impl From<TableRow> for Table {
    fn from(r: TableRow) -> Self {
        Self {
            id: r.id,
            x: r.x as i32,
            y: r.y as i32,
            rotation: Rotation::parse(&r.rotation),
        }
    }
}

#[async_trait]
impl MapRepository for PgMapRepo {
    async fn get_all(&self) -> Result<Vec<Map>, AppError> {
        let rows: Vec<MapRow> = sqlx::query_as("SELECT id, name FROM contest_area_maps")
            .fetch_all(&self.0)
            .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn get_map(&self, map_id: i32) -> Result<Option<FullMap>, AppError> {
        let map: Map = match sqlx::query_as::<_, MapRow>(
            "SELECT id, name FROM contest_area_maps WHERE id = $1",
        )
        .bind(map_id as i64)
        .fetch_optional(&self.0)
        .await?
        {
            Some(m) => m.into(),
            None => return Ok(None),
        };

        let (walls, doors, tables) = tokio::try_join!(
            async {
                sqlx::query_as::<_, WallRow>(
                    "SELECT id, x_start, y_start, x_end, y_end FROM wall_elements WHERE contest_area_map_walls = $1",
                )
                .bind(map_id as i64)
                .fetch_all(&self.0)
                .await
                .map_err(AppError::from)
            },
            async {
                sqlx::query_as::<_, DoorRow>(
                    "SELECT id, x, y, rotation FROM door_elements WHERE contest_area_map_doors = $1",
                )
                .bind(map_id as i64)
                .fetch_all(&self.0)
                .await
                .map_err(AppError::from)
            },
            async {
                sqlx::query_as::<_, TableRow>(
                    "SELECT id, x, y, rotation FROM table_elements WHERE contest_area_map_tables = $1",
                )
                .bind(map_id as i64)
                .fetch_all(&self.0)
                .await
                .map_err(AppError::from)
            },
        )?;

        Ok(Some(FullMap {
            id: map.id,
            name: map.name,
            walls: walls.into_iter().map(Into::into).collect(),
            doors: doors.into_iter().map(Into::into).collect(),
            tables: tables.into_iter().map(Into::into).collect(),
        }))
    }

    async fn get_by_contest(&self, contest_id: &str) -> Result<Option<Map>, AppError> {
        let map_id: Option<(i64,)> =
            sqlx::query_as("SELECT map_id FROM contest_maps WHERE contest_id = $1")
                .bind(contest_id)
                .fetch_optional(&self.0)
                .await?;

        match map_id {
            Some((id,)) => {
                let row = sqlx::query_as::<_, MapRow>(
                    "SELECT id, name FROM contest_area_maps WHERE id = $1",
                )
                .bind(id)
                .fetch_optional(&self.0)
                .await?;
                Ok(row.map(Into::into))
            }
            None => Ok(None),
        }
    }

    async fn set_map(&self, map_id: i32, contest_id: &str) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO contest_maps (contest_id, map_id) VALUES ($1, $2)
             ON CONFLICT (contest_id) DO UPDATE SET map_id = $2",
        )
        .bind(contest_id)
        .bind(map_id as i64)
        .execute(&self.0)
        .await?;
        Ok(())
    }

    async fn create_map(&self, name: &str) -> Result<i32, AppError> {
        let (id,): (i64,) =
            sqlx::query_as("INSERT INTO contest_area_maps (name) VALUES ($1) RETURNING id")
                .bind(name)
                .fetch_one(&self.0)
                .await?;
        Ok(id as i32)
    }

    async fn delete_elements(&self, ids: &[Uuid]) -> Result<(), AppError> {
        let mut tx = self.0.begin().await?;
        sqlx::query("DELETE FROM wall_elements WHERE id = ANY($1)")
            .bind(ids)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM door_elements WHERE id = ANY($1)")
            .bind(ids)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM table_elements WHERE id = ANY($1)")
            .bind(ids)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    async fn upsert_elements(
        &self,
        map_id: i32,
        walls: &[Wall],
        doors: &[Door],
        tables: &[Table],
    ) -> Result<(), AppError> {
        let mut tx = self.0.begin().await?;
        let map_id = map_id as i64;

        for w in walls {
            sqlx::query(
                "INSERT INTO wall_elements (id, x_start, y_start, x_end, y_end, contest_area_map_walls)
                 VALUES ($1, $2, $3, $4, $5, $6)
                 ON CONFLICT (id) DO UPDATE
                 SET x_start = $2, y_start = $3, x_end = $4, y_end = $5, contest_area_map_walls = $6",
            )
            .bind(w.id).bind(w.x_start as i64).bind(w.y_start as i64)
            .bind(w.x_end as i64).bind(w.y_end as i64).bind(map_id)
            .execute(&mut *tx).await?;
        }
        for d in doors {
            sqlx::query(
                "INSERT INTO door_elements (id, x, y, rotation, contest_area_map_doors)
                 VALUES ($1, $2, $3, $4, $5)
                 ON CONFLICT (id) DO UPDATE SET x = $2, y = $3, rotation = $4, contest_area_map_doors = $5",
            )
            .bind(d.id).bind(d.x as i64).bind(d.y as i64)
            .bind(d.rotation.as_str()).bind(map_id)
            .execute(&mut *tx).await?;
        }
        for t in tables {
            sqlx::query(
                "INSERT INTO table_elements (id, x, y, rotation, contest_area_map_tables)
                 VALUES ($1, $2, $3, $4, $5)
                 ON CONFLICT (id) DO UPDATE SET x = $2, y = $3, rotation = $4, contest_area_map_tables = $5",
            )
            .bind(t.id).bind(t.x as i64).bind(t.y as i64)
            .bind(t.rotation.as_str()).bind(map_id)
            .execute(&mut *tx).await?;
        }

        tx.commit().await?;
        Ok(())
    }
}
