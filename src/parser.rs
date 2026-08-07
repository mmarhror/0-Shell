use crate::error::ShellError;

pub fn parse(inp: &str) -> Result<(String, Vec<String>), ShellError> {
    let mut parts = Vec::new();
    let mut buf = String::new();
    let mut opened: Option<char> = None;
    let mut has_quotes = false;

    for ch in inp.chars() {
        match ch {
            ' ' => {
                if opened.is_some() {
                    buf.push(ch);
                } else if !buf.is_empty() || has_quotes {
                    parts.push(buf.clone());
                    buf.clear();
                    has_quotes = false;
                }
            }

            '"' | '\'' => {
                match opened {
                    None => {
                        opened = Some(ch);
                        has_quotes = true;
                    }
                    Some(q) if q == ch => {
                        opened = None;
                    }
                    _ => buf.push(ch),
                }
            }
            _ => {
                buf.push(ch);
            }
        }
    }

    if opened.is_some() {
        return Err(ShellError::one("osh", "Unclosed quote"));
    }

    if !buf.is_empty() || has_quotes {
        parts.push(buf);
    }

    let cmd = parts.remove(0);

    Ok((cmd, parts))
}
