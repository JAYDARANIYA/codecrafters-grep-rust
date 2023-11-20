#[derive(Debug)]
pub enum RegexPattern {
    Char(char),
    Digit,                      // \d
    Word,                       // \w
    PositiveCharSet(Vec<char>), // [abc]
    NegativeCharSet(Vec<char>), // [^abc]
    Start,                      // ^
    End,                        // $
    Plus(char),                 // +
}

pub mod matcher {

    use super::RegexPattern;

    pub fn match_pattern(input_line: &str, pattern: &str) -> bool {
        if pattern.starts_with('^') && input_line.starts_with(&pattern[1..]) {
            return true;
        }

        if pattern.ends_with('$') && input_line.ends_with(&pattern[..pattern.len() - 1]) {
            return true;
        }

        let regex_pattern = parse_pattern(pattern);

        match_with_pattern(input_line, &regex_pattern)
    }

    fn parse_pattern(pattern: &str) -> Vec<RegexPattern> {
        let mut pattern_chars = pattern.chars();
        let mut tokens: Vec<RegexPattern> = Vec::new();

        loop {
            match pattern_chars.next() {
                Some('\\') => match pattern_chars.next() {
                    Some('\\') => {
                        tokens.push(RegexPattern::Char('\\'));
                    }
                    Some('d') => {
                        tokens.push(RegexPattern::Digit);
                    }
                    Some('w') => {
                        tokens.push(RegexPattern::Word);
                    }
                    _ => panic!("Unhandled escape sequence: \\{:?}", pattern),
                },
                Some('+') => {
                    if tokens.len() == 0 {
                        // normal character
                        tokens.push(RegexPattern::Char('+'));
                    } else {
                        // get the last token
                        let last_token = tokens.pop().expect("No token to apply + operator to");
                        match last_token {
                            RegexPattern::Char(c) => {
                                tokens.push(RegexPattern::Plus(c));
                            }
                            _ => panic!("Unhandled + operator: {:?}", pattern),
                        }
                    }
                }
                Some('^') => {
                    // line anchor should only be at the start of the pattern or it's considered a normal character
                    if tokens.len() > 0 {
                        tokens.push(RegexPattern::Char('^'));
                    } else {
                        tokens.push(RegexPattern::Start);
                        // if it is line anchor, we don't need to parse the rest of the pattern
                        return tokens;
                    }
                }
                Some('$') => {
                    tokens.push(RegexPattern::End);
                    return tokens;
                }
                Some('[') => {
                    let mut char_set = Vec::new();
                    let mut is_negative = false;

                    match pattern_chars.next() {
                        Some('^') => {
                            is_negative = true;
                            match pattern_chars.next() {
                                Some(']') => {}
                                Some(c) => char_set.push(c),
                                None => panic!("Unterminated character set: {:?}", pattern),
                            }
                        }
                        Some(']') => {}
                        Some(c) => char_set.push(c),
                        None => panic!("Unterminated character set: {:?}", pattern),
                    }

                    loop {
                        match pattern_chars.next() {
                            Some(']') => break,
                            Some(c) => char_set.push(c),
                            None => panic!("Unterminated character set: {:?}", pattern),
                        }
                    }

                    if is_negative {
                        tokens.push(RegexPattern::NegativeCharSet(char_set));
                    } else {
                        tokens.push(RegexPattern::PositiveCharSet(char_set));
                    }

                    continue;
                }
                Some(c) => {
                    tokens.push(RegexPattern::Char(c));
                }
                None => {
                    return tokens;
                }
            }
        }
    }

    fn match_with_pattern(input_line: &str, pattern: &[RegexPattern]) -> bool {
        let mut input_bytes = input_line.as_bytes();
        let mut pattern_iter = pattern.iter();

        while let Some(pat) = pattern_iter.next() {
            match pat {
                RegexPattern::Char(c) => {
                    if input_bytes.first() == Some(&(*c as u8)) {
                        input_bytes = &input_bytes[1..];
                    } else {
                        return false;
                    }
                }
                RegexPattern::Digit => {
                    if let Some((index, _)) = input_bytes
                        .iter()
                        .enumerate()
                        .find(|(_, &b)| b.is_ascii_digit())
                    {
                        input_bytes = &input_bytes[index + 1..];
                    } else {
                        return false;
                    }
                }
                RegexPattern::Word => {
                    if let Some((index, _)) = input_bytes
                        .iter()
                        .enumerate()
                        .find(|(_, &b)| b.is_ascii_alphanumeric() || b == ('_' as u8))
                    {
                        input_bytes = &input_bytes[index + 1..];
                    } else {
                        return false;
                    }
                }
                RegexPattern::PositiveCharSet(char_set) => {
                    // Positive character groups match any character that is present within a pair of square brackets
                    // Example: [abc] matches any character that is either a, b, or c

                    let mut matched = false;
                    for c in char_set {
                        if input_bytes.first() == Some(&(*c as u8)) {
                            matched = true;
                            break;
                        }
                    }

                    if matched {
                        input_bytes = &input_bytes[1..];
                    } else {
                        return false;
                    }
                }
                RegexPattern::NegativeCharSet(char_set) => {
                    // Negative character groups match any character that is not present within a pair of square brackets
                    // Example: [^abc] matches any character that is not a, b, or c

                    let mut matched = false;
                    for c in char_set {
                        if input_bytes.first() == Some(&(*c as u8)) {
                            matched = true;
                            break;
                        }
                    }

                    if matched {
                        return false;
                    } else {
                        input_bytes = &input_bytes[1..];
                    }
                }
                RegexPattern::Start => {
                    return false;
                }
                RegexPattern::End => {
                    return false;
                }
                RegexPattern::Plus(c) => {
                    // check input line for the character c
                    if input_bytes.first() == Some(&(*c as u8)) {
                        // if it is present, keep consuming the character
                        input_bytes = &input_bytes[1..];

                        // check if the next character is also the same
                        while input_bytes.first() == Some(&(*c as u8)) {
                            input_bytes = &input_bytes[1..];
                        }
                    } else {
                        return false;
                    }
                }
            }
        }
        true
    }
}
