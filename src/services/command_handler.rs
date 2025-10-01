use crate::app_server::parser::Command;

pub fn handle(cmd: Command) -> String {
    println!("command received: {cmd:?}");
    "+OK".to_string()
}
