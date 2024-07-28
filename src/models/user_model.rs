use chrono::{DateTime, Utc};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum UserRole {
    #[serde(rename = "ADMIN")]
    Admin,
    #[serde(rename = "USER")]
    User,
}

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserRole::Admin => write!(f, "ADMIN"),
            UserRole::User => write!(f, "USER"),
        }
    }
}

impl FromStr for UserRole {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ADMIN" => Ok(UserRole::Admin),
            "USER" => Ok(UserRole::User),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub role: UserRole,
    pub api_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

impl User {
    pub fn new(name: &str, role: &UserRole, created_by: &str) -> Self {
        User {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            role: role.clone(),
            api_key: Some(generate_api_key(32)),
            created_at: Utc::now(),
            created_by: created_by.to_string(),
        }
    }
}

fn generate_api_key(length: usize) -> String {
    let api_key: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect();
    api_key
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn generate_api_key_with_length_32() {
        let api_key = generate_api_key(32);
        assert_eq!(api_key.len(), 32);
    }

    #[test]
    fn every_api_key_is_unique() {
        let mut unique_keys = HashSet::with_capacity(100);

        for _ in 0..100 {
            let api_key = generate_api_key(32);
            assert!(unique_keys.insert(api_key), "Duplicate API key found");
        }

        // Verify that the total number of unique keys is 100
        assert_eq!(unique_keys.len(), 100);
    }

    #[test]
    fn create_with_role_user() {
        let name = "test user";
        let role = UserRole::User;
        let created_by = "Some admin user";

        let user = User::new(name, &role, created_by);

        assert!(
            Uuid::from_str(&user.id).is_ok(),
            "Expected id to be a valid UUID v4, but was not."
        );
        assert_eq!(
            user.name, name,
            "Expected name to be '{}', but was '{}'.",
            name, user.name
        );
        assert_eq!(
            user.role,
            UserRole::User,
            "Expected role to be 'User', but was '{:?}'.",
            user.role
        );
        assert!(
            user.api_key.is_some(),
            "Expected the api_key to set but was None"
        );

        let api_key = user.api_key.unwrap();
        assert_eq!(
            api_key.len(),
            32,
            "Expected API key length to be 32, but was {}.",
            api_key.len()
        );
        assert_eq!(
            user.created_by, created_by,
            "Expected created_by to be '{}', but was '{}'.",
            created_by, user.created_by
        );
    }

    #[test]
    fn create_with_role_admin() {
        let name = "test user";
        let role = UserRole::Admin;
        let created_by = "Some admin user";

        let user = User::new(name, &role, created_by);

        assert!(
            Uuid::from_str(&user.id).is_ok(),
            "Expected id to be a valid UUID v4, but was not."
        );
        assert_eq!(
            user.name, name,
            "Expected name to be '{}', but was '{}'.",
            name, user.name
        );
        assert_eq!(
            user.role,
            UserRole::Admin,
            "Expected role to be 'Admin', but was '{:?}'.",
            user.role
        );
        assert!(
            user.api_key.is_some(),
            "Expected the api_key to set but was None"
        );

        let api_key = user.api_key.unwrap();
        assert_eq!(
            api_key.len(),
            32,
            "Expected API key length to be 32, but was {}.",
            api_key.len()
        );
        assert_eq!(
            user.created_by, created_by,
            "Expected created_by to be '{}', but was '{}'.",
            user.created_by, created_by
        );
    }
}
