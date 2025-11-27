use serde::{Deserialize, Serialize};

/// Main configuration structure containing all years, forms, and events
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Configuration {
    /// Config Version
    pub version: String,
    /// Genders for Events
    pub genders: Vec<String>,
    // The Scoring System
    pub scores: Vec<Score>,
    /// All available years in the system
    pub years: Vec<Year>,
    /// All available forms/classes in the system  
    pub forms: Vec<Form>,
    /// All available events with their applicability rules
    pub events: Vec<Event>,
}

/// Represents a school year
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Year {
    /// Unique identifier for the year (e.g., "2024", "2025")
    pub id: String,
    /// Human-readable name (e.g., "Academic Year 2024-2025")
    pub name: String,
}

/// Represents a form/class level
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Form {
    /// Unique identifier (e.g., "year7", "year8", "reception")
    pub id: String,
    /// Display name (e.g., "Year 7", "Reception")
    pub name: String,
    /// Colour (lightgreen, #fdfd80, rgb(249, 164, 164))
    pub colour: String,
}

/// Represents a sports event with flexible year/form applicability
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    /// Unique identifier for the event
    pub id: String,
    /// Display name of the event
    pub name: String,
    /// Rules for which years this event applies to
    pub applicable_years: ApplicabilityRules,
    /// Rules for which gender this event applies to
    pub applicable_genders: ApplicabilityRules,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Score {
    pub name: String,
    pub value: i64,
    pub default: bool,
}

/// Flexible rules for determining applicability
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum ApplicabilityRules {
    /// Apply to all years/forms
    #[serde(rename = "all")]
    All,
    /// Apply to none (event disabled)
    #[serde(rename = "none")]
    None,
    /// Apply only to specific IDs
    #[serde(rename = "include")]
    Include { ids: Vec<String> },
    /// Apply to all except specific IDs
    #[serde(rename = "exclude")]
    Exclude { ids: Vec<String> },
}

impl Configuration {
    /// Load configuration from YAML file
    pub fn from_yaml_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Configuration = serde_yml::from_str(&content)?;
        Ok(config)
    }

    /// Check if an event applies to a specific year
    pub fn is_event_applicable_to_year(&self, event: &Event, year_id: &str) -> bool {
        match &event.applicable_years {
            ApplicabilityRules::All => true,
            ApplicabilityRules::None => false,
            ApplicabilityRules::Include { ids } => ids.contains(&year_id.to_string()),
            ApplicabilityRules::Exclude { ids } => !ids.contains(&year_id.to_string()),
        }
    }

    /// Check if an event applies to a specific gender
    pub fn is_event_applicable_to_gender(&self, event: &Event, gender_id: &str) -> bool {
        match &event.applicable_genders {
            ApplicabilityRules::All => true,
            ApplicabilityRules::None => false,
            ApplicabilityRules::Include { ids } => ids.contains(&gender_id.to_string()),
            ApplicabilityRules::Exclude { ids } => !ids.contains(&gender_id.to_string()),
        }
    }

    /// Get Schema Version
    pub fn get_version(&self) -> String {
        self.version.clone()
    }
}
