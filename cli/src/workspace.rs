use anyhow::{anyhow, Result};
use dbkp_core::{databases::DatabaseConfig, storage::provider::StorageConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub name: String,
    pub database: DatabaseConfig,
    pub storage: StorageConfig,
    pub created_at: String,
    pub last_used: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceCollection {
    pub workspaces: HashMap<String, Workspace>,
    pub active_workspace: Option<String>,
}

impl WorkspaceCollection {
    pub fn new() -> Self {
        Self {
            workspaces: HashMap::new(),
            active_workspace: None,
        }
    }

    pub fn add_workspace(&mut self, workspace: Workspace) {
        self.workspaces.insert(workspace.name.clone(), workspace);
    }

    pub fn get_workspace(&self, name: &str) -> Option<&Workspace> {
        self.workspaces.get(name)
    }

    pub fn remove_workspace(&mut self, name: &str) -> Option<Workspace> {
        let workspace = self.workspaces.remove(name);
        if Some(name) == self.active_workspace.as_deref() {
            self.active_workspace = None;
        }
        workspace
    }

    pub fn set_active(&mut self, name: &str) -> Result<()> {
        if self.workspaces.contains_key(name) {
            self.active_workspace = Some(name.to_string());
            Ok(())
        } else {
            Err(anyhow!("Workspace '{}' does not exist", name))
        }
    }

    pub fn get_active(&self) -> Option<&Workspace> {
        self.active_workspace
            .as_ref()
            .and_then(|name| self.workspaces.get(name))
    }

    pub fn list_workspaces(&self) -> Vec<&Workspace> {
        self.workspaces.values().collect()
    }
}

pub struct WorkspaceManager {
    config_path: PathBuf,
}

impl WorkspaceManager {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("Could not determine config directory"))?
            .join("dbkp");

        fs::create_dir_all(&config_dir)?;

        Ok(Self {
            config_path: config_dir.join("workspaces.json"),
        })
    }

    pub fn load(&self) -> Result<WorkspaceCollection> {
        if !self.config_path.exists() {
            return Ok(WorkspaceCollection::new());
        }

        let content = fs::read_to_string(&self.config_path)?;
        let collection: WorkspaceCollection = serde_json::from_str(&content)?;
        Ok(collection)
    }

    pub fn save(&self, collection: &WorkspaceCollection) -> Result<()> {
        let content = serde_json::to_string_pretty(collection)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }
}

impl Default for WorkspaceManager {
    fn default() -> Self {
        Self::new().expect("Failed to create workspace manager")
    }
}
