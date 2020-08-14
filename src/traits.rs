pub trait Actor {
    fn feature_flag_id(&self) -> String;
}

impl<T> Actor for T
where
    T: AsRef<str>,
{
    fn feature_flag_id(&self) -> String {
        format!("{}", self.as_ref())
    }
}

pub trait Group {
    fn is_in_group(&self, _group_name: &str) -> bool {
        false
    }
}

impl<T> Group for T
where
    T: AsRef<str>,
{
    fn is_in_group(&self, _group_name: &str) -> bool {
        false
    }
}
