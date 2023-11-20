#[derive(Debug)]
pub enum RegexPattern {
    Char(char),
    Digit,                      // \d
    Word,                       // \w
    PositiveCharSet(Vec<char>), // [abc]
    NegativeCharSet(Vec<char>), // [^abc]
    Start,                      // ^
    End,                        // $
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

        let regex_pattern = parse_pattern(pattern.chars(), Vec::new());

        match_with_pattern(input_line, &regex_pattern)
    }

    fn parse_pattern(
        mut pattern: std::str::Chars,
        mut tokens: Vec<RegexPattern>,
    ) -> Vec<RegexPattern> {
        match pattern.next() {
            Some('\\') => match pattern.next() {
                Some('\\') => {
                    tokens.push(RegexPattern::Char('\\'));
                    parse_pattern(pattern, tokens)
                }
                Some('d') => {
                    tokens.push(RegexPattern::Digit);
                    parse_pattern(pattern, tokens)
                }
                Some('w') => {
                    tokens.push(RegexPattern::Word);
                    parse_pattern(pattern, tokens)
                }
                _ => panic!("Unhandled escape sequence: \\{:?}", pattern),
            },
            // line anchor
            Some('^') => {
                // line anchor should only be at the start of the pattern or it's considered a normal character
                if tokens.len() > 0 {
                    tokens.push(RegexPattern::Char('^'));
                    parse_pattern(pattern, tokens)
                } else {
                    tokens.push(RegexPattern::Start);
                    // if it is line anchor, we don't need to parse the rest of the pattern
                    tokens
                }
            }
            Some('$') => {
                tokens.push(RegexPattern::End);
                tokens
            }
            Some('[') => {
                let mut char_set = Vec::new();
                let mut is_negative = false;

                match pattern.next() {
                    Some('^') => {
                        is_negative = true;
                        match pattern.next() {
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
                    match pattern.next() {
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

                parse_pattern(pattern, tokens)
            }
            Some(c) => {
                tokens.push(RegexPattern::Char(c));
                parse_pattern(pattern, tokens)
            }
            None => tokens,
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
            }
        }
        true
    }
}
