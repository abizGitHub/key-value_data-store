use crate::app_server::parser::{extract_number, extract_string, skip_new_line, Command};
use std::{
    io::{Read, Write},
    net::TcpStream,
};

#[derive(Debug)]
pub struct Connector {
    url: String,
    stream: Option<TcpStream>,
}

impl Connector {
    pub fn with_port(port: &str) -> Self {
        Connector {
            url: format!("127.0.0.1:{}", port),
            stream: None,
        }
        .connect()
    }

    pub fn with_url(url: &str) -> Self {
        Connector {
            url: url.to_string(),
            stream: None,
        }
        .connect()
    }

    fn connect(mut self: Self) -> Self {
        let mut stream = TcpStream::connect(self.url.clone()).unwrap();
        stream
            .write_all(Command::PING.to_string().as_bytes())
            .unwrap();
        let mut buffer: [u8; 512] = [0; 512];
        let n = stream.read(&mut buffer).unwrap();
        let res = String::from_utf8_lossy(&buffer[..n]).to_string();
        assert_eq!(res, String::from("+PONG\r\n"));
        self.stream = Some(stream);
        self
    }

    pub fn get(self: &Self, key: &str) -> Option<String> {
        match self.call_server(Command::cmd_get(key)).as_str() {
            "$-1\r\n" => None,
            value => {
                let mut chars = value.chars();
                let len = extract_number(&mut chars);
                skip_new_line(&mut chars);
                Some(extract_string(len, &mut chars))
            }
        }
    }

    pub fn insert(self: &Self, key: &str, value: &str) {
        self.call_server(Command::cmd_set(key, value));
    }

    pub fn keys(self: &Self, pt: &str) -> Vec<String> {
        Command::cmd_to_list(self.call_server(Command::cmd_keys(pt))).unwrap()
    }

    pub fn call_server(self: &Self, cmd: Command) -> String {
        self.stream
            .as_ref()
            .unwrap()
            .write_all(cmd.to_string().as_bytes())
            .unwrap();
        let mut buffer: [u8; 4096] = [0; 4096];
        let n = self.stream.as_ref().unwrap().read(&mut buffer).unwrap();
        String::from_utf8_lossy(&buffer[..n]).to_string()
    }
}
