use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum RegexPattern {
    Char(char),
    Digit,                                                               // \d
    Word,                                                                // \w
    PositiveCharSet(Vec<char>),                                          // [abc]
    NegativeCharSet(Vec<char>),                                          // [^abc]
    Start,                                                               // ^
    End,                                                                 // $
    Plus(char),                                                          // +
    ZeroOrOne(char),                                                     // ?
    Dot,                                                                 // .
    Alternative(Rc<Box<Vec<RegexPattern>>>, Rc<Box<Vec<RegexPattern>>>), // (a|b)
}

pub mod matcher {

    use std::rc::Rc;

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
                Some('?') => {
                    if tokens.len() == 0 {
                        // normal character
                        tokens.push(RegexPattern::Char('?'));
                    } else {
                        // get the last token
                        let last_token = tokens.pop().expect("No token to apply ? operator to");
                        match last_token {
                            RegexPattern::Char(c) => {
                                tokens.push(RegexPattern::ZeroOrOne(c));
                            }
                            _ => panic!("Unhandled ? operator: {:?}", pattern),
                        }
                    }
                }
                Some('.') => {
                    tokens.push(RegexPattern::Dot);
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
                Some('(') => {
                    // Example: (pattern_a|pattern_b)
                    let mut pattern_a = String::new();
                    let mut pattern_b = String::new();

                    let mut is_pattern_a = true;

                    loop {
                        match pattern_chars.next() {
                            Some('|') => {
                                is_pattern_a = false;
                                continue;
                            }
                            Some(')') => break,
                            Some(c) => {
                                if is_pattern_a {
                                    pattern_a.push(c);
                                } else {
                                    pattern_b.push(c);
                                }
                            }
                            None => panic!("Unterminated group: {:?}", pattern),
                        }
                    }

                    // let pattern_a = parse_pattern(&pattern_a);
                    // let pattern_b = parse_pattern(&pattern_b);

                    tokens.push(RegexPattern::Alternative(
                        Rc::new(Box::new(parse_pattern(&pattern_a))),
                        Rc::new(Box::new(parse_pattern(&pattern_b))),
                    ));
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
        let mut pattern_iter = pattern.iter().peekable();

        while let Some(pat) = pattern_iter.next() {
            match pat {
                RegexPattern::Char(c) => {
                    if input_bytes.first() == Some(&(*c as u8)) {
                        input_bytes = &input_bytes[1..];
                    } else {
                        if let Some(&RegexPattern::ZeroOrOne(_)) = pattern_iter.peek() {
                            input_bytes = &input_bytes[2..];
                            pattern_iter.next(); // Skip the ZeroOrOne pattern
                        } else {
                            return false;
                        }
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
                RegexPattern::ZeroOrOne(c) => {
                    // check input line for the character c
                    if input_bytes.first() == Some(&(*c as u8)) {
                        // if it is present, keep consuming the character
                        input_bytes = &input_bytes[1..];
                    }
                }
                RegexPattern::Dot => {
                    if input_bytes.first().is_some() {
                        input_bytes = &input_bytes[1..];
                    } else {
                        return false;
                    }
                }

                RegexPattern::Alternative(pattern_1, pattern_2) => {
                    let input_bytes_1 = input_bytes;
                    let input_bytes_2 = input_bytes;

                    let pattern_1: Vec<RegexPattern> = pattern_1.as_ref().as_ref().clone();
                    let pattern_2: Vec<RegexPattern> = pattern_2.as_ref().as_ref().clone();

                    if let true = match_with_pattern(
                        std::str::from_utf8(input_bytes_1).unwrap(),
                        &pattern_1
                    ) {
                        input_bytes = input_bytes_1;
                    } else if let true = match_with_pattern(
                        std::str::from_utf8(input_bytes_2).unwrap(),
                        &pattern_2,
                    ) {
                        input_bytes = input_bytes_2;
                    } else {
                        return false;
                    }
                }
            }
        }
        true
    }
}
