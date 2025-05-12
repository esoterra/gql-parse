use bumpalo::Bump;
use bumpalo::collections::Vec;

use crate::ast;
use crate::lexer::{Span, Token, TokenData};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Next,
    AssertNext,
    ParseRelation,
}

pub struct ParseInput<'q, 'src> {
    source: &'src str,
    tokens: &'q [TokenData],
    index: usize,
}

impl<'q, 'src> ParseInput<'q, 'src> {
    pub fn new(source: &'src str, tokens: &'q [TokenData]) -> Self {
        Self {
            source,
            tokens,
            index: 0,
        }
    }

    fn peek(&self) -> Option<TokenData> {
        if self.index < self.tokens.len() {
            Some(self.tokens[self.index])
        } else {
            None
        }
    }

    fn next(&mut self) -> Result<TokenData, Error> {
        let index = self.index;
        if index < self.tokens.len() {
            self.index += 1;
            Ok(self.tokens[index])
        } else {
            Err(Error::Next)
        }
    }

    fn assert_next(&mut self, token: Token) -> Result<Span, Error> {
        let (next_token, next_span) = self.next()?;
        if next_token == token {
            Ok(next_span)
        } else {
            Err(Error::AssertNext)
        }
    }

    fn next_if(&mut self, token: Token) -> Option<Span> {
        {
            let (next_token, _) = self.peek()?;
            if next_token != token {
                return None;
            }
        }
        Some(self.next().ok()?.1)
    }

    fn get(&self, span: Span) -> &'src str {
        &self.source[span.offset..span.offset + span.len]
    }
}

pub trait Parse<'q, 'src> {
    fn parse(b: &'q Bump, input: &mut ParseInput<'q, 'src>) -> Result<&'q Self, Error>;
}

impl<'q, 'src> Parse<'q, 'src> for ast::Path<'q, 'src> {
    fn parse(b: &'q Bump, input: &mut ParseInput<'q, 'src>) -> Result<&'q Self, Error> {
        let init = ast::Entity::parse(b, input)?;
        let edges = parse_edges_rec(b, input, 1)?;
        let edges = edges
            .map(|mut edges| {
                edges.reverse();
                edges.into_bump_slice()
            })
            .unwrap_or(&[]);
        Ok(b.alloc(ast::Path { init, edges }))
    }
}

fn parse_edges_rec<'q, 'src>(
    b: &'q Bump,
    input: &mut ParseInput<'q, 'src>,
    depth: usize,
) -> Result<Option<Vec<'q, &'q ast::Edge<'q, 'src>>>, Error> {
    let token = match input.peek() {
        Some((token, _)) => token,
        None => return Ok(None),
    };
    if matches!(token, Token::Dash | Token::LArrow | Token::RArrow) {
        let edge = ast::Edge::parse(b, input)?;

        let mut v = match parse_edges_rec(b, input, depth + 1)? {
            Some(v) => v,
            None => Vec::with_capacity_in(depth, b),
        };

        v.push(edge);

        Ok(Some(v))
    } else {
        Ok(None)
    }
}

impl<'q, 'src> Parse<'q, 'src> for ast::Edge<'q, 'src> {
    fn parse(b: &'q Bump, input: &mut ParseInput<'q, 'src>) -> Result<&'q Self, Error> {
        let relation = ast::Relation::parse(b, input)?;
        let entity = ast::Entity::parse(b, input)?;
        Ok(b.alloc(ast::Edge { relation, entity }))
    }
}

impl<'q, 'src> Parse<'q, 'src> for ast::Relation<'src> {
    fn parse(b: &'q Bump, input: &mut ParseInput<'q, 'src>) -> Result<&'q Self, Error> {
        let dir = match input.next()? {
            (Token::Dash, _) => ast::Direction::None,
            (Token::LArrow, _) => ast::Direction::Left,
            _ => return Err(Error::ParseRelation),
        };

        input.assert_next(Token::LBrace)?;
        let rel = input.assert_next(Token::Ident)?;
        let variable = input.get(rel);
        input.assert_next(Token::RBrace)?;

        let dir = match (dir, input.next()?.0) {
            (ast::Direction::Left, Token::Dash) => ast::Direction::Left,
            (ast::Direction::Left, Token::RArrow) => return Err(Error::ParseRelation),
            (ast::Direction::None, Token::Dash) => ast::Direction::None,
            (ast::Direction::None, Token::RArrow) => ast::Direction::Right,
            _ => return Err(Error::ParseRelation),
        };

        let relation = b.alloc(ast::Relation { variable, dir });
        Ok(relation)
    }
}

impl<'q, 'src> Parse<'q, 'src> for ast::Entity<'q, 'src> {
    fn parse(b: &'q Bump, input: &mut ParseInput<'q, 'src>) -> Result<&'q Self, Error> {
        input.assert_next(Token::LParen)?;

        let variable = input.next_if(Token::Ident).map(|span| input.get(span));

        let labels = if matches!(input.peek(), Some((Token::Colon, _))) {
            let mut v = Vec::new_in(b);
            loop {
                input.next()?;
                let label = input.assert_next(Token::Ident)?;
                v.push(input.get(label));
                if !matches!(input.peek(), Some((Token::Colon, _))) {
                    break;
                }
            }
            v.into_bump_slice()
        } else {
            &[]
        };

        input.assert_next(Token::RParen)?;

        let entity = b.alloc(ast::Entity { variable, labels });
        Ok(entity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bumpalo::vec;

    #[test]
    fn test_assert_next() {
        let b = Bump::new();
        let v = vec![in &b;
            (Token::LParen, Span { offset: 0, len: 1 }),
            (Token::RParen, Span { offset: 1, len: 1 })
        ];
        let mut input = ParseInput {
            source: "()",
            tokens: v.into_bump_slice(),
            index: 0,
        };
        assert_eq!(
            Ok(Span { offset: 0, len: 1 }),
            input.assert_next(Token::LParen)
        );
        assert_eq!(
            Ok(Span { offset: 1, len: 1 }),
            input.assert_next(Token::RParen)
        );
    }

    #[test]
    fn test_parse_empty_entity() {
        let b = Bump::new();
        let v = vec![in &b;
            (Token::LParen, Span { offset: 0, len: 1 }),
            (Token::RParen, Span { offset: 1, len: 1 })
        ];
        let mut input = ParseInput {
            source: "()",
            tokens: v.into_bump_slice(),
            index: 0,
        };
        let entity = ast::Entity::parse(&b, &mut input).unwrap();
        assert_eq!(
            entity,
            &ast::Entity {
                variable: None,
                labels: &[]
            }
        );
    }

    #[test]
    fn test_parse_named_entity() {
        let b = Bump::new();
        let v = vec![in &b;
            (Token::LParen, Span { offset: 0, len: 1 }),
            (Token::Ident, Span { offset: 1, len: 1 }),
            (Token::RParen, Span { offset: 2, len: 1 })
        ];
        let mut input = ParseInput {
            source: "(a)",
            tokens: v.into_bump_slice(),
            index: 0,
        };
        let entity = ast::Entity::parse(&b, &mut input).unwrap();
        assert_eq!(
            entity,
            &ast::Entity {
                variable: Some("a"),
                labels: &[]
            }
        );
    }

    #[test]
    fn test_parse_relation() {
        let b = Bump::new();
        let v = vec![in &b;
            (Token::Dash, Span { offset: 0, len: 1 }),
            (Token::LBrace, Span { offset: 1, len: 1 }),
            (Token::Ident, Span { offset: 2, len: 1 }),
            (Token::RBrace, Span { offset: 3, len: 1 }),
            (Token::RArrow, Span { offset: 4, len: 2 })
        ];
        let mut input = ParseInput {
            source: "-[a]->",
            tokens: v.into_bump_slice(),
            index: 0,
        };
        let entity = ast::Relation::parse(&b, &mut input).unwrap();
        assert_eq!(
            entity,
            &ast::Relation {
                variable: "a",
                dir: ast::Direction::Right
            }
        );
    }

    #[test]
    fn test_parse_path() {
        let b = Bump::new();
        let v = vec![in &b;
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
        ];
        let mut input = ParseInput {
            source: "(a)-[b]->(c)",
            tokens: v.into_bump_slice(),
            index: 0,
        };
        let entity = ast::Path::parse(&b, &mut input).unwrap();
        assert_eq!(
            entity,
            &ast::Path {
                init: &ast::Entity {
                    variable: Some("a"),
                    labels: &[]
                },
                edges: &[&ast::Edge {
                    relation: &ast::Relation {
                        variable: "b",
                        dir: ast::Direction::Right
                    },
                    entity: &ast::Entity {
                        variable: Some("c"),
                        labels: &[]
                    }
                }]
            }
        );
    }
}
