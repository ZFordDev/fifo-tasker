use std::collections::HashSet;

use serde::{Deserialize, Serialize};

const SCHEMA_VERSION: u32 = 1;

#[derive(Default)]
pub(crate) struct TaskData {
    pub(crate) document: TaskDocument,
    pub(crate) load_error: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Task {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) completed: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "RawTaskDocument")]
pub(crate) struct TaskDocument {
    schema_version: u32,
    tasks: Vec<Task>, // Array order is queue order. No ticket machine required.
}

// Parse into a temporary shape so invalid data never becomes an accepted document.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTaskDocument {
    schema_version: u32,
    tasks: Vec<Task>,
}

impl Default for TaskDocument {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            tasks: Vec::new(),
        }
    }
}

impl TaskDocument {
    pub(crate) fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub(crate) fn tasks(&self) -> &[Task] {
        &self.tasks
    }
}

impl TryFrom<RawTaskDocument> for TaskDocument {
    type Error = String;

    fn try_from(document: RawTaskDocument) -> Result<Self, Self::Error> {
        if document.schema_version != SCHEMA_VERSION {
            return Err(format!(
                "Unsupported task schema version {}; expected {SCHEMA_VERSION}",
                document.schema_version
            ));
        }
        let mut ids = HashSet::new();
        for (index, task) in document.tasks.iter().enumerate() {
            if task.id.is_empty() || task.id.trim() != task.id {
                return Err(format!(
                    "Task {} needs a nonblank ID without surrounding whitespace",
                    index + 1
                ));
            }
            if !ids.insert(&task.id) {
                return Err(format!("Duplicate task ID: {}", task.id));
            }
            if task.title.trim().is_empty() {
                return Err(format!("Task {} needs a nonblank title", task.id));
            }
        }
        Ok(Self {
            schema_version: document.schema_version,
            tasks: document.tasks,
        })
    }
}
