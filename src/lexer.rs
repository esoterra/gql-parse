use bumpalo::{Bump, collections::Vec};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    Ident,
    Colon,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LArrow,
    RArrow,
    Dash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub offset: usize,
    pub len: usize,
}

pub type TokenData = (Token, Span);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    LessThanSuccessor(char),
    UnrecognizedTokenPrefix(char),
    EOF,
}

pub fn tokenize<'q, 'src>(b: &'q Bump, source: &'src str) -> Result<&'q [TokenData], Error> {
    let mut chars = source.char_indices().peekable();
    let mut v = Vec::new_in(b);

    while let Some((i, c)) = chars.next() {
        let t = match c {
            'a'..='z' | 'A'..='Z' | '_' => {
                let end = loop {
                    if let Some((p_i, p_c)) = chars.peek() {
                        if !matches!(p_c, 'a'..='z' | 'A'..='Z' | '_') {
                            break *p_i;
                        }
                        chars.next().unwrap();
                    }
                };
                (
                    Token::Ident,
                    Span {
                        offset: i,
                        len: end - i,
                    },
                )
            }
            ':' => (Token::Colon, Span { offset: i, len: 1 }),
            '(' => (Token::LParen, Span { offset: i, len: 1 }),
            ')' => (Token::RParen, Span { offset: i, len: 1 }),
            '[' => (Token::LBrace, Span { offset: i, len: 1 }),
            ']' => (Token::RBrace, Span { offset: i, len: 1 }),
            '<' => match chars.peek() {
                Some((_, p_c)) if *p_c == '-' => {
                    chars.next().unwrap();
                    (Token::LArrow, Span { offset: i, len: 2 })
                }
                Some((_, p_c)) => return Err(Error::LessThanSuccessor(*p_c)),
                _ => return Err(Error::EOF),
            },
            '-' => match chars.peek() {
                Some((_, p_c)) if *p_c == '>' => {
                    chars.next().unwrap();
                    (Token::RArrow, Span { offset: i, len: 2 })
                }
                _ => (Token::Dash, Span { offset: i, len: 1 }),
            },
            _ => return Err(Error::UnrecognizedTokenPrefix(c)),
        };
        v.push(t);
    }
    Ok(v.into_bump_slice())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bumpalo::vec;

    #[test]
    fn test_tokenize_empty_entity() {
        let b = Bump::new();
        let expected = vec![in &b;
            (Token::LParen, Span { offset: 0, len: 1 }),
            (Token::RParen, Span { offset: 1, len: 1 })
        ]
        .into_bump_slice();
        let actual = tokenize(&b, "()").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_tokenize_named_entity() {
        let b = Bump::new();
        let expected = vec![in &b;
            (Token::LParen, Span { offset: 0, len: 1 }),
            (Token::Ident, Span { offset: 1, len: 1 }),
            (Token::RParen, Span { offset: 2, len: 1 })
        ]
        .into_bump_slice();
        let actual = tokenize(&b, "(a)").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_tokenize_relation() {
        let b = Bump::new();
        let expected = vec![in &b;
            (Token::Dash, Span { offset: 0, len: 1 }),
            (Token::LBrace, Span { offset: 1, len: 1 }),
            (Token::Ident, Span { offset: 2, len: 1 }),
            (Token::RBrace, Span { offset: 3, len: 1 }),
            (Token::RArrow, Span { offset: 4, len: 2 })
        ]
        .into_bump_slice();
        let actual = tokenize(&b, "-[a]->").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_tokenize_path() {
        let b = Bump::new();
        let expected = vec![in &b;
            (Token::LParen, Span { offset: 0, len: 1 }),
            (Token::Ident, Span { offset: 1, len: 1 }),
            (Token::RParen, Span { offset: 2, len: 1 }),
            (Token::Dash, Span { offset: 3, len: 1 }),
            (Token::LBrace, Span { offset: 4, len: 1 }),
            (Token::Ident, Span { offset: 5, len: 1 }),
            (Token::RBrace, Span { offset: 6, len: 1 }),
            (Token::RArrow, Span { offset: 7, len: 2 }),
            (Token::LParen, Span { offset: 9, len: 1 }),
            (Token::Ident, Span { offset: 10, len: 1 }),
            (Token::RParen, Span { offset: 11, len: 1 })
        ]
        .into_bump_slice();
        let actual = tokenize(&b, "(a)-[b]->(c)").unwrap();
        assert_eq!(expected, actual);
    }
}
