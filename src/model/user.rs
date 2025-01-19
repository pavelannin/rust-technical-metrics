use serde_json::{from_str, Value};
use std::error::Error;
use std::fs;
use indexmap::IndexMap;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct User {
    pub username: String,
    pub avatar_url: String,
    pub role: String,
    pub teams: Vec<String>,
    pub emails: Vec<String>,
}

// Create
impl User {
    pub fn from_config(path: &str) -> Result<Vec<Self>> {
        let json_str = fs::read_to_string(path)?;
        Self::parse(&json_str)
    }

    fn new(
        username: impl ToString,
        avatar_url: impl ToString,
        role: impl ToString,
        teams: Vec<impl ToString>,
        emails: Vec<impl ToString>,
    ) -> Self {
        Self {
            username: username.to_string(),
            avatar_url: avatar_url.to_string(),
            role: role.to_string(),
            teams: teams.iter().clone().map(|t| t.to_string()).collect(),
            emails: emails.iter().clone().map(|t| t.to_string()).collect(),
        }
    }
}

// Parser
impl User {
    fn parse(json_str: &str) -> crate::model::Result<Vec<Self>> {
        let elements: IndexMap<String, Value> = from_str(json_str)?;
        let mut result = Vec::new();
        for (name, details) in elements {
            let Some(avatar_url) = details["avatarUrl"].as_str() else {
                return Err("Not fond 'avatarUrl' field".into());
            };
            let Some(role) = details["role"].as_str() else {
                return Err("Not fond 'role' field".into());
            };
            let teams = match details["teams"].as_array() {
                Some(map) => map
                    .iter()
                    .filter_map(|team| team.as_str().map(String::from))
                    .collect(),
                None => return Err("Not fond 'teams' field".into()),
            };
            let emails = match details["emails"].as_array() {
                Some(map) => map
                    .iter()
                    .filter_map(|email| email.as_str().map(String::from))
                    .collect(),
                None => return Err("Not fond 'emails' field".into()),
            };
            let new = Self::new(name, avatar_url, role, teams, emails);
            result.push(new);
        }
        Ok(result)
    }
}
