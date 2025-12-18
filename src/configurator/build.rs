use crate::configurator::parser::Configuration;

pub fn build_plan(configuration: Configuration) -> Plan {
    let mut plan = Plan { year_plans: vec![] };
    let config = &configuration;

    let mut empty_scores = serde_json::json!({});

    for form in config.forms.iter() {
        empty_scores[form.id.clone()] = 0.into();
    }

    let empty_scores = empty_scores.to_string();

    for year in config.years.iter() {
        let year_id = year.id.clone();
        let year_name = year.name.clone();

        let mut year_plan = YearPlan {
            id: year_id.clone(),
            name: year_name,
            events: vec![],
        };

        for event in config.events.iter() {
            if !configuration.is_event_applicable_to_year(event, &year.clone().id) {
                continue;
            }
            for gender in config.genders.iter() {
                if configuration.is_event_applicable_to_gender(event, gender) {
                    year_plan.events.push(EventPlan {
                        id: format!("{}-{}-{}", year_plan.clone().id, gender, event.clone().id),
                        name: event.clone().name,
                        gender_id: gender.clone(),
                        filter_key: event.clone().id,
                        scores: empty_scores.clone(),
                    })
                }
            }
        }
        plan.year_plans.push(year_plan);
    }
    plan
}

#[derive(Debug)]
pub struct Plan {
    pub year_plans: Vec<YearPlan>,
}

#[derive(Debug, Clone)]

pub struct YearPlan {
    pub id: String,
    pub name: String,
    pub events: Vec<EventPlan>,
}

#[derive(Debug, Clone)]

pub struct EventPlan {
    pub id: String,
    pub name: String,
    pub gender_id: String,
    pub filter_key: String,
    pub scores: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configurator::parser::{ApplicabilityRules, Event, Form, Score, Year};

    #[test]
    fn test_build_plan_empty_config() {
        let config = Configuration {
            version: "1.0.0".to_string(),
            genders: vec![],
            scores: vec![],
            years: vec![],
            forms: vec![],
            events: vec![],
        };

        let plan = build_plan(config);
        assert_eq!(plan.year_plans.len(), 0);
    }

    #[test]
    fn test_build_plan_single_year_no_events() {
        let config = Configuration {
            version: "1.0.0".to_string(),
            genders: vec!["mixed".to_string()],
            scores: vec![],
            years: vec![Year {
                id: "year7".to_string(),
                name: "Year 7".to_string(),
            }],
            forms: vec![],
            events: vec![],
        };

        let plan = build_plan(config);
        assert_eq!(plan.year_plans.len(), 1);
        assert_eq!(plan.year_plans[0].id, "year7");
        assert_eq!(plan.year_plans[0].events.len(), 0);
    }

    #[test]
    fn test_build_plan_with_events() {
        let config = Configuration {
            version: "1.0.0".to_string(),
            genders: vec!["mixed".to_string()],
            scores: vec![Score {
                name: "1st".to_string(),
                value: 10,
                default: true,
            }],
            years: vec![Year {
                id: "year7".to_string(),
                name: "Year 7".to_string(),
            }],
            forms: vec![Form {
                id: "form1".to_string(),
                name: "Form 1".to_string(),
                colour: "#ff0000".to_string(),
            }],
            events: vec![Event {
                id: "event1".to_string(),
                name: "Event 1".to_string(),
                applicable_years: ApplicabilityRules::All,
                applicable_genders: ApplicabilityRules::All,
            }],
        };

        let plan = build_plan(config);
        assert_eq!(plan.year_plans.len(), 1);
        assert_eq!(plan.year_plans[0].events.len(), 1);
        assert_eq!(plan.year_plans[0].events[0].name, "Event 1");
        assert_eq!(plan.year_plans[0].events[0].gender_id, "mixed");
    }

    #[test]
    fn test_build_plan_multiple_genders() {
        let config = Configuration {
            version: "1.0.0".to_string(),
            genders: vec!["boys".to_string(), "girls".to_string()],
            scores: vec![],
            years: vec![Year {
                id: "year7".to_string(),
                name: "Year 7".to_string(),
            }],
            forms: vec![],
            events: vec![Event {
                id: "event1".to_string(),
                name: "Event 1".to_string(),
                applicable_years: ApplicabilityRules::All,
                applicable_genders: ApplicabilityRules::All,
            }],
        };

        let plan = build_plan(config);
        assert_eq!(plan.year_plans[0].events.len(), 2);
        assert_eq!(plan.year_plans[0].events[0].gender_id, "boys");
        assert_eq!(plan.year_plans[0].events[1].gender_id, "girls");
    }

    #[test]
    fn test_build_plan_event_not_applicable_to_year() {
        let config = Configuration {
            version: "1.0.0".to_string(),
            genders: vec!["mixed".to_string()],
            scores: vec![],
            years: vec![
                Year {
                    id: "year7".to_string(),
                    name: "Year 7".to_string(),
                },
                Year {
                    id: "year8".to_string(),
                    name: "Year 8".to_string(),
                },
            ],
            forms: vec![],
            events: vec![Event {
                id: "event1".to_string(),
                name: "Event 1".to_string(),
                applicable_years: ApplicabilityRules::Include {
                    ids: vec!["year7".to_string()],
                },
                applicable_genders: ApplicabilityRules::All,
            }],
        };

        let plan = build_plan(config);
        assert_eq!(plan.year_plans.len(), 2);
        assert_eq!(plan.year_plans[0].events.len(), 1);
        assert_eq!(plan.year_plans[1].events.len(), 0);
    }

    #[test]
    fn test_build_plan_event_not_applicable_to_gender() {
        let config = Configuration {
            version: "1.0.0".to_string(),
            genders: vec!["boys".to_string(), "girls".to_string(), "mixed".to_string()],
            scores: vec![],
            years: vec![Year {
                id: "year7".to_string(),
                name: "Year 7".to_string(),
            }],
            forms: vec![],
            events: vec![Event {
                id: "event1".to_string(),
                name: "Event 1".to_string(),
                applicable_years: ApplicabilityRules::All,
                applicable_genders: ApplicabilityRules::Include {
                    ids: vec!["boys".to_string()],
                },
            }],
        };

        let plan = build_plan(config);
        assert_eq!(plan.year_plans[0].events.len(), 1);
        assert_eq!(plan.year_plans[0].events[0].gender_id, "boys");
    }

    #[test]
    fn test_build_plan_event_id_format() {
        let config = Configuration {
            version: "1.0.0".to_string(),
            genders: vec!["mixed".to_string()],
            scores: vec![],
            years: vec![Year {
                id: "year7".to_string(),
                name: "Year 7".to_string(),
            }],
            forms: vec![],
            events: vec![Event {
                id: "event1".to_string(),
                name: "Event 1".to_string(),
                applicable_years: ApplicabilityRules::All,
                applicable_genders: ApplicabilityRules::All,
            }],
        };

        let plan = build_plan(config);
        assert_eq!(plan.year_plans[0].events[0].id, "year7-mixed-event1");
    }

    #[test]
    fn test_build_plan_scores_initialization() {
        let config = Configuration {
            version: "1.0.0".to_string(),
            genders: vec!["mixed".to_string()],
            scores: vec![],
            years: vec![Year {
                id: "year7".to_string(),
                name: "Year 7".to_string(),
            }],
            forms: vec![
                Form {
                    id: "form1".to_string(),
                    name: "Form 1".to_string(),
                    colour: "#ff0000".to_string(),
                },
                Form {
                    id: "form2".to_string(),
                    name: "Form 2".to_string(),
                    colour: "#00ff00".to_string(),
                },
            ],
            events: vec![Event {
                id: "event1".to_string(),
                name: "Event 1".to_string(),
                applicable_years: ApplicabilityRules::All,
                applicable_genders: ApplicabilityRules::All,
            }],
        };

        let plan = build_plan(config);
        let scores = &plan.year_plans[0].events[0].scores;
        assert!(scores.contains("form1"));
        assert!(scores.contains("form2"));
        assert!(scores.contains(":0"));
    }
}
