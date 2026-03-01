use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::{Team, TeamRepository};
use crate::error::AppError;

pub struct PgTeamRepo(PgPool);

impl PgTeamRepo {
    pub fn new(pool: PgPool) -> Self {
        Self(pool)
    }
}

#[derive(sqlx::FromRow)]
struct TeamRow {
    id: String,
    name: String,
    ip: Option<String>,
}

impl From<TeamRow> for Team {
    fn from(r: TeamRow) -> Self {
        Self {
            id: r.id,
            name: r.name,
            ip: r.ip,
        }
    }
}

#[async_trait]
impl TeamRepository for PgTeamRepo {
    async fn set_ip(&self, team_id: &str, ip: Option<&str>) -> Result<(), AppError> {
        if let Some(ip) = ip {
            let conflict: Option<(String,)> = sqlx::query_as(
                "SELECT t.id FROM teams t
                 JOIN stations s ON t.team_station = s.id
                 WHERE s.ip = $1 AND t.id != $2",
            )
            .bind(ip)
            .bind(team_id)
            .fetch_optional(&self.0)
            .await?;

            if conflict.is_some() {
                return Err(AppError::AlreadyExists(
                    "ip is already in use by a different team".into(),
                ));
            }

            sqlx::query(
                "UPDATE teams SET team_station = (SELECT id FROM stations WHERE ip = $1) WHERE id = $2",
            )
            .bind(ip)
            .bind(team_id)
            .execute(&self.0)
            .await?;
        } else {
            sqlx::query("UPDATE teams SET team_station = NULL WHERE id = $1")
                .bind(team_id)
                .execute(&self.0)
                .await?;
        }
        Ok(())
    }

    async fn get_all(&self, contest_id: &str) -> Result<Vec<Team>, AppError> {
        let rows = sqlx::query_as::<_, TeamRow>(
            "SELECT t.id, t.name, s.ip
             FROM teams t
             JOIN contest_teams ct ON ct.team_id = t.id
             LEFT JOIN stations s ON t.team_station = s.id
             WHERE ct.contest_id = $1",
        )
        .bind(contest_id)
        .fetch_all(&self.0)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn get_by_ip(&self, ip: &str) -> Result<Option<Team>, AppError> {
        let row = sqlx::query_as::<_, TeamRow>(
            "SELECT t.id, t.name, s.ip
             FROM teams t
             JOIN stations s ON t.team_station = s.id
             WHERE s.ip = $1",
        )
        .bind(ip)
        .fetch_optional(&self.0)
        .await?;

        Ok(row.map(Into::into))
    }
}
