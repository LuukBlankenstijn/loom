use async_trait::async_trait;
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct Map {
    pub id: i32,
    pub name: String,
    pub elements: Vec<MapElement>,
}

#[derive(Debug, Clone)]
pub enum MapElement {
    Wall(Wall),
    Door(Door),
    Seat(Seat),
}

#[derive(Debug, Clone)]
pub struct MapMetaData {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Door {
    pub id: Uuid,
    pub x: i32,
    pub y: i32,
    pub rotation: Rotation,
}

#[derive(Debug, Clone)]
pub struct Wall {
    pub id: Uuid,
    pub x_start: i32,
    pub y_start: i32,
    pub x_end: i32,
    pub y_end: i32,
}

#[derive(Debug, Clone)]
pub struct Seat {
    pub id: Uuid,
    pub x: i32,
    pub y: i32,
    pub rotation: Rotation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rotation {
    R0,
    R90,
    R180,
    R270,
}

impl Rotation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::R0 => "0",
            Self::R90 => "90",
            Self::R180 => "180",
            Self::R270 => "270",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "90" => Self::R90,
            "180" => Self::R180,
            "270" => Self::R270,
            _ => Self::R0,
        }
    }
}

#[async_trait]
pub trait MapRepository: Send + Sync {
    async fn get(&self, map_id: i32) -> Result<Option<Map>, AppError>;
    async fn get_by_contest(&self, contest_id: &str) -> Result<Option<Map>, AppError>;

    async fn create_map(&self, name: &str) -> Result<Map, AppError>;

    async fn get_all_metadata(&self) -> Result<Vec<Map>, AppError>;

    async fn delete_elements(&self, element_ids: &[Uuid]) -> Result<(), AppError>;
    async fn upsert_elements(&self, map_id: i32, elements: Vec<MapElement>)
    -> Result<(), AppError>;
}
