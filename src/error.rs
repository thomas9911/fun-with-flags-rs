#[derive(Debug)]
pub enum Error {
    #[cfg(feature = "r2d2")]
    R2D2(r2d2::Error),
    #[cfg(feature = "redis-backend")]
    Redis(redis::RedisError),
    #[cfg(feature = "postgres-backend")]
    Postgres(postgres::Error),
    Custom(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;

        match self {
            #[cfg(feature = "r2d2")]
            R2D2(x) => write!(f, "{}", x),
            #[cfg(feature = "redis-backend")]
            Redis(x) => write!(f, "{}", x),
            #[cfg(feature = "postgres-backend")]
            Postgres(x) => write!(f, "{}", x),
            Custom(x) => write!(f, "{}", x),
        }
    }
}

impl From<String> for Error {
    fn from(e: String) -> Self {
        Error::Custom(e)
    }
}

#[cfg(feature = "r2d2")]
impl From<r2d2::Error> for Error {
    fn from(e: r2d2::Error) -> Self {
        Error::R2D2(e)
    }
}

#[cfg(feature = "redis-backend")]
impl From<redis::RedisError> for Error {
    fn from(e: redis::RedisError) -> Self {
        Error::Redis(e)
    }
}

#[cfg(feature = "postgres-backend")]
impl From<postgres::Error> for Error {
    fn from(e: postgres::Error) -> Self {
        Error::Postgres(e)
    }
}
