use std::convert::TryFrom;

pub use config::ConfigError;
use config::{Config, Environment, File};

use dotenv::dotenv;

#[derive(Debug, Deserialize)]
pub struct RawConfig {
    #[serde(rename(deserialize = "name"))]
    pub database_name: Option<String>,
    #[serde(rename(deserialize = "address"))]
    pub database_address: Option<String>,
    #[serde(rename(deserialize = "general"))]
    pub general_config: Option<GeneralConfig>,
    #[serde(rename(deserialize = "redis"))]
    pub redis_config: Option<RedisConfig>,
    #[serde(rename(deserialize = "postgres"))]
    pub postgres_config: Option<PostgresConfig>,
}

#[derive(Debug, Deserialize)]
pub struct GeneralConfig {
    pub backend: BackendType,
}

#[derive(Debug, Deserialize)]
pub struct RedisConfig {
    pub url: String, // cluster configs ect...
}

#[derive(Debug, Deserialize)]
pub struct PostgresConfig {
    pub url: String,
}

#[derive(Debug, PartialEq, Clone, Copy, Deserialize)]
#[serde(rename_all(deserialize = "lowercase"))]
pub enum BackendType {
    Redis,
    Postgres,
    Auto,
}

impl RawConfig {
    pub fn what_type(&self) -> Option<BackendType> {
        let env_var = self.database_name.is_some() && self.database_address.is_some();
        if env_var {
            return Some(BackendType::Auto);
        }

        if let Some(general_config) = &self.general_config {
            return Some(general_config.backend);
        }

        if self.postgres_config.is_some() && self.redis_config.is_some() {
            return None;
        }

        if self.redis_config.is_some() {
            return Some(BackendType::Redis);
        }

        if self.postgres_config.is_some() {
            return Some(BackendType::Postgres);
        }

        None
    }

    pub fn parts(&self) -> Option<(String, String)> {
        if let Some(BackendType::Auto) = self.what_type() {
            let address = self.database_address.as_ref().unwrap().to_owned();
            let name = self.database_name.as_ref().unwrap().to_owned();
            Some((address, name))
        } else {
            None
        }
    }

    pub fn to_url(&self) -> Option<String> {
        if let Some(x) = self.what_type() {
            match x {
                BackendType::Redis => Some(self.redis_config.as_ref().unwrap().url.to_owned()),
                BackendType::Postgres => {
                    Some(self.postgres_config.as_ref().unwrap().url.to_owned())
                }
                BackendType::Auto => {
                    let (address, name) = self
                        .parts()
                        .expect("this only fails if backend type is not auto");
                    Some(format!("{}/{}", address, name))
                }
            }
        } else {
            None
        }
    }
}

impl TryFrom<Config> for RawConfig {
    type Error = ConfigError;

    fn try_from(input: Config) -> Result<RawConfig, Self::Error> {
        let config: RawConfig = input.try_into()?;

        if config.what_type().is_none() {
            return Err(ConfigError::Message(String::from("backend type not found")));
        }

        Ok(config)
    }
}

pub fn fetch_config() -> Result<RawConfig, ConfigError> {
    dotenv().ok();

    let mut settings = Config::default();
    settings
        .merge(File::with_name("fun-with-flags").required(false))
        .unwrap()
        .merge(Environment::with_prefix("DATABASE"))
        .unwrap();

    RawConfig::try_from(settings)
}

#[test]
fn get_from_file_and_env_test() {
    assert!(fetch_config().is_ok())
}

#[test]
fn to_url_redis_test() {
    let config = RawConfig {
        database_address: None,
        database_name: None,
        general_config: Some(GeneralConfig {
            backend: BackendType::Redis,
        }),
        redis_config: Some(RedisConfig {
            url: "redis://testing".into(),
        }),
        postgres_config: Some(PostgresConfig {
            url: "postgres://testing".into(),
        }),
    };

    assert_eq!(Some("redis://testing".into()), config.to_url());
}

#[test]
fn to_url_postgres_test() {
    let config = RawConfig {
        database_address: None,
        database_name: None,
        general_config: Some(GeneralConfig {
            backend: BackendType::Postgres,
        }),
        redis_config: Some(RedisConfig {
            url: "redis://testing".into(),
        }),
        postgres_config: Some(PostgresConfig {
            url: "postgres://testing".into(),
        }),
    };

    assert_eq!(Some("postgres://testing".into()), config.to_url());
}

#[test]
fn to_url_env_test() {
    let config = RawConfig {
        database_address: Some("test://testing".into()),
        database_name: Some("database".into()),
        general_config: Some(GeneralConfig {
            backend: BackendType::Auto,
        }),
        redis_config: Some(RedisConfig {
            url: "redis://testing".into(),
        }),
        postgres_config: Some(PostgresConfig {
            url: "postgres://testing".into(),
        }),
    };

    assert_eq!(Some("test://testing/database".into()), config.to_url());
}

#[test]
fn config_from_toml_redis() {
    use config::FileFormat;

    let mut settings = Config::default();
    settings
        .merge(File::from_str(
            r#"
        [redis]
        url = "redis://redis"
        "#,
            FileFormat::Toml,
        ))
        .unwrap();

    let config = RawConfig::try_from(settings).unwrap();

    assert_eq!(Some(BackendType::Redis), config.what_type());
    assert_eq!(Some("redis://redis".into()), config.to_url());
}

#[test]
fn config_from_toml_postgres() {
    use config::FileFormat;

    let mut settings = Config::default();
    settings
        .merge(File::from_str(
            r#"
        [postgres]
        url = "postgres://postgres"
        "#,
            FileFormat::Toml,
        ))
        .unwrap();

    let config = RawConfig::try_from(settings).unwrap();

    assert_eq!(Some(BackendType::Postgres), config.what_type());
    assert_eq!(Some("postgres://postgres".into()), config.to_url());
}

#[test]
fn config_from_toml_redis_postgres_without_general() {
    use config::FileFormat;

    let mut settings = Config::default();
    settings
        .merge(File::from_str(
            r#"
        [redis]
        url = "redis://redis"

        [postgres]
        url = "postgres://postgres"
        "#,
            FileFormat::Toml,
        ))
        .unwrap();

    assert!(RawConfig::try_from(settings).is_err())
}

#[test]
fn config_from_toml_redis_postgres_redis() {
    use config::FileFormat;

    let mut settings = Config::default();
    settings
        .merge(File::from_str(
            r#"
        [general]
        backend = "redis"

        [redis]
        url = "redis://redis"

        [postgres]
        url = "postgres://postgres"
        "#,
            FileFormat::Toml,
        ))
        .unwrap();

    let config = RawConfig::try_from(settings).unwrap();

    assert_eq!(Some(BackendType::Redis), config.what_type());
    assert_eq!(Some("redis://redis".into()), config.to_url());
}

#[test]
fn config_from_toml_redis_postgres_postgres() {
    use config::FileFormat;

    let mut settings = Config::default();
    settings
        .merge(File::from_str(
            r#"
        [general]
        backend = "postgres"

        [redis]
        url = "redis://redis"

        [postgres]
        url = "postgres://postgres"
        "#,
            FileFormat::Toml,
        ))
        .unwrap();

    let config = RawConfig::try_from(settings).unwrap();

    assert_eq!(Some(BackendType::Postgres), config.what_type());
    assert_eq!(Some("postgres://postgres".into()), config.to_url());
}
