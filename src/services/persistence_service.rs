use crate::app_server::parser::{parse_command, Command};
use crate::services::command_handler::handle_on_memory;
use crate::SETTING;
use once_cell::sync::Lazy;
use std::io::{Read, Write};
use std::{
    fs::{File, OpenOptions},
    sync::RwLock,
};
use tokio::sync::mpsc::{self, Sender};

static DB_FILE: Lazy<RwLock<File>> = Lazy::new(|| {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .read(true)
        .open(SETTING.db_file.clone())
        .expect("error in read or create DB file!");
    RwLock::new(file)
});

static QUEUE: Lazy<Sender<String>> = Lazy::new(|| {
    let (tx, mut rx) = mpsc::channel::<String>(10_000);
    tokio::spawn(async move {
        while let Some(mut message) = rx.recv().await {
            message.push_str("\r\n");
            let _ = DB_FILE.write().unwrap().write(message.as_bytes());
        }
    });
    tx
});

pub async fn persist_log(cmd: &Command) {
    QUEUE
        .send(cmd.to_string().replace("\r\n", "\\r\\n"))
        .await
        .expect("error sending log to queue!");
}

pub async fn load_data() {
    let mut stored_data = String::new();
    DB_FILE
        .write()
        .expect("error opening db file!")
        .read_to_string(&mut stored_data)
        .expect("error opening db file!");

    for row in stored_data.lines() {
        let cmd = parse_command(row.replace("\\r\\n", "\r\n")).expect("error reading db rows!");
        handle_on_memory(cmd).await;
    }
}

pub async fn clear_log_file() {
    OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(SETTING.db_file.clone())
        .expect("error in read or create DB file!");

    let _ = DB_FILE.write().unwrap().sync_data();
}
