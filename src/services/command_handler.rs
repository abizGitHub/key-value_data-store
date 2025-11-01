use crate::app_server::parser::Command;
use crate::services::persistence_service::{clear_log_file, persist_log};
use crate::services::timer_service::do_after_delay;

use globset::{Glob, GlobMatcher};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::RwLock;
use std::time::{Duration, SystemTime};

pub static PERSIST: Lazy<RwLock<bool>> = Lazy::new(|| RwLock::new(false));

static GLOBAL_STORE: Lazy<RwLock<HashMap<String, StoredData>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

static GLOBAL_NUMBERS: Lazy<RwLock<HashMap<String, AtomicIsize>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

struct StoredData {
    value: String,
    ttl: Option<SystemTime>,
}
impl StoredData {
    fn new(value: String) -> Self {
        StoredData {
            value: value,
            ttl: None,
        }
    }
}

pub async fn handle_on_memory(cmd: Command) -> String {
    let already_in_map = |key| match GLOBAL_STORE.write().unwrap().remove(key) {
        Some(stored) => match str::parse::<isize>(&stored.value) {
            Ok(i) => Ok(i),
            Err(_) => Err(()),
        },
        None => Ok(0),
    };
    match cmd {
        Command::PING => "+PONG".to_string(),
        Command::GET { key } => match GLOBAL_STORE.read().unwrap().get(&key) {
            Some(stored) => format!("${}\r\n{}", stored.value.len(), stored.value),
            None => match GLOBAL_NUMBERS.read().unwrap().get(&key) {
                Some(i) => {
                    let x = i.load(Ordering::SeqCst);
                    format!("${}\r\n{}", x.to_string().len(), x)
                }
                None => "$-1".to_string(),
            },
        },
        Command::DEL { key } => match GLOBAL_STORE.write().unwrap().remove(&key) {
            Some(_) => ":1".to_string(),
            None => ":0".to_string(),
        },
        Command::SET { key, value } => {
            GLOBAL_STORE
                .write()
                .unwrap()
                .insert(key, StoredData::new(value));
            "+OK".to_string()
        }
        Command::KEYS { pattern } => {
            let glob: Glob = Glob::new(&pattern).expect("Invalid glob pattern");
            let matcher: GlobMatcher = glob.compile_matcher();

            let keys = GLOBAL_STORE
                .read()
                .unwrap()
                .keys()
                .filter(|&k| matcher.is_match(k))
                .map(|k| k.clone())
                .collect::<Vec<String>>();

            let ln = keys.len();

            keys.into_iter()
                .fold(String::from(format!("*{}", ln)), |mut acc, k| {
                    acc.push_str(format!("\r\n${}\r\n{}", k.len(), k).as_str());
                    acc
                })
        }
        Command::EXPIRE { key, sec } => match GLOBAL_STORE.write().unwrap().get_mut(&key) {
            Some(stored) => {
                do_after_delay(
                    move || {
                        GLOBAL_STORE.write().unwrap().remove(&key);
                    },
                    Duration::from_secs(sec),
                );
                stored.ttl = SystemTime::now().checked_add(Duration::from_secs(sec));
                ":1".to_string()
            }
            None => ":0".to_string(),
        },
        Command::FLUSHALL => {
            GLOBAL_STORE.write().unwrap().clear();
            GLOBAL_NUMBERS.write().unwrap().clear();
            "+OK".to_string()
        }
        Command::TTL { key } => match GLOBAL_STORE.read().unwrap().get(&key) {
            Some(stored) => match stored.ttl {
                Some(ttl) => {
                    let sec = ttl
                        .duration_since(SystemTime::now())
                        .unwrap_or_default()
                        .as_secs()
                        .to_string();

                    format!(":{}", sec)
                }
                None => ":-1".to_string(),
            },
            None => ":-2".to_string(),
        },
        Command::INCR { key } => match already_in_map(&key) {
            Ok(i) => {
                let mut map = GLOBAL_NUMBERS.write().unwrap();
                let counter = map.entry(key).or_insert_with(|| AtomicIsize::new(i));
                let new_value = counter.fetch_add(1, Ordering::SeqCst) + 1;
                format!(":{new_value}")
            }
            Err(_) => "-ERR value is not an integer or out of range".to_string(),
        },
        Command::DECR { key } => match already_in_map(&key) {
            Ok(i) => {
                let mut map = GLOBAL_NUMBERS.write().unwrap();
                let counter = map.entry(key).or_insert_with(|| AtomicIsize::new(i));
                let new_value = counter.fetch_sub(1, Ordering::SeqCst) - 1;
                format!(":{new_value}")
            }
            Err(_) => "-ERR value is not an integer or out of range".to_string(),
        },
    }
}

pub async fn handle_on_memory_and_file(cmd: Command) -> String {
    if *PERSIST.read().unwrap() {
        match &cmd {
            Command::PING {}
            | Command::GET { key: _ }
            | Command::KEYS { pattern: _ }
            | Command::TTL { key: _ } => {}

            Command::INCR { key: _ }
            | Command::DECR { key: _ }
            | Command::DEL { key: _ }
            | Command::SET { key: _, value: _ } => {
                persist_log(&cmd).await;
            }

            Command::EXPIRE { key, sec: _ } => {
                persist_log(&Command::DEL { key: key.clone() }).await;
            }

            Command::FLUSHALL => {
                clear_log_file().await;
            }
        }
    }
    handle_on_memory(cmd).await
}
