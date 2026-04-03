use std::collections::HashSet;

/// Permission check types for flexible access control.
#[derive(Debug, Clone)]
pub enum PermissionsCheck {
    /// User needs any one of the permissions (OR logic).
    Any(Vec<&'static str>),
    /// User needs all permissions (AND logic).
    All(Vec<&'static str>),
    /// User needs this specific permission.
    Single(&'static str),
}

impl PermissionsCheck {
    /// Core permission validation logic.
    pub fn check(&self, user_permissions: &HashSet<String>) -> bool {
        if user_permissions.contains("*") {
            return true;
        }
        match self {
            PermissionsCheck::Single(code) => user_permissions.contains(*code),
            PermissionsCheck::Any(codes) => {
                codes.iter().any(|code| user_permissions.contains(*code))
            }
            PermissionsCheck::All(codes) => {
                codes.iter().all(|code| user_permissions.contains(*code))
            }
        }
    }

    /// Returns a description of the permission check for logging.
    pub fn description(&self) -> String {
        match self {
            PermissionsCheck::Single(p) => format!("single permission '{}'", p),
            PermissionsCheck::Any(ps) => format!("any of permissions {:?}", ps),
            PermissionsCheck::All(ps) => format!("all permissions {:?}", ps),
        }
    }
}

