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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_applicability_rules_all() {
        let rules = ApplicabilityRules::All;
        let yaml = serde_yml::to_string(&rules).unwrap();
        assert!(yaml.contains("all"));
    }

    #[test]
    fn test_applicability_rules_none() {
        let rules = ApplicabilityRules::None;
        let yaml = serde_yml::to_string(&rules).unwrap();
        assert!(yaml.contains("none"));
    }

    #[test]
    fn test_applicability_rules_include() {
        let rules = ApplicabilityRules::Include {
            ids: vec!["year7".to_string(), "year8".to_string()],
        };
        let yaml = serde_yml::to_string(&rules).unwrap();
        assert!(yaml.contains("include"));
        assert!(yaml.contains("year7"));
    }

    #[test]
    fn test_applicability_rules_exclude() {
        let rules = ApplicabilityRules::Exclude {
            ids: vec!["year7".to_string()],
        };
        let yaml = serde_yml::to_string(&rules).unwrap();
        assert!(yaml.contains("exclude"));
        assert!(yaml.contains("year7"));
    }

    #[test]
    fn test_configuration_is_event_applicable_to_year_all() {
        let config = Configuration {
            version: "1.0.0".to_string(),
            genders: vec![],
            scores: vec![],
            years: vec![],
            forms: vec![],
            events: vec![],
        };

        let event = Event {
            id: "test".to_string(),
            name: "Test".to_string(),
            applicable_years: ApplicabilityRules::All,
            applicable_genders: ApplicabilityRules::All,
        };

        assert!(config.is_event_applicable_to_year(&event, "year7"));
        assert!(config.is_event_applicable_to_year(&event, "year8"));
    }

    #[test]
    fn test_configuration_is_event_applicable_to_year_none() {
        let config = Configuration {
            version: "1.0.0".to_string(),
            genders: vec![],
            scores: vec![],
            years: vec![],
            forms: vec![],
            events: vec![],
        };

        let event = Event {
            id: "test".to_string(),
            name: "Test".to_string(),
            applicable_years: ApplicabilityRules::None,
            applicable_genders: ApplicabilityRules::All,
        };

        assert!(!config.is_event_applicable_to_year(&event, "year7"));
    }

    #[test]
    fn test_configuration_is_event_applicable_to_year_include() {
        let config = Configuration {
            version: "1.0.0".to_string(),
            genders: vec![],
            scores: vec![],
            years: vec![],
            forms: vec![],
            events: vec![],
        };

        let event = Event {
            id: "test".to_string(),
            name: "Test".to_string(),
            applicable_years: ApplicabilityRules::Include {
                ids: vec!["year7".to_string(), "year8".to_string()],
            },
            applicable_genders: ApplicabilityRules::All,
        };

        assert!(config.is_event_applicable_to_year(&event, "year7"));
        assert!(config.is_event_applicable_to_year(&event, "year8"));
        assert!(!config.is_event_applicable_to_year(&event, "year9"));
    }

    #[test]
    fn test_configuration_is_event_applicable_to_year_exclude() {
        let config = Configuration {
            version: "1.0.0".to_string(),
            genders: vec![],
            scores: vec![],
            years: vec![],
            forms: vec![],
            events: vec![],
        };

        let event = Event {
            id: "test".to_string(),
            name: "Test".to_string(),
            applicable_years: ApplicabilityRules::Exclude {
                ids: vec!["year7".to_string()],
            },
            applicable_genders: ApplicabilityRules::All,
        };

        assert!(!config.is_event_applicable_to_year(&event, "year7"));
        assert!(config.is_event_applicable_to_year(&event, "year8"));
        assert!(config.is_event_applicable_to_year(&event, "year9"));
    }

    #[test]
    fn test_configuration_is_event_applicable_to_gender_all() {
        let config = Configuration {
            version: "1.0.0".to_string(),
            genders: vec![],
            scores: vec![],
            years: vec![],
            forms: vec![],
            events: vec![],
        };

        let event = Event {
            id: "test".to_string(),
            name: "Test".to_string(),
            applicable_years: ApplicabilityRules::All,
            applicable_genders: ApplicabilityRules::All,
        };

        assert!(config.is_event_applicable_to_gender(&event, "boys"));
        assert!(config.is_event_applicable_to_gender(&event, "girls"));
        assert!(config.is_event_applicable_to_gender(&event, "mixed"));
    }

    #[test]
    fn test_configuration_is_event_applicable_to_gender_include() {
        let config = Configuration {
            version: "1.0.0".to_string(),
            genders: vec![],
            scores: vec![],
            years: vec![],
            forms: vec![],
            events: vec![],
        };

        let event = Event {
            id: "test".to_string(),
            name: "Test".to_string(),
            applicable_years: ApplicabilityRules::All,
            applicable_genders: ApplicabilityRules::Include {
                ids: vec!["boys".to_string()],
            },
        };

        assert!(config.is_event_applicable_to_gender(&event, "boys"));
        assert!(!config.is_event_applicable_to_gender(&event, "girls"));
    }

    #[test]
    fn test_configuration_get_version() {
        let config = Configuration {
            version: "2.5.3".to_string(),
            genders: vec![],
            scores: vec![],
            years: vec![],
            forms: vec![],
            events: vec![],
        };

        assert_eq!(config.get_version(), "2.5.3");
    }

    #[test]
    fn test_configuration_from_yaml_file() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let yaml_content = "version: \"1.0.0\"\ngenders:\n  - boys\n  - girls\n  - mixed\nscores:\n  - name: \"1st\"\n    value: 10\n    default: true\n  - name: \"2nd\"\n    value: 8\n    default: false\nyears:\n  - id: \"year7\"\n    name: \"Year 7\"\nforms:\n  - id: \"form1\"\n    name: \"Form 1\"\n    colour: \"#ff0000\"\nevents:\n  - id: \"event1\"\n    name: \"Event 1\"\n    applicable_years:\n      type: all\n    applicable_genders:\n      type: all\n";

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(yaml_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config = Configuration::from_yaml_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(config.version, "1.0.0");
        assert_eq!(config.genders.len(), 3);
        assert_eq!(config.scores.len(), 2);
        assert_eq!(config.years.len(), 1);
        assert_eq!(config.forms.len(), 1);
        assert_eq!(config.events.len(), 1);
        assert_eq!(config.years[0].id, "year7");
        assert_eq!(config.forms[0].name, "Form 1");
    }

    #[test]
    fn test_configuration_from_yaml_file_not_found() {
        let result = Configuration::from_yaml_file("nonexistent.yaml");
        assert!(result.is_err());
    }
}
