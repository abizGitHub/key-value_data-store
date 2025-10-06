use crate::app_server::parser::Command;
use crate::services::persistence_service::persist_log;

use globset::{Glob, GlobMatcher};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;

static GLOBAL_STORE: Lazy<RwLock<HashMap<String, String>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub async fn handle(cmd: Command) -> String {
    if matches!(
        cmd,
        Command::SET { key: _, value: _ } | Command::DEL { key: _ }
    ) {
        persist_log(&cmd).await;
    }

    match cmd {
        Command::GET { key } => match GLOBAL_STORE.read().unwrap().get(&key) {
            Some(value) => format!("${}\r\n{}", value.len(), value),
            None => "$-1".to_string(),
        },
        Command::DEL { key } => match GLOBAL_STORE.write().unwrap().remove(&key) {
            Some(_) => ":1".to_string(),
            None => ":0".to_string(),
        },
        Command::SET { key, value } => {
            GLOBAL_STORE.write().unwrap().insert(key, value);
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
    }
}
