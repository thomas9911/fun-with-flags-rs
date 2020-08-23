#[macro_use]
extern crate serde_derive;

use reqwest::blocking::Client;
use reqwest::IntoUrl;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{to_string_pretty, Value};

use sha2::{Digest, Sha256};

#[derive(Debug)]
pub struct Config<'a> {
    url: &'a str,
}

#[derive(Debug)]
pub enum Error {
    Reqwest(reqwest::Error),
    CouchDB(CouchDBError),
    Serde(serde_json::Error),
    Custom(String),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::Reqwest(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::Serde(err)
    }
}

impl From<CouchDBError> for Error {
    fn from(err: CouchDBError) -> Error {
        Error::CouchDB(err)
    }
}

impl From<String> for Error {
    fn from(err: String) -> Error {
        Error::Custom(err)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Reqwest(e) => Some(e),
            Error::CouchDB(e) => Some(e),
            Error::Serde(e) => Some(e),
            Error::Custom(_) => None,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Reqwest(e) => write!(f, "{}", e),
            Error::CouchDB(e) => write!(f, "{}", e),
            Error::Serde(e) => write!(f, "{}", e),
            Error::Custom(e) => write!(f, "{}", e),
        }
    }
}

#[derive(Debug)]
pub struct CouchDBError {
    code: String,
    reason: String,
}

impl std::error::Error for CouchDBError {}

impl CouchDBError {
    pub fn new(code: String, reason: String) -> Self {
        CouchDBError { code, reason }
    }
}

impl std::fmt::Display for CouchDBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, reason: {}", self.code, self.reason)
    }
}

pub trait CouchDBObject {
    fn to_id(&self) -> String;
    fn get_id(&self) -> String {
        self.to_id()
    }

    fn get_rev(&self) -> Option<&str>;
    fn has_rev(&self) -> bool {
        self.get_rev().is_some()
    }

    fn update_rev(&mut self, rev: String);
}

fn fetch_value_from_map(map: &serde_json::Map<String, Value>, key: &'static str) -> String {
    map.get(key).unwrap().as_str().unwrap().to_owned()
}

fn to_result<D: DeserializeOwned>(value: Value) -> Result<D, Error> {
    // println!("{}", to_string_pretty(&value).unwrap());
    let xd = if let Value::Object(map) = value {
        if map.contains_key("error") && map.contains_key("reason") {
            Err(CouchDBError::new(
                fetch_value_from_map(&map, "error"),
                fetch_value_from_map(&map, "reason"),
            ))?
        } else {
            Value::Object(map)
        }
    } else {
        value
    };

    Ok(serde_json::from_value(xd)?)
}

pub fn create_db(client: &Client, config: &Config, db_name: &str) -> Result<Value, Error> {
    let res = put(client, &format!("{}/{}", config.url, db_name))?;
    Ok(to_result(res)?)
}

pub fn delete_db(client: &Client, config: &Config, db_name: &str) -> Result<Value, Error> {
    let res = delete(client, &format!("{}/{}", config.url, db_name))?;
    Ok(to_result(res)?)
}

pub fn put_object<J: Serialize + ?Sized, D: DeserializeOwned>(
    client: &Client,
    config: &Config,
    db_name: &str,
    body: &J,
) -> Result<D, Error> {
    let res = post_json(client, &format!("{}/{}", config.url, db_name), body)?;
    Ok(to_result(res)?)
}

pub fn get_latest_revision(
    client: &Client,
    config: &Config,
    db_name: &str,
    id: &str,
) -> Result<String, Error> {
    let url = format!("{}/{}/{}", config.url, db_name, id);

    let res = head(client, &url)?;
    match to_result(res) {
        Ok(Value::Object(map)) => {
            let tag_value = map
                .get("etag")
                .ok_or(Error::Custom("Invalid etag header".to_string()))?;
            Ok(tag_value.as_str().unwrap().trim_matches('"').to_owned())
        }
        Ok(_) => Err(Error::Custom("Invalid etag header".to_string())),
        Err(e) => Err(e),
    }
}

pub fn update_object<J, D>(
    client: &Client,
    config: &Config,
    db_name: &str,
    body: &mut J,
) -> Result<D, Error>
where
    J: Serialize + ?Sized + CouchDBObject,
    D: DeserializeOwned,
{
    let url = format!("{}/{}", config.url, db_name);
    if body.has_rev() {
        let res = post_json(client, &url, body)?;
        Ok(to_result(res)?)
    } else {
        let id = body.get_id();
        let rev = get_latest_revision(client, config, db_name, &id)?;
        body.update_rev(rev);

        update_object(client, config, db_name, body)
    }
}

pub fn get_object<D>(client: &Client, config: &Config, db_name: &str, id: &str) -> Result<D, Error>
where
    D: DeserializeOwned,
{
    let url = format!("{}/{}/{}", config.url, db_name, id);
    let res = get(client, &url)?;
    Ok(to_result(res)?)
}

pub fn delete_object<J, D>(
    client: &Client,
    config: &Config,
    db_name: &str,
    body: &mut J,
) -> Result<D, Error>
where
    J: Serialize + ?Sized + CouchDBObject,
    D: DeserializeOwned,
{
    if body.has_rev() {
        let id = body.get_id();
        let rev = body.get_rev().unwrap();

        let url = format!("{}/{}/{}?rev={}", config.url, db_name, id, rev);

        let res = delete(client, &url)?;
        Ok(to_result(res)?)
    } else {
        let id = body.get_id();
        let rev = get_latest_revision(client, config, db_name, &id)?;
        body.update_rev(rev);

        delete_object(client, config, db_name, body)
    }
}

pub fn delete_object_by_id<D>(
    client: &Client,
    config: &Config,
    db_name: &str,
    id: &str,
) -> Result<D, Error>
where
    D: DeserializeOwned,
{
    let rev = get_latest_revision(client, config, db_name, id)?;
    let url = format!("{}/{}/{}?rev={}", config.url, db_name, id, rev);

    let res = delete(client, &url)?;
    Ok(to_result(res)?)
}

pub fn get<U: IntoUrl>(client: &Client, url: U) -> Result<Value, Error> {
    Ok(client.get(url).send()?.json()?)
}

pub fn head<U: IntoUrl>(client: &Client, url: U) -> Result<Value, Error> {
    let mut map = serde_json::Map::<String, Value>::new();

    for (key, v) in client.head(url).send()?.headers().into_iter() {
        if let Ok(value) = v.to_str() {
            map.insert(key.as_str().to_owned(), value.into());
        }
    }

    Ok(Value::Object(map))
}

pub fn put<U: IntoUrl>(client: &Client, url: U) -> Result<Value, Error> {
    Ok(client.put(url).send()?.json()?)
}

pub fn put_json<U: IntoUrl, J: Serialize + ?Sized>(
    client: &Client,
    url: U,
    json: &J,
) -> Result<Value, Error> {
    Ok(client.put(url).json(json).send()?.json()?)
}

pub fn post_json<U: IntoUrl, J: Serialize + ?Sized>(
    client: &Client,
    url: U,
    json: &J,
) -> Result<Value, Error> {
    Ok(client.post(url).json(json).send()?.json()?)
}

pub fn delete<U: IntoUrl>(client: &Client, url: U) -> Result<Value, Error> {
    Ok(client.delete(url).send()?.json()?)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestObject {
    #[serde(rename = "_id")]
    id: String,
    #[serde(rename = "_rev")]
    #[serde(skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    name: String,
    fields: Vec<String>,
}

fn sha2(
    input: &str,
) -> sha2::digest::generic_array::GenericArray<u8, <Sha256 as sha2::digest::FixedOutput>::OutputSize>
{
    Sha256::digest(input.as_bytes())
}

fn string_to_uuid(input: &str) -> String {
    format!("{:x}", sha2(input))[..32].to_string()
}

impl TestObject {
    pub fn new(name: String, fields: Vec<String>) -> TestObject {
        let id = string_to_uuid(&name);
        TestObject {
            id,
            name,
            fields,
            rev: None,
        }
    }

    pub fn empty(name: String) -> TestObject {
        TestObject {
            id: String::new(),
            name,
            fields: Vec::new(),
            rev: None,
        }
    }

    // pub fn to_id(&self) -> String {
    //     string_to_uuid(&self.name)
    // }
}

impl CouchDBObject for TestObject {
    fn to_id(&self) -> String {
        string_to_uuid(self.name.as_ref())
    }

    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn get_rev(&self) -> Option<&str> {
        match &self.rev {
            Some(x) => Some(x),
            None => None,
        }
    }

    fn update_rev(&mut self, input: String) {
        self.rev = Some(input);
    }
}

fn action() -> Result<(), Error> {
    let config = Config {
        url: "http://username:password@127.0.0.1:5984",
    };
    let client = Client::new();

    // delete_db(&client, &config, "xd").is_ok();

    // let res = create_db(&client, &config, "xd")?;
    // println!("{}", res);

    let mut t = TestObject::new("xds".to_string(), vec![String::from("HAHAHAHA")]);

    println!("{}", to_string_pretty(&t).unwrap());
    let res: Value = delete_object(&client, &config, "xd", &mut t)?;
    println!("{:?}", res);

    // let res = put_object(&client, &config, "xd", &t)?;
    // let res = update_object(&client, &config, "xd", &mut t)?;
    let res: TestObject = get_object(&client, &config, "xd", &t.get_id())?;
    println!("{:?}", res);

    // let res = delete_db(&client, &config, "xd")?;
    // println!("{}", res);

    // println!("{}", t.to_id());

    Ok(())
}

fn main() {
    match action() {
        Ok(_) => (),
        Err(x) => println!("{}", x),
    }
}
