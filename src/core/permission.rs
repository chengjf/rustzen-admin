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

#[cfg(test)]
mod tests {
    use super::*;

    fn perms(codes: &[&str]) -> HashSet<String> {
        codes.iter().map(|s| s.to_string()).collect()
    }

    // --- Single ---

    #[test]
    fn single_grants_when_permission_present() {
        let check = PermissionsCheck::Single("system:user:list");
        assert!(check.check(&perms(&["system:user:list"])));
    }

    #[test]
    fn single_denies_when_permission_absent() {
        let check = PermissionsCheck::Single("system:user:list");
        assert!(!check.check(&perms(&["system:role:list"])));
    }

    #[test]
    fn single_denies_empty_permissions() {
        let check = PermissionsCheck::Single("system:user:list");
        assert!(!check.check(&perms(&[])));
    }

    // --- Any ---

    #[test]
    fn any_grants_when_at_least_one_matches() {
        let check = PermissionsCheck::Any(vec!["system:user:create", "system:user:list"]);
        assert!(check.check(&perms(&["system:user:list"])));
    }

    #[test]
    fn any_denies_when_none_match() {
        let check = PermissionsCheck::Any(vec!["system:user:create", "system:user:delete"]);
        assert!(!check.check(&perms(&["system:role:list"])));
    }

    // --- All ---

    #[test]
    fn all_grants_when_every_permission_present() {
        let check = PermissionsCheck::All(vec!["system:user:create", "system:user:delete"]);
        assert!(check.check(&perms(&["system:user:create", "system:user:delete"])));
    }

    #[test]
    fn all_denies_when_any_permission_missing() {
        let check = PermissionsCheck::All(vec!["system:user:create", "system:user:delete"]);
        assert!(!check.check(&perms(&["system:user:create"])));
    }

    // --- Wildcard ---

    #[test]
    fn wildcard_grants_single() {
        let check = PermissionsCheck::Single("system:user:list");
        assert!(check.check(&perms(&["*"])));
    }

    #[test]
    fn wildcard_grants_any() {
        let check = PermissionsCheck::Any(vec!["system:user:create"]);
        assert!(check.check(&perms(&["*"])));
    }

    #[test]
    fn wildcard_grants_all() {
        let check = PermissionsCheck::All(vec!["system:user:create", "system:user:delete"]);
        assert!(check.check(&perms(&["*"])));
    }

    // --- description ---

    #[test]
    fn description_single() {
        let d = PermissionsCheck::Single("p").description();
        assert!(d.contains("single") && d.contains("p"));
    }

    #[test]
    fn description_any() {
        let d = PermissionsCheck::Any(vec!["a", "b"]).description();
        assert!(d.contains("any"));
    }

    #[test]
    fn description_all() {
        let d = PermissionsCheck::All(vec!["a", "b"]).description();
        assert!(d.contains("all"));
    }
}
