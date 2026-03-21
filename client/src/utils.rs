fn find_char_bytes_len(ch: &char) -> i32 {
    let mut b = [0; 4];
    ch.encode_utf8(&mut b);
    let mut clen = 0;
    for a in b.iter() {
        clen += match a {
            0 => 0,
            _ => 1,
        }
    }
    clen
}

pub fn truncate_text(text: &str, tlen: usize) -> &str {
    if text.len() <= tlen {
        return text;
    }

    let c = text.chars().nth(tlen);
    match c {
        Some(s) => match char::is_whitespace(s) {
            true => text.split_at(tlen).0,
            false => {
                let chars: Vec<_> = text.chars().collect();
                let truncated = chars.split_at(tlen);
                let mut first_len = 0;
                for ch in truncated.0.iter() {
                    first_len += find_char_bytes_len(ch);
                }

                let mut prev_ws = first_len - 1;
                for ch in truncated.0.iter().rev() {
                    if char::is_whitespace(*ch) {
                        break;
                    }
                    prev_ws -= find_char_bytes_len(ch);
                }

                let mut next_ws = first_len + 1;
                for ch in truncated.1.iter() {
                    let mut b = [0; 4];
                    ch.encode_utf8(&mut b);
                    if char::is_whitespace(*ch) {
                        break;
                    }
                    next_ws += find_char_bytes_len(ch);
                }

                match next_ws > prev_ws && prev_ws > 0 {
                    true => text.split_at(prev_ws as usize).0,
                    false => text.split_at(next_ws as usize).1,
                }
            }
        },
        None => text,
    }
}