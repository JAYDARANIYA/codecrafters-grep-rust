#[derive(Debug)]
pub enum RegexPattern {
    Char(char),
    Digit, // \d
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
        match pattern.first() {
            None => true,
            Some(RegexPattern::Char(c)) => {
                if input_line.starts_with(*c) {
                    match_with_pattern(&input_line[1..], &pattern[1..])
                } else {
                    false
                }
            }
            Some(RegexPattern::Digit) => {
                if input_line.chars().any(|c| c.is_digit(10)) {
                    let index = input_line
                        .chars()
                        .position(|c| c.is_digit(10))
                        .expect("Digit not found")
                        + 1;
                    match_with_pattern(&input_line[index..], &pattern[1..])
                } else {
                    false
                }
            }
        }
    }
}
