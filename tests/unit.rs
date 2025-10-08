#[cfg(test)]
mod tests {
    use kvds::app_server::parser::{parse_command, Command};

    #[test]
    fn parse_get_command() {
        let cmd =
            parse_command("*3\r\n$3\r\nSET\r\n$8\r\nsome-key\r\n$10\r\nsome-value\r\n".to_string());
        assert_eq!(
            cmd.unwrap(),
            Command::SET {
                key: "some-key".to_string(),
                value: "some-value".to_string()
            }
        );
    }
}

#[cfg(test)]
mod base_command_tests {
    use std::{
        io::{Read, Write},
        net::TcpStream,
        thread::{self, sleep},
        time::Duration,
    };

    use ctor::ctor;
    use kvds::app_server::{parser::Command, socket_server::AppServer};
    use serial_test::serial;

    #[ctor]
    fn global_setup() {
        thread::spawn(|| {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async { AppServer::new("7878").start().await });
        });
        thread::sleep(Duration::from_millis(300));
    }

    #[serial]
    #[test]
    fn set_then_get_and_delete() {
        // ======================= SET A VALUE =======================

        let resp = call_server(Command::cmd_set("some-key", "some-value"));
        assert_eq!(resp, "+OK\r\n");

        // ======================= GET THE VALUE =====================

        let resp = call_server(Command::cmd_get("some-key"));
        assert_eq!(resp, "$10\r\nsome-value\r\n");

        // ======================= DEL THE VALUE =====================

        let resp = call_server(Command::cmd_del("some-key"));
        assert_eq!(resp, ":1\r\n");

        // ========================= GET NULL ========================

        let resp = call_server(Command::cmd_get("some-key"));
        assert_eq!(resp, "$-1\r\n");

        flush_all()
    }

    #[serial]
    #[test]
    fn keys_by_pattern() {
        // ======================= SET SOME VALUES ========================

        let keys = vec!["key1", "key2", "key3", "key4", "key5"];
        for k in &keys {
            call_server(Command::cmd_set(k, "some-value"));
        }

        // ==================== GET KEYS BY PATTERN =======================

        let mut result = Command::cmd_to_list(call_server(Command::cmd_keys("*"))).unwrap();
        result.sort();
        assert_eq!(keys, result);

        flush_all()
    }

    #[serial]
    #[test]
    fn expire_time_for_a_key() {
        // ======================= SET A KEY WITH EXPIRATION ========================

        let resp = call_server(Command::cmd_set("some-key", "some-value"));
        assert_eq!(resp, "+OK\r\n");

        // ============================ EXPIRATION ==================================

        let resp = call_server(Command::cmd_expire("some-key", 1));
        assert_eq!(resp, ":1\r\n");
        // ======
        let resp = call_server(Command::cmd_get("some-key"));
        assert_eq!(resp, "$10\r\nsome-value\r\n");

        // =================== GET THE VALUE AFTER A SECOND =========================

        sleep(Duration::from_secs(1));
        let resp = call_server(Command::cmd_get("some-key"));
        assert_eq!(resp, "$-1\r\n");
    }

    fn call_server(cmd: Command) -> String {
        let mut stream = TcpStream::connect("127.0.0.1:7878").unwrap();
        stream.write_all(cmd.to_string().as_bytes()).unwrap();
        let mut buffer: [u8; 512] = [0; 512];
        let n = stream.read(&mut buffer).unwrap();
        String::from_utf8_lossy(&buffer[..n]).to_string()
    }

    fn flush_all() {
        let mut stream: TcpStream = TcpStream::connect("127.0.0.1:7878").unwrap();
        stream
            .write_all(Command::FLUSHALL.to_string().as_bytes())
            .unwrap();
        let mut buffer: [u8; 512] = [0; 512];
        let n = stream.read(&mut buffer).unwrap();

        assert_eq!(String::from_utf8_lossy(&buffer[..n]), "+OK\r\n");
    }
}
