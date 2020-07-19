#[derive(Debug, FromSqlRow)]
pub struct RawFeatureFlag {
    pub id: i64,
    pub flag_name: String,
    pub gate_type: String,
    pub target: String,
    pub enabled: bool,
}

#[derive(Debug)]
pub enum FeatureFlag {
    Boolean {
        name: String,
        enabled: bool,
    },
    Actor {
        name: String,
        target: String,
        enabled: bool,
    },
    Group {
        name: String,
        target: String,
        enabled: bool,
    },
    Time {
        name: String,
        target: f64,
        enabled: bool,
    },
    Percentage {
        name: String,
        target: f64,
        enabled: bool,
    },
}

impl FeatureFlag {
    pub fn enabled<'a>(&'a self) -> &'a bool {
        use FeatureFlag::*;

        match self {
            Boolean { enabled, .. } => enabled,
            Actor { enabled, .. } => enabled,
            Group { enabled, .. } => enabled,
            Time { enabled, .. } => enabled,
            Percentage { enabled, .. } => enabled,
        }
    }
}
