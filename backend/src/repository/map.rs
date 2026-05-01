use async_trait::async_trait;
use loom_core::map::door::Door;
use loom_core::map::seat::Seat;
use loom_core::map::wall::Wall;
use loom_core::map::{Map, MapElement, MapMetadata, Point, Rotation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::error;
use uuid::Uuid;

use crate::domain::MapRepository;
use crate::error::AppError;

pub struct MapRepo(PgPool);

impl MapRepo {
    pub fn new(pool: PgPool) -> Self {
        Self(pool)
    }
}

// Intermediate helper structs for JSON conversion
#[derive(Serialize, Deserialize)]
struct WallProps {
    x_start: f32,
    y_start: f32,
    x_end: f32,
    y_end: f32,
}
#[derive(Serialize, Deserialize)]
struct PointProps {
    x: f32,
    y: f32,
    rotation: String,
}

#[async_trait]
impl MapRepository for MapRepo {
    async fn get(&self, map_id: i32) -> Result<Option<Map>, AppError> {
        let map_row = sqlx::query!("SELECT id, name FROM contest_map WHERE id = $1", map_id)
            .fetch_optional(&self.0)
            .await?;

        let (id, name) = match map_row {
            Some(row) => (row.id, row.name),
            None => return Ok(None),
        };

        let element_rows = sqlx::query!(
            "
            SELECT me.id, me.element_type, s.ip as \"ip?\", me.props FROM map_element as me 
            LEFT JOIN stations as s ON me.station_id = s.id 
            WHERE me.map_id = $1
            ",
            map_id
        )
        .fetch_all(&self.0)
        .await?;

        let mut elements = Vec::with_capacity(element_rows.len());
        for row in element_rows {
            let id: Uuid = row.id;
            let props = row.props;

            let element = match row.element_type.as_str() {
                "Wall" => {
                    let p: WallProps = serde_json::from_value(props).map_err(|e| {
                        error!("{e}");
                        AppError::Internal("".to_string())
                    })?;
                    MapElement::Wall(Wall {
                        id,
                        start: Point::new(p.x_start, p.x_end),
                        end: Point::new(p.y_start, p.y_end),
                    })
                }
                "Door" => {
                    let p: PointProps = serde_json::from_value(props).map_err(|e| {
                        error!("{e}");
                        AppError::Internal("".to_string())
                    })?;
                    MapElement::Door(Door {
                        id,
                        position: Point::new(p.x, p.y),
                        rotation: Rotation::parse(&p.rotation),
                    })
                }
                "Seat" => {
                    let p: PointProps = serde_json::from_value(props).map_err(|e| {
                        error!("{e}");
                        AppError::Internal("".to_string())
                    })?;
                    MapElement::Seat(Seat {
                        id,
                        position: Point::new(p.x, p.y),
                        rotation: Rotation::parse(&p.rotation),
                        ip: row.ip,
                    })
                }
                _ => {
                    return Err(AppError::Internal(format!(
                        "Unknown element type: {}",
                        row.element_type
                    )));
                }
            };
            elements.push(element);
        }

        Ok(Some(Map { id, name, elements }))
    }

    async fn upsert_elements(
        &self,
        map_id: i32,
        elements: Vec<MapElement>,
    ) -> Result<(), AppError> {
        let mut tx = self.0.begin().await?;

        for el in elements {
            let (id, el_type, props) = match el {
                MapElement::Wall(w) => (
                    w.id,
                    "Wall",
                    serde_json::to_value(WallProps {
                        x_start: w.start.x,
                        y_start: w.start.y,
                        x_end: w.end.x,
                        y_end: w.end.y,
                    })
                    .map_err(|e| {
                        error!("{e}");
                        AppError::Internal("".to_string())
                    })?,
                ),
                MapElement::Door(d) => (
                    d.id,
                    "Door",
                    serde_json::to_value(PointProps {
                        x: d.position.x,
                        y: d.position.y,
                        rotation: d.rotation.as_str().to_string(),
                    })
                    .map_err(|e| {
                        error!("{e}");
                        AppError::Internal("".to_string())
                    })?,
                ),
                MapElement::Seat(s) => (
                    s.id,
                    "Seat",
                    serde_json::to_value(PointProps {
                        x: s.position.x,
                        y: s.position.y,
                        rotation: s.rotation.as_str().to_string(),
                    })
                    .map_err(|e| {
                        error!("{e}");
                        AppError::Internal("".to_string())
                    })?,
                ),
            };

            sqlx::query!(
                "INSERT INTO map_element (id, map_id, element_type, props)
                 VALUES ($1, $2, $3, $4)
                 ON CONFLICT (id) DO UPDATE SET
                    element_type = EXCLUDED.element_type,
                    props = EXCLUDED.props,
                    map_id = EXCLUDED.map_id",
                id,
                map_id,
                el_type,
                props
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn delete_elements(&self, element_ids: &[Uuid]) -> Result<(), AppError> {
        sqlx::query!(
            "DELETE FROM map_element WHERE id = ANY($1)",
            element_ids as &[Uuid]
        )
        .execute(&self.0)
        .await?;
        Ok(())
    }

    async fn get_all_metadata(&self) -> Result<Vec<MapMetadata>, AppError> {
        let rows = sqlx::query!("SELECT id, name FROM contest_map")
            .fetch_all(&self.0)
            .await?;

        Ok(rows
            .into_iter()
            .map(|r| MapMetadata {
                id: r.id,
                name: r.name,
            })
            .collect())
    }

    async fn get_by_contest(&self, contest_id: &str) -> Result<Option<Map>, AppError> {
        let row = sqlx::query!(
            "SELECT m.id
             FROM contest_map m
             JOIN contest_map_contest cmc ON cmc.map_id = m.id
             WHERE cmc.contest_id = $1",
            contest_id
        )
        .fetch_optional(&self.0)
        .await?;

        match row {
            Some(r) => self.get(r.id).await,
            None => Ok(None),
        }
    }

    async fn create_map(&self, name: &str) -> Result<Map, AppError> {
        let row = sqlx::query!(
            "INSERT INTO contest_map (name) VALUES ($1) RETURNING id, name",
            name
        )
        .fetch_one(&self.0)
        .await?;

        Ok(Map {
            id: row.id,
            name: row.name,
            elements: vec![],
        })
    }

    async fn assign_station_to_seat(
        &self,
        seat_id: Uuid,
        station_ip: Option<String>,
    ) -> Result<Option<Uuid>, AppError> {
        // check seat
        let Some(seat) = sqlx::query!(
            "SELECT * FROM map_element WHERE id = $1 AND element_type = \'Seat\'",
            seat_id
        )
        .fetch_optional(&self.0)
        .await?
        else {
            return Err(AppError::NotFound("seat not found".to_string()));
        };

        if let Some(station_ip) = station_ip {
            let mut tx = self.0.begin().await?;
            let station = sqlx::query!(
                r#"
                    SELECT * FROM stations
                    WHERE ip = $1
                "#,
                station_ip
            )
            .fetch_one(&mut *tx)
            .await?;

            let old = sqlx::query!(
                r#"
                UPDATE map_element
                SET station_id = NULL
                WHERE map_id = $1 AND station_id = $2
                RETURNING *
                "#,
                seat.map_id,
                station.id,
            )
            .fetch_optional(&mut *tx)
            .await?;

            sqlx::query!(
                r#"
                UPDATE map_element
                SET station_id = $1
                WHERE id = $2
                "#,
                station.id,
                seat.id
            )
            .execute(&mut *tx)
            .await?;
            tx.commit().await?;
            if let Some(old) = old {
                Ok(Some(old.id))
            } else {
                Ok(None)
            }
        } else {
            // remove current assignment
            sqlx::query!(
                r#"
                    UPDATE map_element SET station_id = NULL WHERE id = $1
                "#,
                seat.id
            )
            .execute(&self.0)
            .await?;

            Ok(Some(seat.id))
        }
    }

    async fn get_all_station_assignments(&self) -> Result<Vec<(Uuid, Option<String>)>, AppError> {
        let rows = sqlx::query!(
            "
            SELECT e.id, s.ip FROM map_element as e
            INNER JOIN stations AS s ON s.id = e.station_id
            ",
        )
        .fetch_all(&self.0)
        .await?;

        Ok(rows.into_iter().map(|r| (r.id, Some(r.ip))).collect())
    }

    async fn get_seat_id_by_ip(&self, ip: &str) -> Result<Option<Uuid>, AppError> {
        let row = sqlx::query!(
            "
            SELECT e.id FROM map_element as e
            INNER JOIN stations AS s ON s.id = e.station_id
            WHERE s.ip = $1 AND e.element_type = 'Seat'
            ",
            ip
        )
        .fetch_optional(&self.0)
        .await?;

        Ok(row.map(|r| r.id))
    }
}
