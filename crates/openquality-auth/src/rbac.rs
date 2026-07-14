use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Role {
    Owner,
    Admin,
    Editor,
    Member,
    Viewer,
}

impl std::str::FromStr for Role {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "owner" => Ok(Self::Owner),
            "admin" => Ok(Self::Admin),
            "editor" => Ok(Self::Editor),
            "member" => Ok(Self::Member),
            "viewer" => Ok(Self::Viewer),
            _ => Err(format!("invalid role: {s}")),
        }
    }
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Admin => "admin",
            Self::Editor => "editor",
            Self::Member => "member",
            Self::Viewer => "viewer",
        }
    }

    pub fn can(&self, action: &str) -> bool {
        let perms = self.permissions();
        perms.contains(action)
    }

    pub fn permissions(&self) -> HashSet<&'static str> {
        let mut perms = HashSet::new();
        match self {
            Self::Viewer => {
                perms.insert("read:workspace");
                perms.insert("read:monitor");
                perms.insert("read:incident");
                perms.insert("read:suite");
                perms.insert("read:datasource");
                perms.insert("read:profile");
            }
            Self::Member => {
                perms.extend(Self::Viewer.permissions());
                perms.insert("read:user");
            }
            Self::Editor => {
                perms.extend(Self::Member.permissions());
                perms.insert("write:monitor");
                perms.insert("write:incident");
                perms.insert("write:suite");
                perms.insert("write:expectation");
                perms.insert("run:suite");
                perms.insert("run:monitor");
                perms.insert("write:datasource");
            }
            Self::Admin => {
                perms.extend(Self::Editor.permissions());
                perms.insert("write:user");
                perms.insert("write:apikey");
                perms.insert("delete:monitor");
                perms.insert("delete:suite");
                perms.insert("delete:datasource");
                perms.insert("admin:workspace");
            }
            Self::Owner => {
                perms.extend(Self::Admin.permissions());
                perms.insert("delete:workspace");
                perms.insert("admin:billing");
            }
        }
        perms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_hierarchy() {
        assert!(Role::Viewer.can("read:monitor"));
        assert!(!Role::Viewer.can("write:monitor"));
        assert!(Role::Editor.can("write:monitor"));
        assert!(Role::Admin.can("write:user"));
        assert!(!Role::Editor.can("write:user"));
    }

    #[test]
    fn test_role_from_str() {
        use std::str::FromStr;
        assert_eq!(Role::from_str("admin").unwrap(), Role::Admin);
        assert_eq!(Role::from_str("VIEWER").unwrap(), Role::Viewer);
        assert!(Role::from_str("unknown").is_err());
    }
}
