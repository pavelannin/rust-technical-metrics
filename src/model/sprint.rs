use std::error::Error;
use std::fs;
use chrono::{DateTime, FixedOffset};
use indexmap::IndexMap;
use serde_json::{from_str, Value};

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Sprint {
    pub name: String,
    pub since: DateTime<FixedOffset>,
    pub until: DateTime<FixedOffset>,
}

// Create
impl Sprint {
    pub fn from_config(path: &str) -> Result<Vec<Self>> {
        let json_str = fs::read_to_string(path)?;
        Self::parse(&json_str)
    }

    fn new(
        name: impl ToString,
        since: &DateTime<FixedOffset>,
        until: &DateTime<FixedOffset>
    ) -> Sprint {
        Self {
            name: name.to_string(),
            since: since.clone(),
            until: until.clone()
        }
    }
}

// Parser
impl Sprint {
    fn parse(json_str: &str) -> crate::model::Result<Vec<Self>> {
        let elements: IndexMap<String, Value> = from_str(json_str)?;
        let mut result = Vec::new();
        for (name, details) in elements {
            let Some(since) = details["since"].as_str() else {
                return Err("Not fond 'since' field".into());
            };
            let Ok(since) = DateTime::parse_from_rfc3339(since) else {
                return Err(format!("Not a valid date time: {}", since).into());
            };
            let Some(until) = details["until"].as_str() else {
                return Err("Not fond 'until' field".into());
            };
            let Ok(until) = DateTime::parse_from_rfc3339(until) else {
                return Err(format!("Not a valid date time: {}", until).into());
            };
            let new = Self::new(name, &since, &until);
            result.push(new);
        }
        Ok(result)
    }
}

