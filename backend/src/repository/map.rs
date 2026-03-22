use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::error;
use uuid::Uuid;

use crate::domain::{
    Door, Map, MapElement, MapMetadata, MapRepository, Rotation, Seat, StationAssignment, Wall,
};
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
    x_start: i32,
    y_start: i32,
    x_end: i32,
    y_end: i32,
}
#[derive(Serialize, Deserialize)]
struct PointProps {
    x: i32,
    y: i32,
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
            "SELECT id, element_type, props FROM map_element WHERE map_id = $1",
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
                        x_start: p.x_start,
                        y_start: p.y_start,
                        x_end: p.x_end,
                        y_end: p.y_end,
                    })
                }
                "Door" => {
                    let p: PointProps = serde_json::from_value(props).map_err(|e| {
                        error!("{e}");
                        AppError::Internal("".to_string())
                    })?;
                    MapElement::Door(Door {
                        id,
                        x: p.x,
                        y: p.y,
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
                        x: p.x,
                        y: p.y,
                        rotation: Rotation::parse(&p.rotation),
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
                        x_start: w.x_start,
                        y_start: w.y_start,
                        x_end: w.x_end,
                        y_end: w.y_end,
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
                        x: d.x,
                        y: d.y,
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
                        x: s.x,
                        y: s.y,
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
        station_ip: String,
        seat_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        // check if station exists
        let station = sqlx::query!("SELECT * FROM stations WHERE ip = ($1)", station_ip)
            .fetch_one(&self.0)
            .await?;
        if let Some(seat_id) = seat_id {
            // check if the elment exists and is of the correct type
            let element = sqlx::query!("SELECT * FROM map_element WHERE id = ($1)", seat_id)
                .fetch_one(&self.0)
                .await?;
            if element.element_type != "Seat" {
                return Err(AppError::InvalidArgument(
                    "Element is not of type seat".to_string(),
                ));
            }
            // assign station
            sqlx::query!(
                "UPDATE map_element SET station_id = ($2) WHERE id = ($1)",
                element.id,
                station.id
            )
            .execute(&self.0)
            .await?;
        } else {
            sqlx::query!(
                "UPDATE map_element SET station_id = NULL WHERE station_id = ($1)",
                station.id
            )
            .execute(&self.0)
            .await?;
        }

        Ok(())
    }

    async fn get_all_station_assignments(
        &self,
        map_id: Option<i32>,
    ) -> Result<Vec<StationAssignment>, AppError> {
        let rows = sqlx::query!(
            "
            SELECT s.ip, e.id FROM map_element AS e 
            INNER JOIN stations AS s ON e.station_id = s.id 
            WHERE ($1::int4 IS NULL OR e.map_id = $1)
            ",
            map_id
        )
        .fetch_all(&self.0)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| StationAssignment {
                seat_id: Some(r.id),
                station_ip: r.ip,
            })
            .collect())
    }
}
