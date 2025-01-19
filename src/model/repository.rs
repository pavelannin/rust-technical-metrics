use serde_json::{from_str, Value};
use std::error::Error;
use std::fs;
use indexmap::IndexMap;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Repository {
    pub name: String,
    pub ssh: String,
    pub branch: String,
    pub owner: String,
}

// New
impl Repository {
    pub fn from_config(path: &str) -> crate::model::user::Result<Vec<Self>> {
        let json_str = fs::read_to_string(path)?;
        Self::parse(&json_str)
    }

    fn new(
        name: impl ToString,
        ssh: impl ToString,
        branch: impl ToString,
        owner: impl ToString,
    ) -> Self {
        Self {
            name: name.to_string(),
            ssh: ssh.to_string(),
            branch: branch.to_string(),
            owner: owner.to_string(),
        }
    }
}

// Parser
impl Repository {
    fn parse(json_str: &str) -> crate::model::Result<Vec<Self>> {
        let elements: IndexMap<String, Value> = from_str(json_str)?;
        let mut result = Vec::new();
        for (name, details) in elements {
            let Some(ssh) = details["ssh"].as_str() else {
                return Err("Not fond 'ssh' field".into());
            };
            let Some(branch) = details["branch"].as_str() else {
                return Err("Not fond 'branch' field".into());
            };
            let Some(owner) = details["owner"].as_str() else {
                return Err("Not fond 'owner' field".into());
            };
            let new = Self::new(name, ssh, branch, owner);
            result.push(new);
        }
        Ok(result)
    }
}
