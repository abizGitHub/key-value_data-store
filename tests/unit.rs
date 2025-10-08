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
        thread,
        time::Duration,
    };

    use ctor::ctor;
    use kvds::app_server::{parser::Command, socket_server::AppServer};

    #[ctor]
    fn global_setup() {
        thread::spawn(|| {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async { AppServer::new("7878").start().await });
        });
        thread::sleep(Duration::from_millis(300));
    }

    #[test]
    fn set_then_get_and_delete() {
        let mut stream = TcpStream::connect("127.0.0.1:7878").unwrap();
        // ======================= SET A VALUE ========================
        let set_cmd = Command::SET {
            key: "some-key".to_string(),
            value: "some-value".to_string(),
        };
        stream.write_all(set_cmd.to_string().as_bytes()).unwrap();
        let mut buffer = [0; 512];
        let n = stream.read(&mut buffer).unwrap();

        assert_eq!(String::from_utf8_lossy(&buffer[..n]), "+OK\r\n");

        // ====================== GET THE VALUE =====================
        let get_cmd = Command::GET {
            key: "some-key".to_string(),
        };
        stream.write_all(get_cmd.to_string().as_bytes()).unwrap();
        let mut buffer: [u8; 512] = [0; 512];
        let n = stream.read(&mut buffer).unwrap();

        assert_eq!(
            String::from_utf8_lossy(&buffer[..n]),
            "$10\r\nsome-value\r\n"
        );

        // ====================== DEL THE VALUE =====================
        let del_cmd = Command::DEL {
            key: "some-key".to_string(),
        };
        stream.write_all(del_cmd.to_string().as_bytes()).unwrap();
        let mut buffer: [u8; 512] = [0; 512];
        let n = stream.read(&mut buffer).unwrap();

        assert_eq!(String::from_utf8_lossy(&buffer[..n]), ":1\r\n");

        // ======================= GET NULL =====================
        let get_cmd = Command::GET {
            key: "some-key".to_string(),
        };
        stream.write_all(get_cmd.to_string().as_bytes()).unwrap();
        let mut buffer: [u8; 512] = [0; 512];
        let n = stream.read(&mut buffer).unwrap();

        assert_eq!(String::from_utf8_lossy(&buffer[..n]), "$-1\r\n");
    }

    #[test]
    fn keys_by_pattern() {
        let mut stream = TcpStream::connect("127.0.0.1:7878").unwrap();

        // ======================= SET SOME VALUES ========================
        let keys = vec!["key1", "key2", "key3", "key4", "key5"];
        for k in &keys {
            let set_cmd = Command::SET {
                key: k.to_string(),
                value: "some-value".to_string(),
            };
            stream.write_all(set_cmd.to_string().as_bytes()).unwrap();
            let mut buffer: [u8; 512] = [0; 512];
            stream.read(&mut buffer).unwrap();
        }

        // ==================== GET KEYS BY PATTERN =======================
        let cmd_keys = Command::cmd_keys("*");
        stream.write_all(cmd_keys.to_string().as_bytes()).unwrap();
        let mut buffer: [u8; 512] = [0; 512];
        let n = stream.read(&mut buffer).unwrap();
        let mut result =
            Command::cmd_to_list(String::from_utf8_lossy(&buffer[..n]).to_string()).unwrap();

        result.sort();

        assert_eq!(keys, result);
    }
}
