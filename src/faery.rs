use serde::{Deserialize, Serialize};
use crate::dross::{DrossError, DrossHolder, DrossResult};
use crate::sql::SqlModel;

// Faery represents the user of the application.
// It has the name of the user, their email, an authentication token, and a count of their dross.
#[derive(Debug, Deserialize, Serialize)]
pub struct Faery {
    pub name: String,
    pub email: String,
    pub is_admin: bool,
    pub auth_token: Option<String>,
    pub dross: u32,
}

impl Faery {
    // This is a method that creates a new Faery.
    // It takes a name and an email and returns a Faery.
    pub fn new(name: String, email: String) -> Faery {
        Faery {
            name,
            email,
            is_admin: false,
            auth_token: None,
            dross: 0,
        }
    }

    // This is a method that creates a new admin Faery.
    // It takes a name and an email and returns a Faery.
    pub fn new_admin(name: String, email: String) -> Faery {
        Faery {
            name,
            email,
            is_admin: true,
            auth_token: None,
            dross: 0,
        }
    }

    // This is a method that returns the name of the Faery.
    pub fn name(&self) -> &str {
        &self.name
    }

    // This is a method that returns the email of the Faery.
    pub fn email(&self) -> &str {
        &self.email
    }

    // This is a method that returns whether the Faery is an admin.
    pub fn is_admin(&self) -> bool {
        self.is_admin
    }

    // This is a method that returns the dross of the Faery.
    pub fn dross(&self) -> u32 {
        self.dross
    }

    // This is a method that returns the auth token of the Faery.
    pub fn auth_token(&self) -> Option<&str> {
        self.auth_token.as_deref()
    }

    // This is a method that sets the auth token of the Faery.
    pub fn set_auth_token(&mut self, auth_token: String) {
        self.auth_token = Some(auth_token);
    }
}

impl SqlModel for Faery {
    fn to_sql_insert(&self) -> String {
        format!(
            "INSERT INTO faeries (name, is_admin, email, dross) VALUES ('{}', {}, '{}', {})",
            self.name,
            self.is_admin,
            self.email,
            self.dross
        )
    }

    fn generate_sql_create_table() -> String {
        "CREATE TABLE IF NOT EXISTS faeries (
            id INTEGER PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            is_admin BOOLEAN NOT NULL,
            email VARCHAR(255) NOT NULL,
            dross INTEGER
        )".to_string()
    }

    fn table_name() -> String {
        "faeries".to_string()
    }
}

impl DrossHolder for Faery {
    // This is a method that increments the dross of the Faery.
    fn increment_dross(&mut self, amount: u32) -> DrossResult {
        if amount <= 0 {
            return Err(DrossError::InvalidIncrement);
        }
        self.dross += amount;
        Ok(self.dross)
    }

    // This is a method that decrements the dross of the Faery.
    fn decrement_dross(&mut self, amount: u32) -> DrossResult {
        match amount {
            0 => Err(DrossError::InvalidDecrement),
            _ if amount > self.dross => Err(DrossError::NotEnoughDross),
            _ => {
                self.dross -= amount;
                Ok(self.dross)
            }
        }
    }

    // This is a method that returns the dross of the Faery.
    fn dross(&self) -> DrossResult {
        Ok(self.dross)
    }
}
