use crate::app_server::parser::Command;

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;

static GLOBAL_STORE: Lazy<RwLock<HashMap<String, String>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub fn handle(cmd: Command) -> String {
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
        },
    }
}
