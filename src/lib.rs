#[cfg(test)]
extern crate fluid;

extern crate env_logger;
extern crate log;
extern crate lsp_types;
extern crate nom;
extern crate nom_locate;
extern crate serde;
extern crate serde_json;
extern crate tokio;
extern crate m_lexer;
extern crate num_derive;

mod parsing;
mod grammar;
mod syntax;
mod lex;
mod ast;
mod codec;
pub mod server;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
use log::*;

#[allow(dead_code)]
pub(crate) fn parse(text: &str) -> grammar::Parsed {
    let mut tokens = crate::lex::lex(text);
    tokens.reverse();
    trace!("Tokens Length:{}", tokens.len());
    crate::grammar::Parser::new(tokens).parse()

    /*grammar::Parser {
        tokens,
        builder: rowan::GreenNodeBuilder::new(),
        errors: Vec::new(),
    }
    .parse()*/
}

mod tests {
    #[allow(unused_imports)]
    use log::*;

    #[test]
    fn always_returns_ok() {
        env_logger::init();
        let text = "a";
        let root = crate::parse(text).root();
    }
}
