/// Generic enable/disable status shared across Menu, Role, and Dict.
///
/// Stored as `i16` in the database: `1` = Enabled, `2` = Disabled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i16)]
pub enum EnableStatus {
    Enabled = 1,
    Disabled = 2,
}

impl From<EnableStatus> for i16 {
    fn from(s: EnableStatus) -> i16 {
        s as i16
    }
}
