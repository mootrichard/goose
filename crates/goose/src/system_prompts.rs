use crate::config::ConfigError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// Represents a system prompt with metadata
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SystemPrompt {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub content: String,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub model_specific: Option<String>, // e.g., "gpt-4", "claude-3"
}

impl SystemPrompt {
    pub fn new(name: String, content: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description: None,
            content,
            is_default: false,
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
            model_specific: None,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_model_specific(mut self, model: String) -> Self {
        self.model_specific = Some(model);
        self
    }

    pub fn set_as_default(mut self) -> Self {
        self.is_default = true;
        self
    }

    pub fn update_content(&mut self, content: String) {
        self.content = content;
        self.updated_at = Utc::now();
    }
}

/// System prompt storage and management
#[derive(Debug)]
pub struct SystemPromptManager {
    config_dir: PathBuf,
    prompts_file: PathBuf,
}

impl Default for SystemPromptManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemPromptManager {
    pub fn new() -> Self {
        use crate::config::APP_STRATEGY;
        use etcetera::{choose_app_strategy, AppStrategy};

        let config_dir = choose_app_strategy(APP_STRATEGY.clone())
            .expect("goose requires a home dir")
            .config_dir();
        let prompts_file = config_dir.join("system_prompts.yaml");

        Self {
            config_dir,
            prompts_file,
        }
    }

    /// Initialize system prompts storage with built-in defaults
    pub fn initialize(&self) -> Result<(), ConfigError> {
        if !self.config_dir.exists() {
            fs::create_dir_all(&self.config_dir).map_err(|e| {
                ConfigError::DirectoryError(format!("Failed to create config directory: {}", e))
            })?;
        }

        // Only create defaults if no prompts file exists
        if !self.prompts_file.exists() {
            self.create_default_prompts()?;
        }

        Ok(())
    }

    /// Create default system prompts from built-in templates
    fn create_default_prompts(&self) -> Result<(), ConfigError> {
        let mut prompts = HashMap::new();

        // Create default system prompt from system.md
        let default_content = include_str!("prompts/system.md");
        let default_prompt = SystemPrompt::new("Default".to_string(), default_content.to_string())
            .with_description("Default Goose system prompt".to_string())
            .with_tags(vec!["default".to_string()])
            .set_as_default();

        prompts.insert(default_prompt.id.clone(), default_prompt);

        // Create GPT-4.1 specific prompt from embedded content
        let gpt4_content = include_str!("prompts/system_gpt_4.1.md");
        let gpt4_prompt =
            SystemPrompt::new("GPT-4.1 Optimized".to_string(), gpt4_content.to_string())
                .with_description("System prompt optimized for GPT-4.1 models".to_string())
                .with_model_specific("gpt-4.1".to_string())
                .with_tags(vec!["gpt-4".to_string(), "optimized".to_string()]);

        prompts.insert(gpt4_prompt.id.clone(), gpt4_prompt);

        self.save_prompts(&prompts)?;
        Ok(())
    }

    /// Load all system prompts
    pub fn load_prompts(&self) -> Result<HashMap<String, SystemPrompt>, ConfigError> {
        if !self.prompts_file.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(&self.prompts_file)?;
        let prompts: HashMap<String, SystemPrompt> = serde_yaml::from_str(&content)?;
        Ok(prompts)
    }

    /// Save all system prompts
    fn save_prompts(&self, prompts: &HashMap<String, SystemPrompt>) -> Result<(), ConfigError> {
        let content = serde_yaml::to_string(prompts)?;
        fs::write(&self.prompts_file, content)?;
        Ok(())
    }

    /// Create a new system prompt
    pub fn create_prompt(&self, prompt: SystemPrompt) -> Result<SystemPrompt, ConfigError> {
        let mut prompts = self.load_prompts()?;

        // If this is being set as default, unset other defaults
        if prompt.is_default {
            for existing_prompt in prompts.values_mut() {
                existing_prompt.is_default = false;
            }
        }

        prompts.insert(prompt.id.clone(), prompt.clone());
        self.save_prompts(&prompts)?;
        Ok(prompt)
    }

    /// Get system prompt by ID
    pub fn get_prompt(&self, id: &str) -> Result<Option<SystemPrompt>, ConfigError> {
        let prompts = self.load_prompts()?;
        Ok(prompts.get(id).cloned())
    }

    /// Get system prompt by name
    pub fn get_prompt_by_name(&self, name: &str) -> Result<Option<SystemPrompt>, ConfigError> {
        let prompts = self.load_prompts()?;
        Ok(prompts.values().find(|p| p.name == name).cloned())
    }

    /// Get the default system prompt
    pub fn get_default_prompt(&self) -> Result<Option<SystemPrompt>, ConfigError> {
        let prompts = self.load_prompts()?;
        Ok(prompts.values().find(|p| p.is_default).cloned())
    }

    /// Get system prompt for a specific model
    pub fn get_prompt_for_model(&self, model: &str) -> Result<Option<SystemPrompt>, ConfigError> {
        let prompts = self.load_prompts()?;

        // First try exact model match
        if let Some(prompt) = prompts
            .values()
            .find(|p| p.model_specific.as_ref().map_or(false, |m| m == model))
        {
            return Ok(Some(prompt.clone()));
        }

        // Then try partial model match (e.g., "gpt-4" matches "gpt-4.1")
        if let Some(prompt) = prompts.values().find(|p| {
            p.model_specific
                .as_ref()
                .map_or(false, |m| model.contains(m) || m.contains(model))
        }) {
            return Ok(Some(prompt.clone()));
        }

        // Fall back to default
        self.get_default_prompt()
    }

    /// List all system prompts
    pub fn list_prompts(&self) -> Result<Vec<SystemPrompt>, ConfigError> {
        let prompts = self.load_prompts()?;
        let mut prompt_list: Vec<SystemPrompt> = prompts.into_values().collect();
        prompt_list.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(prompt_list)
    }

    /// Update an existing system prompt
    pub fn update_prompt(
        &self,
        id: &str,
        updated_prompt: SystemPrompt,
    ) -> Result<SystemPrompt, ConfigError> {
        let mut prompts = self.load_prompts()?;

        if !prompts.contains_key(id) {
            return Err(ConfigError::NotFound(format!(
                "System prompt with ID {} not found",
                id
            )));
        }

        // If this is being set as default, unset other defaults
        if updated_prompt.is_default {
            for existing_prompt in prompts.values_mut() {
                existing_prompt.is_default = false;
            }
        }

        prompts.insert(id.to_string(), updated_prompt.clone());
        self.save_prompts(&prompts)?;
        Ok(updated_prompt)
    }

    /// Delete a system prompt
    pub fn delete_prompt(&self, id: &str) -> Result<(), ConfigError> {
        let mut prompts = self.load_prompts()?;

        if let Some(prompt) = prompts.get(id) {
            if prompt.is_default {
                return Err(ConfigError::DeserializeError(
                    "Cannot delete the default system prompt. Set another prompt as default first."
                        .to_string(),
                ));
            }
        }

        if prompts.remove(id).is_none() {
            return Err(ConfigError::NotFound(format!(
                "System prompt with ID {} not found",
                id
            )));
        }

        self.save_prompts(&prompts)?;
        Ok(())
    }

    /// Set a prompt as the default
    pub fn set_default_prompt(&self, id: &str) -> Result<(), ConfigError> {
        let mut prompts = self.load_prompts()?;

        if !prompts.contains_key(id) {
            return Err(ConfigError::NotFound(format!(
                "System prompt with ID {} not found",
                id
            )));
        }

        // Unset all defaults
        for prompt in prompts.values_mut() {
            prompt.is_default = false;
        }

        // Set the specified prompt as default
        if let Some(prompt) = prompts.get_mut(id) {
            prompt.is_default = true;
        }

        self.save_prompts(&prompts)?;
        Ok(())
    }

    /// Search prompts by tags
    pub fn search_by_tags(&self, tags: &[String]) -> Result<Vec<SystemPrompt>, ConfigError> {
        let prompts = self.load_prompts()?;
        let matching_prompts: Vec<SystemPrompt> = prompts
            .into_values()
            .filter(|prompt| tags.iter().any(|tag| prompt.tags.contains(tag)))
            .collect();
        Ok(matching_prompts)
    }

    /// Import system prompt from file
    pub fn import_from_file(
        &self,
        file_path: &PathBuf,
        name: String,
    ) -> Result<SystemPrompt, ConfigError> {
        let content = fs::read_to_string(file_path).map_err(|e| ConfigError::FileError(e))?;

        let prompt = SystemPrompt::new(name, content)
            .with_description(format!("Imported from {}", file_path.display()));

        self.create_prompt(prompt)
    }

    /// Export system prompt to file
    pub fn export_to_file(&self, id: &str, file_path: &PathBuf) -> Result<(), ConfigError> {
        let prompt = self.get_prompt(id)?.ok_or_else(|| {
            ConfigError::NotFound(format!("System prompt with ID {} not found", id))
        })?;

        fs::write(file_path, &prompt.content).map_err(|e| ConfigError::FileError(e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_manager() -> (SystemPromptManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let manager = SystemPromptManager {
            config_dir: temp_dir.path().to_path_buf(),
            prompts_file: temp_dir.path().join("system_prompts.yaml"),
        };
        (manager, temp_dir)
    }

    #[test]
    fn test_create_and_get_prompt() {
        let (manager, _temp_dir) = create_test_manager();

        let prompt = SystemPrompt::new("Test Prompt".to_string(), "Test content".to_string());
        let created = manager.create_prompt(prompt.clone()).unwrap();

        assert_eq!(created.name, "Test Prompt");
        assert_eq!(created.content, "Test content");

        let retrieved = manager.get_prompt(&created.id).unwrap().unwrap();
        assert_eq!(retrieved.name, created.name);
        assert_eq!(retrieved.content, created.content);
    }

    #[test]
    fn test_default_prompt_management() {
        let (manager, _temp_dir) = create_test_manager();
        manager.initialize().unwrap();

        let prompt1 =
            SystemPrompt::new("Prompt 1".to_string(), "Content 1".to_string()).set_as_default();
        let prompt2 = SystemPrompt::new("Prompt 2".to_string(), "Content 2".to_string());

        manager.create_prompt(prompt1).unwrap();
        let created2 = manager.create_prompt(prompt2).unwrap();

        // Prompt 1 should be default
        let default = manager.get_default_prompt().unwrap().unwrap();
        assert_eq!(default.name, "Prompt 1");

        // Set prompt 2 as default
        manager.set_default_prompt(&created2.id).unwrap();
        let new_default = manager.get_default_prompt().unwrap().unwrap();
        assert_eq!(new_default.name, "Prompt 2");
    }

    #[test]
    fn test_model_specific_prompts() {
        let (manager, _temp_dir) = create_test_manager();
        manager.initialize().unwrap();

        let gpt4_prompt = SystemPrompt::new("GPT-4".to_string(), "GPT-4 content".to_string())
            .with_model_specific("gpt-4".to_string());
        let claude_prompt = SystemPrompt::new("Claude".to_string(), "Claude content".to_string())
            .with_model_specific("claude-3".to_string());

        manager.create_prompt(gpt4_prompt).unwrap();
        manager.create_prompt(claude_prompt).unwrap();

        let gpt4_result = manager.get_prompt_for_model("gpt-4o").unwrap().unwrap();
        assert_eq!(gpt4_result.name, "GPT-4");

        let claude_result = manager
            .get_prompt_for_model("claude-3.5-sonnet")
            .unwrap()
            .unwrap();
        assert_eq!(claude_result.name, "Claude");
    }
}
