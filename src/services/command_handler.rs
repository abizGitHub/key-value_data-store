use crate::app_server::parser::Command;
use crate::services::persistence_service::{clear_log_file, persist_log};
use crate::services::timer_service::do_after_delay;

use globset::{Glob, GlobMatcher};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, SystemTime};

static GLOBAL_STORE: Lazy<RwLock<HashMap<String, StoredData>>> =
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
    match cmd {
        Command::PING => "+PONG".to_string(),
        Command::GET { key } => match GLOBAL_STORE.read().unwrap().get(&key) {
            Some(stored) => format!("${}\r\n{}", stored.value.len(), stored.value),
            None => "$-1".to_string(),
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
    }
}

pub async fn handle_on_memory_and_file(cmd: Command) -> String {
    match &cmd {
        Command::PING => {}
        Command::SET { key: _, value: _ } => {
            persist_log(&cmd).await;
        }
        Command::DEL { key: _ } => {
            persist_log(&cmd).await;
        }
        Command::EXPIRE { key, sec: _ } => {
            persist_log(&Command::DEL { key: key.clone() }).await;
        }
        Command::FLUSHALL => {
            clear_log_file().await;
        }
        Command::GET { key: _ } => {}
        Command::KEYS { pattern: _ } => {}
        Command::TTL { key: _ } => {}
    }
    handle_on_memory(cmd).await
}
