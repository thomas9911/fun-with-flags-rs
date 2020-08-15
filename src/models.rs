use crate::Group;
use std::collections::HashSet;

#[derive(Debug)]
pub struct RawFeatureFlag {
    pub flag_name: String,
    pub gate_type: String,
    pub target: String,
    pub enabled: bool,
}

#[derive(Debug)]
pub struct RawOptionalFeatureFlags {
    pub flag_name: Option<String>,
    pub data: Vec<RawOptionalFeatureFlag>,
    pub name_set: bool,
}

impl RawOptionalFeatureFlags {
    pub fn add(&mut self, flag: RawOptionalFeatureFlag) {
        self.data.push(flag)
    }

    pub fn set_flag_name(&mut self, flag_name: String) {
        self.flag_name = Some(flag_name)
    }

    pub fn update_flag_name(&mut self) {
        for item in self.data.iter_mut() {
            item.flag_name = self.flag_name.clone()
        }
        self.name_set = true;
    }

    pub fn find(self, flag: &FeatureFlag) -> Option<FeatureFlag> {
        assert!(self.name_set);

        self.data.into_iter().find_map(|x| {
            let result = FeatureFlag::from(x);
            if flag.same(&result) {
                Some(result)
            } else {
                None
            }
        })
    }
}

impl From<RawOptionalFeatureFlags> for Vec<FeatureFlag> {
    fn from(flags: RawOptionalFeatureFlags) -> Vec<FeatureFlag> {
        flags
            .data
            .into_iter()
            .map(|x| FeatureFlag::from(x))
            .collect()
    }
}

impl Default for RawOptionalFeatureFlags {
    fn default() -> Self {
        RawOptionalFeatureFlags {
            flag_name: None,
            data: Vec::new(),
            name_set: false,
        }
    }
}

#[derive(Debug)]
pub struct RawOptionalFeatureFlag {
    pub flag_name: Option<String>,
    pub gate_type: String,
    pub target: String,
    pub enabled: bool,
}

impl From<RawOptionalFeatureFlag> for FeatureFlag {
    fn from(flags: RawOptionalFeatureFlag) -> FeatureFlag {
        match flags.gate_type.as_ref() {
            "boolean" => FeatureFlag::Boolean {
                name: flags.flag_name.unwrap(),
                enabled: flags.enabled,
            },
            "actor" => FeatureFlag::Actor {
                name: flags.flag_name.unwrap(),
                target: flags.target,
                enabled: flags.enabled,
            },
            "group" => FeatureFlag::Group {
                name: flags.flag_name.unwrap(),
                target: GroupSet::new(flags.target),
                enabled: flags.enabled,
            },
            "time" => FeatureFlag::Time {
                name: flags.flag_name.unwrap(),
                target: flags.target.parse().expect("db contains invalid data"),
                enabled: flags.enabled,
            },
            "actors" => FeatureFlag::Percentage {
                name: flags.flag_name.unwrap(),
                target: flags.target.parse().expect("db contains invalid data"),
                enabled: flags.enabled,
            },
            other_gate => panic!(format!("this gate ({}) is not supported", other_gate)),
        }
    }
}

#[derive(Debug, Clone)]
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
        target: GroupSet,
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

#[derive(Debug, Clone, PartialEq)]
pub struct GroupSet {
    data: HashSet<String>,
    length: usize,
}

impl GroupSet {
    pub fn new(one: String) -> Self {
        let mut set = HashSet::new();
        set.insert(one);
        GroupSet {
            data: set,
            length: 1,
        }
    }

    pub fn check<T: Group>(&self, to_check: &T) -> bool {
        self.data.iter().any(|x| to_check.is_in_group(x))
    }

    pub fn to_optional_string(&self) -> Option<&str> {
        if self.length == 1 {
            let mut iter = self.data.iter();
            iter.next().map(|x| (*x).as_ref())
        } else {
            None
        }
    }

    pub fn get_first_unsafe(&self) -> &str {
        self.to_optional_string()
            .expect("setting flag in redis can only be done with one group at a time")
    }
}

impl Default for GroupSet {
    fn default() -> Self {
        GroupSet {
            data: HashSet::new(),
            length: 0,
        }
    }
}

impl From<HashSet<String>> for GroupSet {
    fn from(input: HashSet<String>) -> GroupSet {
        let length = input.len();
        GroupSet {
            data: input,
            length,
        }
    }
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

    pub fn name<'a>(&'a self) -> &'a str {
        use FeatureFlag::*;

        match self {
            Boolean { name, .. } => name,
            Actor { name, .. } => name,
            Group { name, .. } => name,
            Time { name, .. } => name,
            Percentage { name, .. } => name,
        }
    }

    pub fn same(&self, other: &FeatureFlag) -> bool {
        use FeatureFlag::*;

        match self {
            Boolean { name, .. } => match other {
                &Boolean {
                    name: ref other_name,
                    ..
                } => other_name == name,
                _ => false,
            },

            Time { name, .. } => match other {
                &Time {
                    name: ref other_name,
                    ..
                } => other_name == name,
                _ => false,
            },

            Percentage { name, .. } => match other {
                &Percentage {
                    name: ref other_name,
                    ..
                } => other_name == name,
                _ => false,
            },

            Actor { name, target, .. } => match other {
                &Actor {
                    name: ref stored_name,
                    target: ref stored_target,
                    ..
                } => stored_name == name && stored_target == target,
                _ => false,
            },

            Group { name, .. } => match other {
                &Group {
                    name: ref stored_name,
                    ..
                } => stored_name == name,
                _ => false,
            },
        }
    }
}
