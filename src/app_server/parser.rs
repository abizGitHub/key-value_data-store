use std::fmt::Error;
use std::str::Chars;

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    GET { key: String },
    SET { key: String, value: String },
    DEL { key: String },
    KEYS { pattern: String },
    EXPIRE { key: String, sec: u64 },
    FLUSHALL,
}

impl Command {
    pub fn cmd_set(key: &str, value: &str) -> Self {
        Self::SET {
            key: key.to_string(),
            value: value.to_string(),
        }
    }
    pub fn cmd_get(key: &str) -> Self {
        Self::GET {
            key: key.to_string(),
        }
    }
    pub fn cmd_del(key: &str) -> Self {
        Self::DEL {
            key: key.to_string(),
        }
    }
    pub fn cmd_expire(key: &str, sec: u64) -> Self {
        Self::EXPIRE {
            key: key.to_string(),
            sec,
        }
    }
    pub fn cmd_keys(pat: &str) -> Self {
        Self::KEYS {
            pattern: pat.to_string(),
        }
    }
    pub fn cmd_to_list(cmd: String) -> Result<Vec<String>, Error> {
        let mut cmd_seq = cmd.chars();
        if !is_char('*', &mut cmd_seq) {
            return Err(Error);
        }
        let n = extract_number(&mut cmd_seq);
        skip_new_line(&mut cmd_seq);
        let mut result = Vec::new();
        for _ in 0..n {
            if !is_char('$', &mut cmd_seq) {
                return Err(Error);
            }
            let len = extract_number(&mut cmd_seq);
            skip_new_line(&mut cmd_seq);
            result.push(extract_string(len, &mut cmd_seq));
            skip_new_line(&mut cmd_seq);
        }
        Ok(result)
    }
}

impl ToString for Command {
    fn to_string(&self) -> String {
        match self {
            Self::GET { key } => format!("*2\r\n$3\r\nGET\r\n${}\r\n{}\r\n", key.len(), key),
            Self::DEL { key } => format!("*2\r\n$3\r\nDEL\r\n${}\r\n{}\r\n", key.len(), key),
            Self::SET { key, value } => format!(
                "*3\r\n$3\r\nSET\r\n${}\r\n{}\r\n${}\r\n{}\r\n",
                key.len(),
                key,
                value.len(),
                value
            ),
            Self::KEYS { pattern } => {
                format!("*2\r\n$4\r\nKEYS\r\n${}\r\n{}\r\n", pattern.len(), pattern)
            }
            Self::EXPIRE { key, sec } => format!(
                "*3\r\n$6\r\nEXPIRE\r\n${}\r\n{}\r\n${}\r\n{}\r\n",
                key.len(),
                key,
                sec.to_string().len(),
                sec
            ),
            Self::FLUSHALL => "*1\r\n$8\r\nFLUSHALL\r\n".to_string(),
        }
    }
}

// '*3\r\n$3\r\nSET\r\n$3\r\nkey\r\n$4\r\value\r\n'
pub fn parse_command(cmd: String) -> Result<Command, Error> {
    let mut cmd_parts = Command::cmd_to_list(cmd)?.into_iter();
    match cmd_parts.next().ok_or(Error)?.as_str() {
        "GET" => {
            let key = cmd_parts.next().ok_or(Error)?;
            Ok(Command::GET { key })
        }
        "DEL" => {
            let key = cmd_parts.next().ok_or(Error)?;
            Ok(Command::DEL { key })
        }
        "SET" => {
            let key = cmd_parts.next().ok_or(Error)?;
            let value = cmd_parts.next().ok_or(Error)?;
            Ok(Command::SET { key, value })
        }
        "KEYS" => {
            let pattern = cmd_parts.next().ok_or(Error)?;
            Ok(Command::KEYS { pattern })
        }
        "EXPIRE" => {
            let key = cmd_parts.next().ok_or(Error)?;
            let sec = cmd_parts.next().ok_or(Error)?.parse().unwrap();
            Ok(Command::EXPIRE { key, sec })
        }
        "FLUSHALL" => Ok(Command::FLUSHALL),
        _ => Err(Error),
    }
}

fn is_char(c: char, cmd: &mut Chars<'_>) -> bool {
    cmd.next() == Some(c)
}

fn extract_number(cmd: &mut Chars<'_>) -> usize {
    let mut v = Vec::new();
    loop {
        let c = cmd.next().unwrap();
        if c.is_numeric() {
            v.push(c);
        } else {
            break;
        }
    }
    v.iter().collect::<String>().parse().unwrap()
}

fn skip_new_line(cmd: &mut Chars<'_>) -> bool {
    cmd.next() == Some('\n') || cmd.next() == Some('\n')
}

fn extract_string(n: usize, cmd: &mut Chars<'_>) -> String {
    cmd.take(n).collect::<String>()
}
