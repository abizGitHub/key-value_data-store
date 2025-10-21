use kvds::{app_server::parser::Command, connector::connector::Connector};

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
        thread::{self, sleep},
        time::Duration,
    };

    use ctor::ctor;
    use kvds::{
        app_server::{parser::Command, socket_server::AppServer},
        connector::connector::Connector,
    };
    use serial_test::serial;

    use crate::flush_all;

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
        let c = Connector::with_port("7878");

        let resp = c.call_server(Command::cmd_set("some-key", "some-value"));
        assert_eq!(resp, "+OK\r\n");

        // ======================= GET THE VALUE =====================

        let resp = c.call_server(Command::cmd_get("some-key"));
        assert_eq!(resp, "$10\r\nsome-value\r\n");

        // ======================= DEL THE VALUE =====================

        let resp = c.call_server(Command::cmd_del("some-key"));
        assert_eq!(resp, ":1\r\n");

        // ========================= GET NULL ========================

        let resp = c.call_server(Command::cmd_get("some-key"));
        assert_eq!(resp, "$-1\r\n");

        flush_all()
    }

    #[serial]
    #[test]
    fn keys_by_pattern() {
        // ======================= SET SOME VALUES ========================
        let c = Connector::with_port("7878");

        let keys = vec!["key1", "key2", "key3", "key4", "key5"];
        for k in &keys {
            c.call_server(Command::cmd_set(k, "some-value"));
        }

        // ==================== GET KEYS BY PATTERN =======================

        let mut result = Command::cmd_to_list(c.call_server(Command::cmd_keys("*"))).unwrap();
        result.sort();
        assert_eq!(keys, result);

        flush_all()
    }

    #[serial]
    #[test]
    fn expire_time_for_a_key() {
        // ======================= SET A KEY WITH EXPIRATION ========================
        let c = Connector::with_port("7878");

        let resp = c.call_server(Command::cmd_set("some-key", "some-value"));
        assert_eq!(resp, "+OK\r\n");

        // ============================ EXPIRATION ==================================

        let resp = c.call_server(Command::cmd_expire("some-key", 1));
        assert_eq!(resp, ":1\r\n");
        // ======
        let resp = c.call_server(Command::cmd_get("some-key"));
        assert_eq!(resp, "$10\r\nsome-value\r\n");

        // =================== GET THE VALUE AFTER A SECOND =========================

        sleep(Duration::from_millis(1_001));
        let resp = c.call_server(Command::cmd_get("some-key"));
        assert_eq!(resp, "$-1\r\n");

        flush_all()
    }

    #[serial]
    #[test]
    fn ttl_for_a_key() {
        // ======================= SET A KEY WITH EXPIRATION ========================
        let c = Connector::with_port("7878");

        let resp = c.call_server(Command::cmd_set("a-key", "some-value"));
        assert_eq!(resp, "+OK\r\n");

        // ============================ EXPIRATION ==================================

        let resp = c.call_server(Command::cmd_expire("a-key", 2));
        assert_eq!(resp, ":1\r\n");
        // ======
        let resp = c.call_server(Command::cmd_ttl("a-key"));
        assert_eq!(resp, ":1\r\n");

        // =================== GET THE VALUE AFTER A SECOND =========================

        sleep(Duration::from_millis(2_001));
        let resp = c.call_server(Command::cmd_get("a-key"));
        assert_eq!(resp, "$-1\r\n");

        flush_all()
    }
}

mod connector_tests {
    use std::{thread, time::Duration};

    use kvds::{app_server::socket_server::AppServer, connector::connector::Connector};

    use crate::flush_all;
    use ctor::ctor;

    #[ctor]
    fn global_setup() {
        thread::spawn(|| {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async { AppServer::new("7879").start().await });
        });
        thread::sleep(Duration::from_millis(300));
    }

    #[test]
    fn test_connector() {
        // ======================= SET SOME VALUES ========================
        let c = Connector::with_port("7879");

        let keys = vec!["key1", "key2", "key3", "key4", "key5"];
        for k in &keys {
            c.insert(k, "some-value");
        }

        // ==================== GET KEYS BY PATTERN =======================

        let mut result = c.keys("*");
        result.sort();
        assert_eq!(keys, result);

        let value = c.get("key1");
        assert_eq!(value, Some("some-value".to_string()));

        let value = c.get("unknown");
        assert_eq!(value, None);

        flush_all()
    }
}

pub fn flush_all() {
    let resp = Connector::with_port("7878").call_server(Command::FLUSHALL);
    assert_eq!(resp, "+OK\r\n");
}
