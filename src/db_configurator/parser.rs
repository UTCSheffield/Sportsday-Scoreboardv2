use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Main configuration structure containing all years, forms, and events
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DBConfiguration {
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
    /// Rules for which forms this event applies to
    pub applicable_forms: ApplicabilityRules,
    /// Rules for which gender this event applies to
    pub applicable_genders: ApplicabilityRules,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Score {
    name: String,
    value: i64,
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

impl DBConfiguration {
    /// Load configuration from YAML file
    pub fn from_yaml_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: DBConfiguration = serde_yml::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to YAML file
    pub fn to_yaml_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_yml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get events applicable to a specific year and form
    pub fn applicable_events(&self, year_id: &str, form_id: &str) -> Vec<&Event> {
        self.events
            .iter()
            .filter(|event| {
                self.is_event_applicable_to_year(event, year_id)
                    && self.is_event_applicable_to_form(event, form_id)
            })
            .collect()
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

    /// Check if an event applies to a specific form
    pub fn is_event_applicable_to_form(&self, event: &Event, form_id: &str) -> bool {
        match &event.applicable_forms {
            ApplicabilityRules::All => true,
            ApplicabilityRules::None => false,
            ApplicabilityRules::Include { ids } => ids.contains(&form_id.to_string()),
            ApplicabilityRules::Exclude { ids } => !ids.contains(&form_id.to_string()),
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

    /// Find a year by ID
    pub fn find_year(&self, id: &str) -> Option<&Year> {
        self.years.iter().find(|year| year.id == id)
    }

    /// Find a form by ID  
    pub fn find_form(&self, id: &str) -> Option<&Form> {
        self.forms.iter().find(|form| form.id == id)
    }

    /// Find an event by ID
    pub fn find_event(&self, id: &str) -> Option<&Event> {
        self.events.iter().find(|event| event.id == id)
    }

    /// Get Config Version
    pub fn get_version(&self) -> String {
        self.version.clone()
    }

    /// Get Scoring System
    pub fn get_scores(&mut self) -> HashMap<String, i64> {
        let mut map: HashMap<String, i64> = HashMap::new();
        self.scores.sort_by(|a, b| a.value.cmp(&b.value));
        for score in self.scores.clone() {
            map.insert(score.name, score.value);
        }
        map
    }
}
