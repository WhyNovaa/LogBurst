use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Role {
    User,
    Admin,
}
impl Display for Role {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            Role::User => "User",
            Role::Admin => "Admin",
        };

        write!(f, "{}", res)
    }
}

impl From<String> for Role {
    // if role name is wrong, role will be 'User'
    fn from(value: String) -> Self {
        match value.as_str() {
            "Admin" => Role::Admin,
            _ => Role::User,
        }
    }
}
impl Role {
    pub fn id(&self) -> i32 {
        match self {
            Role::User => 1,
            Role::Admin => 2,
        }
    }
}