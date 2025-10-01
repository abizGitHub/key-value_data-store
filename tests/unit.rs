#[cfg(test)]
mod tests {
    use kvds::app_server::parser::{parse_command, Command};

    #[test]
    fn parse_get_command() {
        let cmd = parse_command(
            "*3\r\n$3\r\nSET\r\n$8\r\nsome-key\r\n$10\r\nsome-value\r\n"
                .to_string()
                .chars(),
        );
        assert_eq!(
            cmd.unwrap(),
            Command::SET {
                key: "some-key".to_string(),
                value: "some-value".to_string()
            }
        );
    }
}
