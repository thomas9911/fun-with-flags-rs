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
