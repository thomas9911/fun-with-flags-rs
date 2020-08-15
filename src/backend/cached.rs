use std::sync::Mutex;
use std::time::Duration;

use lru_time_cache::LruCache;
use crate::FeatureFlag;
use state::Storage;

use crate::backend::{DataBackend, DBConnection, SetOutput, GetOutput};

lazy_static::lazy_static!{
    static ref GLOBAL_MAP: Storage<Mutex<LruCache<String, FeatureFlag>>> = {
        let storage = Storage::new();
        let initial_map = LruCache::with_expiry_duration_and_capacity(Duration::from_secs(60), 1000);
        storage.set(Mutex::new(initial_map));
        storage
    };
}

pub struct Backend;

impl Backend {
    pub fn get(conn: &DBConnection, flag: FeatureFlag) -> GetOutput {
        if let Some(feature_flag) = get_from_cache(&flag) {
            Ok(feature_flag)
        } else {
            DataBackend::get(conn, flag)
        }
    }

    pub fn set(conn: &DBConnection, flag: FeatureFlag) -> SetOutput {
        match DataBackend::set(conn, flag) {
            Ok(flag) => {
                set_in_cache(flag.clone());
                Ok(flag)
            }
            Err(e) => Err(e)
        }
    }
}

fn get_from_cache(flag: &FeatureFlag) -> Option<FeatureFlag>{
    let key = flag.to_cache_key();

    let mut cache = GLOBAL_MAP.get().lock().unwrap();

    cache.get(&key).cloned()
}

fn set_in_cache(flag: FeatureFlag){
    let key = flag.to_cache_key();

    let mut cache = GLOBAL_MAP.get().lock().unwrap();

    cache.insert(key, flag);
}

#[test]
fn oke() {
    let ff = FeatureFlag::Boolean{name: "oke".to_string(), enabled: true};
    set_in_cache(ff.clone());
    let f = get_from_cache(&ff);
    println!("{:?}", f);

    panic!();
}

impl FeatureFlag{
    pub fn to_cache_key(&self) -> String {
        use FeatureFlag::*;

        match self {
            Boolean{name, ..} => format!("boolean-{}", name),
            Actor{name, ..} => format!("actor-{}", name),
            Group{name, ..} => format!("group-{}", name),
            Time{name, ..} => format!("time-{}", name),
            Percentage{name, ..} => format!("percentage-{}", name),
        }
    }
}