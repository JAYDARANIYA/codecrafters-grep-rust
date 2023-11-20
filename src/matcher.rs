#[derive(Debug)]
pub enum RegexPattern {
    Char(char),
    Digit, // \d
    Word,  // \w
}

pub mod matcher {

    use super::RegexPattern;

    pub fn match_pattern(input_line: &str, pattern: &str) -> bool {
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
            }
        }
        true
    }
}
