use async_trait::async_trait;
use serde::Deserialize;

use crate::config::IcpcApiConfig;
use crate::domain::{Team, TeamRepository};
use crate::error::AppError;

pub struct HttpTeamRepo {
    client: reqwest::Client,
    config: IcpcApiConfig,
}

#[derive(Deserialize)]
struct ApiTeam {
    id: String,
    name: String,
}

#[derive(Deserialize)]
struct ApiUser {
    id: String,
    #[serde(default)]
    team_id: Option<String>,
    #[serde(default)]
    ip: Option<String>,
    name: String,
    username: String,
    #[serde(default)]
    email: Option<String>,
    enabled: bool,
    #[serde(default)]
    roles: Vec<String>,
}

#[derive(Deserialize)]
struct ApiContestRef {
    id: String,
}

impl HttpTeamRepo {
    pub fn new(config: IcpcApiConfig, client: reqwest::Client) -> Self {
        Self { client, config }
    }

    async fn get_json<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T, AppError> {
        let body = self
            .client
            .get(url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .send()
            .await?
            .error_for_status()
            .map_err(|e| AppError::Internal(format!("contest API error: {e}")))?
            .text()
            .await?;

        serde_json::from_str(&body).map_err(|e| {
            tracing::error!(url, body, error = ?e, "failed to decode contest API response");
            AppError::Internal(format!("contest API decode error: {e}"))
        })
    }

    async fn get_users(&self) -> Result<Vec<ApiUser>, AppError> {
        self.get_json(&format!("{}/api/v4/users", self.config.base_url))
            .await
    }
}

#[async_trait]
impl TeamRepository for HttpTeamRepo {
    async fn get_all(&self, contest_id: &str) -> Result<Vec<Team>, AppError> {
        let teams_url = format!(
            "{}/api/v4/contests/{}/teams",
            self.config.base_url, contest_id
        );
        let (teams, users): (Vec<ApiTeam>, Vec<ApiUser>) =
            tokio::try_join!(self.get_json(&teams_url), self.get_users())?;

        let mut ip_map = std::collections::HashMap::new();
        for u in &users {
            if let (Some(tid), Some(ip)) = (&u.team_id, &u.ip)
                && !tid.is_empty()
                && !ip.is_empty()
            {
                ip_map.insert(tid.clone(), ip.clone());
            }
        }

        Ok(teams
            .into_iter()
            .map(|t| {
                let ip = ip_map.get(&t.id).cloned();
                Team {
                    id: t.id,
                    name: t.name,
                    ip,
                }
            })
            .collect())
    }

    async fn set_ip(&self, team_id: &str, ip: Option<&str>) -> Result<Option<String>, AppError> {
        let users = self.get_users().await?;
        let new_ip = ip.unwrap_or("");

        let old_ip = users
            .iter()
            .find(|u| u.team_id.as_deref() == Some(team_id))
            .and_then(|u| u.ip.clone());

        if !new_ip.is_empty() {
            for u in &users {
                if u.ip.as_deref() == Some(new_ip) && u.team_id.as_deref() != Some(team_id) {
                    return Err(AppError::AlreadyExists(
                        "ip already in use by another team".into(),
                    ));
                }
            }
        }

        let team_users: Vec<&ApiUser> = users
            .iter()
            .filter(|u| u.team_id.as_deref() == Some(team_id))
            .collect();

        let mut handles = Vec::new();

        for user in team_users {
            let client = self.client.clone();
            let url = format!("{}/api/v4/users/{}", self.config.base_url, user.id);
            let username = self.config.username.clone();
            let password = self.config.password.clone();

            let mut form: Vec<(String, String)> = vec![
                ("id".into(), user.id.clone()),
                ("name".into(), user.name.clone()),
                ("username".into(), user.username.clone()),
                ("email".into(), user.email.clone().unwrap_or_default()),
                ("ip".into(), new_ip.to_string()),
                (
                    "enabled".into(),
                    if user.enabled { "1" } else { "0" }.into(),
                ),
            ];

            for role in &user.roles {
                form.push(("roles[]".into(), role.clone()));
            }

            handles.push(tokio::spawn(async move {
                let resp = client
                    .put(&url)
                    .basic_auth(&username, Some(&password))
                    .form(&form)
                    .send()
                    .await
                    .map_err(AppError::from)?;

                if !resp.status().is_success() {
                    return Err(AppError::Internal(format!(
                        "contest API error: status {}",
                        resp.status()
                    )));
                }
                Ok(())
            }));
        }

        for h in handles {
            h.await.map_err(|e| AppError::Internal(e.to_string()))??;
        }

        if ip.is_some() {
            Ok(Some(new_ip.to_string()))
        } else {
            Ok(old_ip)
        }
    }

    async fn get_by_ip(&self, ip: &str) -> Result<Option<Team>, AppError> {
        let users = self.get_users().await?;
        let team_id = match users.iter().find(|u| u.ip.as_deref() == Some(ip)) {
            Some(u) if u.team_id.as_ref().is_some_and(|t| !t.is_empty()) => {
                u.team_id.clone().unwrap()
            }
            _ => return Ok(None),
        };

        let contests: Vec<ApiContestRef> = self
            .get_json(&format!("{}/api/v4/contests", self.config.base_url))
            .await?;

        for contest in contests {
            let url = format!(
                "{}/api/v4/contests/{}/teams",
                self.config.base_url, contest.id
            );
            if let Ok(teams) = self.get_json::<Vec<ApiTeam>>(&url).await
                && let Some(t) = teams.into_iter().find(|t| t.id == team_id)
            {
                return Ok(Some(Team {
                    id: t.id,
                    name: t.name,
                    ip: Some(ip.to_string()),
                }));
            }
        }

        Ok(Some(Team {
            id: team_id,
            name: String::new(),
            ip: Some(ip.to_string()),
        }))
    }
}
