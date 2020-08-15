#[derive(Debug)]
// pub struct Error(String);
pub enum Error {
    // #[cfg(any(feature = "redis-backend", feature = "postgres-backend"))]
    #[cfg(feature = "r2d2")]
    R2D2(r2d2::Error),
    #[cfg(feature = "redis-backend")]
    Redis(redis::RedisError),
    Custom(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write!(f, "{}", self.0)
        use Error::*;

        match self {
            #[cfg(feature = "r2d2")]
            R2D2(x) => write!(f, "{}", x),
            #[cfg(feature = "redis-backend")]
            Redis(x) => write!(f, "{}", x),
            Custom(x) => write!(f, "{}", x),
        }
    }
}

#[cfg(feature = "r2d2")]
impl From<r2d2::Error> for Error {
    fn from(e: r2d2::Error) -> Self {
        // Error(e.to_string())
        Error::R2D2(e)
    }
}

#[cfg(feature = "redis-backend")]
impl From<redis::RedisError> for Error {
    fn from(e: redis::RedisError) -> Self {
        Error::Redis(e)
    }
}
