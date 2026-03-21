#![warn(missing_docs)]

use chrono::Utc;
use nargo_types::{Error, Result, Span};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

use crate::types::ChangeType;

/// Change set data structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSet {
    /// The ID of the change set.
    pub id: String,
    /// The type of change.
    pub r#type: ChangeType,
    /// A summary of the change.
    pub summary: String,
    /// Detailed description of the change.
    pub description: Option<String>,
    /// The author of the change.
    pub author: Option<String>,
    /// The packages affected by the change.
    pub packages: Vec<String>,
    /// Whether the change is a prerelease.
    pub prerelease: bool,
}

/// Change set template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSetTemplate {
    /// The name of the template.
    pub name: String,
    /// The default change type.
    pub default_type: ChangeType,
    /// The default description template.
    pub description_template: Option<String>,
    /// The default packages.
    pub default_packages: Vec<String>,
    /// Whether the change is a prerelease by default.
    pub default_prerelease: bool,
}

/// Change set preset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSetPreset {
    /// The name of the preset.
    pub name: String,
    /// The description of the preset.
    pub description: Option<String>,
    /// The template to use for this preset.
    pub template: String,
    /// Additional metadata for the preset.
    pub metadata: HashMap<String, String>,
}

/// Change set manager.
pub struct ChangeSetManager {
    /// The directory where change sets are stored.
    pub changes_dir: PathBuf,
}

impl ChangeSetManager {
    /// Creates a new change set manager.
    pub fn new(changes_dir: &Path) -> Self {
        Self { changes_dir: changes_dir.to_path_buf() }
    }

    /// Creates a new change set.
    pub fn create_change_set(&self, change_set: &ChangeSet) -> Result<PathBuf> {
        // Ensure the changes directory exists
        fs::create_dir_all(&self.changes_dir)?;

        // Generate the file path
        let file_name = format!("{}-{}.json", change_set.id, change_set.r#type.as_str());
        let file_path = self.changes_dir.join(file_name);

        // Write the change set to file
        let mut file = File::create(&file_path)?;
        let content = serde_json::to_string_pretty(change_set).map_err(|e| Error::external_error("json".to_string(), e.to_string(), Span::unknown()))?;
        file.write_all(content.as_bytes())?;

        Ok(file_path)
    }

    /// Reads all change sets from the changes directory.
    pub fn read_change_sets(&self) -> Result<Vec<ChangeSet>> {
        let mut change_sets = Vec::new();

        for entry in WalkDir::new(&self.changes_dir).into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()).filter(|e| e.path().extension().map(|ext| ext == "json").unwrap_or(false)) {
            let content = fs::read_to_string(entry.path())?;
            let change_set: ChangeSet = serde_json::from_str(&content).map_err(|e| Error::external_error("json".to_string(), e.to_string(), Span::unknown()))?;
            change_sets.push(change_set);
        }

        Ok(change_sets)
    }

    /// Generates a changelog from the change sets.
    pub fn generate_changelog(&self, version: &str, date: &str) -> Result<String> {
        let change_sets = self.read_change_sets()?;
        let mut changelog = format!("# Changelog\n\n## [{}] - {}\n\n", version, date);

        // Group change sets by type
        let mut breaking = Vec::new();
        let mut features = Vec::new();
        let mut fixes = Vec::new();
        let mut others = Vec::new();

        for change_set in &change_sets {
            match change_set.r#type {
                ChangeType::Breaking => breaking.push(change_set),
                ChangeType::Feature => features.push(change_set),
                ChangeType::Fix => fixes.push(change_set),
                _ => others.push(change_set),
            }
        }

        // Add breaking changes
        if !breaking.is_empty() {
            changelog.push_str("### Breaking Changes\n\n");
            for change in &breaking {
                changelog.push_str(&format!("- {}\n", change.summary));
                if let Some(desc) = &change.description {
                    changelog.push_str(&format!("  {}\n", desc));
                }
            }
            changelog.push_str("\n");
        }

        // Add features
        if !features.is_empty() {
            changelog.push_str("### Features\n\n");
            for change in &features {
                changelog.push_str(&format!("- {}\n", change.summary));
                if let Some(desc) = &change.description {
                    changelog.push_str(&format!("  {}\n", desc));
                }
            }
            changelog.push_str("\n");
        }

        // Add fixes
        if !fixes.is_empty() {
            changelog.push_str("### Bug Fixes\n\n");
            for change in &fixes {
                changelog.push_str(&format!("- {}\n", change.summary));
                if let Some(desc) = &change.description {
                    changelog.push_str(&format!("  {}\n", desc));
                }
            }
            changelog.push_str("\n");
        }

        // Add other changes
        if !others.is_empty() {
            changelog.push_str("### Other Changes\n\n");
            for change in &others {
                changelog.push_str(&format!("- [{}] {}\n", change.r#type.as_str(), change.summary));
                if let Some(desc) = &change.description {
                    changelog.push_str(&format!("  {}\n", desc));
                }
            }
            changelog.push_str("\n");
        }

        Ok(changelog)
    }

    /// Clears all change sets after generating a changelog.
    pub fn clear_change_sets(&self) -> Result<()> {
        for entry in WalkDir::new(&self.changes_dir).into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()).filter(|e| e.path().extension().map(|ext| ext == "json").unwrap_or(false)) {
            fs::remove_file(entry.path())?;
        }

        Ok(())
    }

    /// Merges multiple change sets into a single change set.
    pub fn merge_change_sets(&self, change_sets: &[ChangeSet]) -> Result<ChangeSet> {
        if change_sets.is_empty() {
            return Err(Error::external_error("changes".to_string(), "No change sets to merge".to_string(), Span::unknown()));
        }

        // Determine the highest priority change type
        let merged_type = change_sets.iter().max_by(|a, b| Self::change_type_priority(a.r#type).cmp(&Self::change_type_priority(b.r#type))).unwrap().r#type.clone();

        // Merge summaries
        let merged_summary = change_sets.iter().map(|cs| cs.summary.clone()).collect::<Vec<_>>().join("; ");

        // Merge descriptions
        let merged_description = change_sets.iter().filter_map(|cs| cs.description.clone()).collect::<Vec<_>>().join("\n\n");

        // Merge packages (dedup)
        let mut merged_packages = HashMap::new();
        for cs in change_sets {
            for pkg in &cs.packages {
                merged_packages.insert(pkg.clone(), ());
            }
        }
        let merged_packages = merged_packages.keys().cloned().collect::<Vec<_>>();

        // Check if any change set is a prerelease
        let merged_prerelease = change_sets.iter().any(|cs| cs.prerelease);

        // Create the merged change set
        let merged_change_set = ChangeSet {
            id: format!("merged-{}", chrono::Utc::now().timestamp()),
            r#type: merged_type,
            summary: merged_summary,
            description: if merged_description.is_empty() { None } else { Some(merged_description) },
            author: None, // Merged change sets don't have a single author
            packages: merged_packages,
            prerelease: merged_prerelease,
        };

        Ok(merged_change_set)
    }

    /// Resolves conflicts between change sets.
    pub fn resolve_conflicts(&self, change_sets: &[ChangeSet]) -> Result<Vec<ChangeSet>> {
        // Simple conflict resolution: group by type and summary
        let mut resolved: HashMap<(ChangeType, String), ChangeSet> = HashMap::new();

        for cs in change_sets {
            let key = (cs.r#type.clone(), cs.summary.clone());
            if let Some(existing) = resolved.get_mut(&key) {
                // Merge packages
                let mut packages = existing.packages.clone();
                for pkg in &cs.packages {
                    if !packages.contains(pkg) {
                        packages.push(pkg.clone());
                    }
                }
                existing.packages = packages;

                // Merge descriptions
                if let Some(desc) = &cs.description {
                    if let Some(existing_desc) = &existing.description {
                        existing.description = Some(format!("{}\n\n{}", existing_desc, desc));
                    }
                    else {
                        existing.description = Some(desc.clone());
                    }
                }

                // Set prerelease if any is true
                if cs.prerelease {
                    existing.prerelease = true;
                }
            }
            else {
                resolved.insert(key, cs.clone());
            }
        }

        Ok(resolved.values().cloned().collect())
    }

    /// Gets the priority of a change type for merging.
    /// Higher priority change types override lower ones.
    fn change_type_priority(change_type: ChangeType) -> u8 {
        match change_type {
            ChangeType::Breaking => 10,
            ChangeType::Feature => 8,
            ChangeType::Fix => 6,
            ChangeType::Perf => 5,
            ChangeType::Refactor => 4,
            ChangeType::Docs => 3,
            ChangeType::Test => 2,
            ChangeType::Build => 1,
            ChangeType::Chore => 0,
        }
    }

    /// Loads a change set template.
    pub fn load_template(&self, template_name: &str) -> Result<ChangeSetTemplate> {
        let template_dir = self.changes_dir.join("templates");
        let template_path = template_dir.join(format!("{}.json", template_name));

        if !template_path.exists() {
            return Err(Error::external_error("changes".to_string(), format!("Template not found: {}", template_name), Span::unknown()));
        }

        let content = fs::read_to_string(template_path)?;
        let template: ChangeSetTemplate = serde_json::from_str(&content).map_err(|e| Error::external_error("json".to_string(), e.to_string(), Span::unknown()))?;

        Ok(template)
    }

    /// Saves a change set template.
    pub fn save_template(&self, template: &ChangeSetTemplate) -> Result<()> {
        let template_dir = self.changes_dir.join("templates");
        fs::create_dir_all(&template_dir)?;

        let template_path = template_dir.join(format!("{}.json", template.name));
        let content = serde_json::to_string_pretty(template).map_err(|e| Error::external_error("json".to_string(), e.to_string(), Span::unknown()))?;

        let mut file = File::create(template_path)?;
        file.write_all(content.as_bytes())?;

        Ok(())
    }

    /// Loads a change set preset.
    pub fn load_preset(&self, preset_name: &str) -> Result<ChangeSetPreset> {
        let preset_dir = self.changes_dir.join("presets");
        let preset_path = preset_dir.join(format!("{}.json", preset_name));

        if !preset_path.exists() {
            return Err(Error::external_error("changes".to_string(), format!("Preset not found: {}", preset_name), Span::unknown()));
        }

        let content = fs::read_to_string(preset_path)?;
        let preset: ChangeSetPreset = serde_json::from_str(&content).map_err(|e| Error::external_error("json".to_string(), e.to_string(), Span::unknown()))?;

        Ok(preset)
    }

    /// Saves a change set preset.
    pub fn save_preset(&self, preset: &ChangeSetPreset) -> Result<()> {
        let preset_dir = self.changes_dir.join("presets");
        fs::create_dir_all(&preset_dir)?;

        let preset_path = preset_dir.join(format!("{}.json", preset.name));
        let content = serde_json::to_string_pretty(preset).map_err(|e| Error::external_error("json".to_string(), e.to_string(), Span::unknown()))?;

        let mut file = File::create(preset_path)?;
        file.write_all(content.as_bytes())?;

        Ok(())
    }

    /// Creates a change set from a template.
    pub fn create_change_set_from_template(&self, template_name: &str, summary: &str, author: Option<String>) -> Result<PathBuf> {
        let template = self.load_template(template_name)?;

        let change_set = ChangeSet { id: format!("{}", chrono::Utc::now().timestamp()), r#type: template.default_type, summary: summary.to_string(), description: template.description_template, author, packages: template.default_packages, prerelease: template.default_prerelease };

        self.create_change_set(&change_set)
    }

    /// Creates a change set from a preset.
    pub fn create_change_set_from_preset(&self, preset_name: &str, summary: &str, author: Option<String>) -> Result<PathBuf> {
        let preset = self.load_preset(preset_name)?;
        self.create_change_set_from_template(&preset.template, summary, author)
    }

    /// Generates an enhanced changelog with more details.
    pub fn generate_enhanced_changelog(&self, version: &str, date: &str, include_authors: bool) -> Result<String> {
        let change_sets = self.read_change_sets()?;
        let mut changelog = format!("# Changelog\n\n## [{}] - {}\n\n", version, date);

        // Group change sets by type
        let mut grouped = HashMap::new();
        for cs in &change_sets {
            grouped.entry(cs.r#type.clone()).or_insert_with(Vec::new).push(cs);
        }

        // Define the order of change types
        let type_order = [ChangeType::Breaking, ChangeType::Feature, ChangeType::Fix, ChangeType::Perf, ChangeType::Refactor, ChangeType::Docs, ChangeType::Test, ChangeType::Build, ChangeType::Chore];

        // Add changes in order
        for change_type in &type_order {
            if let Some(cs_list) = grouped.get(change_type) {
                if !cs_list.is_empty() {
                    // Add section header
                    let section_title = match change_type {
                        ChangeType::Breaking => "Breaking Changes",
                        ChangeType::Feature => "Features",
                        ChangeType::Fix => "Bug Fixes",
                        ChangeType::Perf => "Performance Improvements",
                        ChangeType::Refactor => "Code Refactoring",
                        ChangeType::Docs => "Documentation",
                        ChangeType::Test => "Tests",
                        ChangeType::Build => "Build System",
                        ChangeType::Chore => "Chores",
                    };
                    changelog.push_str(&format!("### {}\n\n", section_title));

                    // Add each change
                    for cs in cs_list {
                        changelog.push_str(&format!("- {}\n", cs.summary));
                        if let Some(desc) = &cs.description {
                            changelog.push_str(&format!("  {}\n", desc));
                        }
                        if include_authors && cs.author.is_some() {
                            changelog.push_str(&format!("  **Author:** {}\n", cs.author.as_ref().unwrap()));
                        }
                        if !cs.packages.is_empty() {
                            changelog.push_str(&format!("  **Packages:** {}\n", cs.packages.join(", ")));
                        }
                        changelog.push_str("\n");
                    }
                }
            }
        }

        Ok(changelog)
    }
}
