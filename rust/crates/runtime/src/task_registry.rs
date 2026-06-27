use std::collections::BTreeMap;
use std::sync::{OnceLock, RwLock};

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Stopped,
}

impl TaskStatus {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Stopped => "stopped",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskRecord {
    pub id: String,
    pub subject: String,
    pub description: String,
    pub active_form: Option<String>,
    pub status: TaskStatus,
    pub owner: Option<String>,
    pub blocks: Vec<String>,
    pub blocked_by: Vec<String>,
    pub metadata: BTreeMap<String, Value>,
    pub output: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Default)]
pub struct TaskRegistry {
    next_id: u64,
    tasks: BTreeMap<String, TaskRecord>,
}

impl TaskRegistry {
    #[must_use]
    pub fn create(
        &mut self,
        subject: String,
        description: String,
        active_form: Option<String>,
        metadata: BTreeMap<String, Value>,
    ) -> TaskRecord {
        self.next_id = self.next_id.saturating_add(1);
        let id = self.next_id.to_string();
        let now = iso8601_now();
        let record = TaskRecord {
            id: id.clone(),
            subject,
            description,
            active_form,
            status: TaskStatus::Pending,
            owner: None,
            blocks: Vec::new(),
            blocked_by: Vec::new(),
            metadata,
            output: String::new(),
            created_at: now.clone(),
            updated_at: now,
        };
        self.tasks.insert(id, record.clone());
        record
    }

    pub fn get(&self, id: &str) -> Option<&TaskRecord> {
        self.tasks.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut TaskRecord> {
        self.tasks.get_mut(id)
    }

    pub fn remove(&mut self, id: &str) -> Option<TaskRecord> {
        self.tasks.remove(id)
    }

    #[must_use]
    pub fn list(&self) -> Vec<TaskRecord> {
        self.tasks.values().cloned().collect()
    }

    pub fn append_output(&mut self, id: &str, chunk: &str) -> Result<(), String> {
        let task = self
            .tasks
            .get_mut(id)
            .ok_or_else(|| format!("task `{id}` not found"))?;
        task.output.push_str(chunk);
        task.updated_at = iso8601_now();
        Ok(())
    }
}

#[must_use]
pub fn global_task_registry() -> &'static RwLock<TaskRegistry> {
    static REGISTRY: OnceLock<RwLock<TaskRegistry>> = OnceLock::new();
    REGISTRY.get_or_init(|| RwLock::new(TaskRegistry::default()))
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
    use super::{global_task_registry, TaskStatus};

    #[test]
    fn creates_and_updates_task_registry_entries() {
        let mut registry = global_task_registry()
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let created = registry.create(
            "test task".to_string(),
            "details".to_string(),
            Some("running tests".to_string()),
            std::collections::BTreeMap::new(),
        );
        assert_eq!(created.status, TaskStatus::Pending);

        let task = registry.get_mut(&created.id).expect("task should exist");
        task.status = TaskStatus::InProgress;
        task.output.push_str("hello");

        let fetched = registry.get(&created.id).expect("task should still exist");
        assert_eq!(fetched.status, TaskStatus::InProgress);
        assert!(fetched.output.contains("hello"));

        let _ = registry.remove(&created.id);
    }
}
