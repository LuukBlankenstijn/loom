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

#[async_trait]
impl TeamRepository for PgTeamRepo {
    async fn set_ip(&self, team_id: &str, ip: Option<&str>) -> Result<Option<String>, AppError> {
        let old_ip = sqlx::query!(
            "SELECT s.ip FROM teams t
             JOIN stations s ON t.team_station = s.id
             WHERE t.id = $1",
            team_id
        )
        .fetch_optional(&self.0)
        .await?
        .map(|r| r.ip);

        if let Some(ip) = ip {
            let conflict = sqlx::query!(
                "SELECT t.id FROM teams t
                 JOIN stations s ON t.team_station = s.id
                 WHERE s.ip = $1 AND t.id != $2",
                ip,
                team_id
            )
            .fetch_optional(&self.0)
            .await?;

            if conflict.is_some() {
                return Err(AppError::AlreadyExists(
                    "ip is already in use by a different team".into(),
                ));
            }

            sqlx::query!(
                "UPDATE teams SET team_station = (SELECT id FROM stations WHERE ip = $1) WHERE id = $2",
                ip,
                team_id
            )
            .execute(&self.0)
            .await?;

            Ok(Some(ip.to_string()))
        } else {
            sqlx::query!("UPDATE teams SET team_station = NULL WHERE id = $1", team_id)
                .execute(&self.0)
                .await?;

            Ok(old_ip)
        }
    }

    async fn get_all(&self, contest_id: &str) -> Result<Vec<Team>, AppError> {
        let rows = sqlx::query!(
            r#"SELECT t.id, t.name, s.ip as "ip?"
             FROM teams t
             JOIN contest_teams ct ON ct.team_id = t.id
             LEFT JOIN stations s ON t.team_station = s.id
             WHERE ct.contest_id = $1"#,
            contest_id
        )
        .fetch_all(&self.0)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| Team { id: r.id, name: r.name, ip: r.ip })
            .collect())
    }

    async fn get_by_ip(&self, ip: &str) -> Result<Team, AppError> {
        let row = sqlx::query!(
            "SELECT t.id, t.name, s.ip
             FROM teams t
             JOIN stations s ON t.team_station = s.id
             WHERE s.ip = $1",
            ip
        )
        .fetch_optional(&self.0)
        .await?;

        match row {
            Some(r) => Ok(Team { id: r.id, name: r.name, ip: Some(r.ip) }),
            None => Err(AppError::NotFound("team not found".to_string())),
        }
    }
}
