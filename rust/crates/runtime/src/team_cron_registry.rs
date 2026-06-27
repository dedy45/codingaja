use std::collections::BTreeMap;
use std::sync::{OnceLock, RwLock};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TeamRecord {
    pub name: String,
    pub description: Option<String>,
    pub lead_agent_id: String,
    pub created_at: String,
}

#[derive(Debug, Default)]
pub struct TeamRegistry {
    teams: BTreeMap<String, TeamRecord>,
}

impl TeamRegistry {
    pub fn create(
        &mut self,
        name: String,
        description: Option<String>,
    ) -> Result<TeamRecord, String> {
        if self.teams.contains_key(&name) {
            return Err(format!("team `{name}` already exists"));
        }
        let record = TeamRecord {
            lead_agent_id: format!("team-lead@{name}"),
            name: name.clone(),
            description,
            created_at: iso8601_now(),
        };
        self.teams.insert(name, record.clone());
        Ok(record)
    }

    pub fn delete(&mut self, name: &str) -> Option<TeamRecord> {
        self.teams.remove(name)
    }

    pub fn get(&self, name: &str) -> Option<&TeamRecord> {
        self.teams.get(name)
    }

    #[must_use]
    pub fn list(&self) -> Vec<TeamRecord> {
        self.teams.values().cloned().collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CronRecord {
    pub id: String,
    pub cron: String,
    pub prompt: String,
    pub recurring: bool,
    pub durable: bool,
    pub created_at: String,
    pub team_name: Option<String>,
}

#[derive(Debug, Default)]
pub struct CronRegistry {
    next_id: u64,
    jobs: BTreeMap<String, CronRecord>,
}

impl CronRegistry {
    #[must_use]
    pub fn create(
        &mut self,
        cron: String,
        prompt: String,
        recurring: bool,
        durable: bool,
        team_name: Option<String>,
    ) -> CronRecord {
        self.next_id = self.next_id.saturating_add(1);
        let id = format!("cron-{}", self.next_id);
        let record = CronRecord {
            id: id.clone(),
            cron,
            prompt,
            recurring,
            durable,
            created_at: iso8601_now(),
            team_name,
        };
        self.jobs.insert(id, record.clone());
        record
    }

    pub fn delete(&mut self, id: &str) -> Option<CronRecord> {
        self.jobs.remove(id)
    }

    #[must_use]
    pub fn list(&self) -> Vec<CronRecord> {
        self.jobs.values().cloned().collect()
    }
}

#[must_use]
pub fn global_team_registry() -> &'static RwLock<TeamRegistry> {
    static REGISTRY: OnceLock<RwLock<TeamRegistry>> = OnceLock::new();
    REGISTRY.get_or_init(|| RwLock::new(TeamRegistry::default()))
}

#[must_use]
pub fn global_cron_registry() -> &'static RwLock<CronRegistry> {
    static REGISTRY: OnceLock<RwLock<CronRegistry>> = OnceLock::new();
    REGISTRY.get_or_init(|| RwLock::new(CronRegistry::default()))
}

fn iso8601_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    format!("{secs}")
}

#[cfg(test)]
mod tests {
    use super::{global_cron_registry, global_team_registry};

    #[test]
    fn creates_team_and_cron_records() {
        let mut teams = global_team_registry()
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let team = teams
            .create("alpha".to_string(), Some("test team".to_string()))
            .expect("team create should succeed");
        assert_eq!(team.name, "alpha");

        let mut crons = global_cron_registry()
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let cron = crons.create(
            "*/5 * * * *".to_string(),
            "ping".to_string(),
            true,
            false,
            Some("alpha".to_string()),
        );
        assert_eq!(cron.team_name.as_deref(), Some("alpha"));

        let _ = teams.delete("alpha");
        let _ = crons.delete(&cron.id);
    }
}
