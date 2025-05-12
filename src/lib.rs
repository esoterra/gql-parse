use bumpalo::Bump;
use parse::Parse;

mod ast;
mod lexer;
mod parse;

pub fn parse<'q, 'src>(b: &'q Bump, source: &'src str) -> Result<&'q ast::Path<'q, 'src>, ()> {
    let tokens = lexer::tokenize(b, source).map_err(|_| ())?;
    let mut input = parse::ParseInput::new(source, tokens);
    let path = ast::Path::parse(b, &mut input).map_err(|_| ())?;
    Ok(path)
}
