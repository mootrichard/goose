use clap::{Args, Subcommand};
use goose::system_prompts::{SystemPrompt, SystemPromptManager};
use std::io::Read;
use std::path::PathBuf;
use tabled::{Table, Tabled};

#[derive(Args)]
pub struct SystemPromptArgs {
    #[command(subcommand)]
    pub command: SystemPromptCommand,
}

#[derive(Subcommand)]
pub enum SystemPromptCommand {
    /// List all system prompts
    List {
        /// Show only prompts with specific tags
        #[arg(long, value_delimiter = ',')]
        tags: Option<Vec<String>>,
        
        /// Show detailed information
        #[arg(long, short)]
        detailed: bool,
    },
    /// Create a new system prompt
    Create {
        /// Name of the system prompt
        name: String,
        
        /// Description of the system prompt
        #[arg(long, short)]
        description: Option<String>,
        
        /// Content of the system prompt (can be - for stdin)
        #[arg(long, short)]
        content: Option<String>,
        
        /// File to read content from
        #[arg(long, short)]
        file: Option<PathBuf>,
        
        /// Tags to associate with the prompt
        #[arg(long, value_delimiter = ',')]
        tags: Option<Vec<String>>,
        
        /// Model this prompt is optimized for
        #[arg(long)]
        model: Option<String>,
        
        /// Set as the default system prompt
        #[arg(long)]
        default: bool,
    },
    /// Show details of a specific system prompt
    Show {
        /// Prompt ID or name
        identifier: String,
        
        /// Show the raw content without formatting
        #[arg(long)]
        raw: bool,
    },
    /// Update an existing system prompt
    Update {
        /// Prompt ID or name
        identifier: String,
        
        /// New name for the prompt
        #[arg(long)]
        name: Option<String>,
        
        /// New description
        #[arg(long)]
        description: Option<String>,
        
        /// New content (can be - for stdin)
        #[arg(long)]
        content: Option<String>,
        
        /// File to read new content from
        #[arg(long)]
        file: Option<PathBuf>,
        
        /// New tags (replaces existing tags)
        #[arg(long, value_delimiter = ',')]
        tags: Option<Vec<String>>,
        
        /// New model specification
        #[arg(long)]
        model: Option<String>,
    },
    /// Delete a system prompt
    Delete {
        /// Prompt ID or name
        identifier: String,
        
        /// Skip confirmation prompt
        #[arg(long, short)]
        yes: bool,
    },
    /// Set a prompt as the default
    SetDefault {
        /// Prompt ID or name
        identifier: String,
    },
    /// Import a system prompt from a file
    Import {
        /// File to import from
        file: PathBuf,
        
        /// Name for the imported prompt
        name: String,
        
        /// Description for the imported prompt
        #[arg(long)]
        description: Option<String>,
        
        /// Tags to associate with the prompt
        #[arg(long, value_delimiter = ',')]
        tags: Option<Vec<String>>,
        
        /// Model this prompt is optimized for
        #[arg(long)]
        model: Option<String>,
    },
    /// Export a system prompt to a file
    Export {
        /// Prompt ID or name
        identifier: String,
        
        /// Output file path
        file: PathBuf,
    },
}

#[derive(Tabled)]
struct PromptSummary {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Default")]
    is_default: String,
    #[tabled(rename = "Model")]
    model: String,
    #[tabled(rename = "Tags")]
    tags: String,
    #[tabled(rename = "Updated")]
    updated: String,
}

impl From<SystemPrompt> for PromptSummary {
    fn from(prompt: SystemPrompt) -> Self {
        Self {
            id: prompt.id[..8].to_string(), // Show only first 8 chars of ID
            name: prompt.name,
            is_default: if prompt.is_default { "Yes" } else { "No" }.to_string(),
            model: prompt.model_specific.unwrap_or_else(|| "Any".to_string()),
            tags: prompt.tags.join(", "),
            updated: prompt.updated_at.format("%Y-%m-%d").to_string(),
        }
    }
}

pub async fn handle_system_prompt_command(args: SystemPromptArgs) -> anyhow::Result<()> {
    let manager = SystemPromptManager::new();
    manager.initialize()?;

    match args.command {
        SystemPromptCommand::List { tags, detailed } => {
            let prompts = if let Some(tags) = tags {
                manager.search_by_tags(&tags)?
            } else {
                manager.list_prompts()?
            };

            if prompts.is_empty() {
                println!("No system prompts found.");
                return Ok(());
            }

            if detailed {
                for prompt in prompts {
                    print_prompt_details(&prompt);
                    println!("{}", "-".repeat(80));
                }
            } else {
                let summaries: Vec<PromptSummary> = prompts.into_iter().map(Into::into).collect();
                let table = Table::new(summaries);
                println!("{}", table);
            }
        }

        SystemPromptCommand::Create {
            name,
            description,
            content,
            file,
            tags,
            model,
            default,
        } => {
            let content = get_content(content, file).await?
                .ok_or_else(|| anyhow::anyhow!("Content is required for creating a system prompt. Use --content or --file."))?;
            
            let mut prompt = SystemPrompt::new(name, content);
            
            if let Some(desc) = description {
                prompt = prompt.with_description(desc);
            }
            
            if let Some(tags) = tags {
                prompt = prompt.with_tags(tags);
            }
            
            if let Some(model) = model {
                prompt = prompt.with_model_specific(model);
            }
            
            if default {
                prompt = prompt.set_as_default();
            }

            let created_prompt = manager.create_prompt(prompt)?;
            println!("Created system prompt: {} (ID: {})", created_prompt.name, created_prompt.id);
        }

        SystemPromptCommand::Show { identifier, raw } => {
            let prompt = find_prompt(&manager, &identifier)?;
            
            if raw {
                println!("{}", prompt.content);
            } else {
                print_prompt_details(&prompt);
            }
        }

        SystemPromptCommand::Update {
            identifier,
            name,
            description,
            content,
            file,
            tags,
            model,
        } => {
            let mut prompt = find_prompt(&manager, &identifier)?;
            
            if let Some(name) = name {
                prompt.name = name;
            }
            
            if let Some(description) = description {
                prompt.description = Some(description);
            }
            
            if let Some(content) = get_content(content, file).await? {
                prompt.update_content(content);
            }
            
            if let Some(tags) = tags {
                prompt.tags = tags;
            }
            
            if let Some(model) = model {
                prompt.model_specific = Some(model);
            }

            manager.update_prompt(&prompt.id.clone(), prompt)?;
            println!("Updated system prompt: {}", identifier);
        }

        SystemPromptCommand::Delete { identifier, yes } => {
            let prompt = find_prompt(&manager, &identifier)?;
            
            if !yes {
                println!("Are you sure you want to delete the system prompt '{}'? (y/N)", prompt.name);
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if !input.trim().to_lowercase().starts_with('y') {
                    println!("Cancelled.");
                    return Ok(());
                }
            }

            manager.delete_prompt(&prompt.id)?;
            println!("Deleted system prompt: {}", prompt.name);
        }

        SystemPromptCommand::SetDefault { identifier } => {
            let prompt = find_prompt(&manager, &identifier)?;
            manager.set_default_prompt(&prompt.id)?;
            println!("Set '{}' as the default system prompt", prompt.name);
        }

        SystemPromptCommand::Import {
            file,
            name,
            description,
            tags,
            model,
        } => {
            let mut prompt = manager.import_from_file(&file, name)?;
            
            if let Some(desc) = description {
                prompt.description = Some(desc);
            }
            
            if let Some(tags) = tags {
                prompt.tags = tags;
            }
            
            if let Some(model) = model {
                prompt.model_specific = Some(model);
            }

            let updated_prompt = manager.update_prompt(&prompt.id.clone(), prompt)?;
            println!("Imported system prompt: {} (ID: {})", updated_prompt.name, updated_prompt.id);
        }

        SystemPromptCommand::Export { identifier, file } => {
            let prompt = find_prompt(&manager, &identifier)?;
            manager.export_to_file(&prompt.id, &file)?;
            println!("Exported system prompt '{}' to {}", prompt.name, file.display());
        }
    }

    Ok(())
}

async fn get_content(content: Option<String>, file: Option<PathBuf>) -> anyhow::Result<Option<String>> {
    match (content, file) {
        (Some(content), None) => {
            if content == "-" {
                // Read from stdin
                let mut buffer = String::new();
                std::io::stdin().read_to_string(&mut buffer)?;
                Ok(Some(buffer.trim().to_string()))
            } else {
                Ok(Some(content))
            }
        }
        (None, Some(file)) => {
            let content = std::fs::read_to_string(file)?;
            Ok(Some(content.trim().to_string()))
        }
        (Some(_), Some(_)) => {
            anyhow::bail!("Cannot specify both --content and --file options");
        }
        (None, None) => Ok(None),
    }
}

fn find_prompt(manager: &SystemPromptManager, identifier: &str) -> anyhow::Result<SystemPrompt> {
    // Try to find by ID first
    if let Ok(Some(prompt)) = manager.get_prompt(identifier) {
        return Ok(prompt);
    }
    
    // Then try by name
    if let Ok(Some(prompt)) = manager.get_prompt_by_name(identifier) {
        return Ok(prompt);
    }
    
    anyhow::bail!("System prompt '{}' not found", identifier);
}

fn print_prompt_details(prompt: &SystemPrompt) {
    println!("ID: {}", prompt.id);
    println!("Name: {}", prompt.name);
    
    if let Some(description) = &prompt.description {
        println!("Description: {}", description);
    }
    
    println!("Default: {}", if prompt.is_default { "Yes" } else { "No" });
    
    if let Some(model) = &prompt.model_specific {
        println!("Model: {}", model);
    }
    
    if !prompt.tags.is_empty() {
        println!("Tags: {}", prompt.tags.join(", "));
    }
    
    println!("Created: {}", prompt.created_at.format("%Y-%m-%d %H:%M:%S"));
    println!("Updated: {}", prompt.updated_at.format("%Y-%m-%d %H:%M:%S"));
    println!("\nContent:");
    println!("{}", prompt.content);
}

