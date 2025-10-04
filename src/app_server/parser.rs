use std::fmt::Error;
use std::str::Chars;

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    GET { key: String },
    SET { key: String, value: String },
    DEL { key: String },
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
        }
    }
}

// '*3\r\n$3\r\nSET\r\n$3\r\nkey\r\n$4\r\value\r\n'
pub fn parse_command(mut cmd_seq: Chars<'_>) -> Result<Command, Error> {
    if !is_char('*', &mut cmd_seq) {
        return Err(Error);
    }

    match extract_number(&mut cmd_seq) {
        2 => {
            let (cmd, key) = extract_cmd_key(&mut cmd_seq).ok_or(Error)?;
            match cmd.as_str() {
                "GET" => {
                    return Ok(Command::GET { key: key });
                }
                "DEL" => {
                    return Ok(Command::DEL { key: key });
                }
                _ => return Err(Error),
            }
        }
        3 => {
            let (cmd, key) = extract_cmd_key(&mut cmd_seq).ok_or(Error)?;
            if cmd.as_str() != "SET" {
                return Err(Error);
            }
            let value: String = extract_value(&mut cmd_seq).ok_or(Error)?;
            Ok(Command::SET {
                key: key,
                value: value,
            })
        }
        _ => return Err(Error),
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

fn extract_cmd_key(chars: &mut Chars<'_>) -> Option<(String, String)> {
    if !skip_new_line(chars) {
        return None;
    }
    if !is_char('$', chars) {
        return None;
    }
    let cmd_len = extract_number(chars);
    if !skip_new_line(chars) {
        return None;
    }
    let cmd = extract_string(cmd_len, chars);
    if !skip_new_line(chars) {
        return None;
    }
    if !is_char('$', chars) {
        return None;
    }
    let key_len = extract_number(chars);
    if !skip_new_line(chars) {
        return None;
    }
    let key = extract_string(key_len, chars);
    Some((cmd, key))
}

fn extract_value(chars: &mut Chars<'_>) -> Option<String> {
    if !skip_new_line(chars) {
        return None;
    }
    if !is_char('$', chars) {
        return None;
    }
    let value_len = extract_number(chars);
    if !skip_new_line(chars) {
        return None;
    }
    let value = extract_string(value_len, chars);
    Some(value)
}
