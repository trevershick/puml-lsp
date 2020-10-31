use super::syntax::SyntaxKind::{self, *};
use log::*;
use m_lexer;
use num::ToPrimitive;
use rowan::SmolStr;

/// Let's use the primitive discriminant from the SyntaxKind enum
/// as the lexer token (which is internally a u16)
impl From<SyntaxKind> for m_lexer::TokenKind {
    fn from(kind: SyntaxKind) -> m_lexer::TokenKind {
        //trace!(
        //    "SyntaxKind {:?} to m_lexer::TokenKind {:?}",
        //    kind,
        //    kind.to_u16()
        //);
        m_lexer::TokenKind(kind.to_u16().unwrap())
    }
}

impl From<SyntaxKind> for (m_lexer::TokenKind, &str) {
    fn from(kind: SyntaxKind) -> (m_lexer::TokenKind, &'static str) {
        (kind.into(), kind.into())
    }
}

/// Split the input string into a flat list of tokens
/// (such as L_PAREN, WORD, and WHITESPACE)
pub fn lex<'a>(text: &str) -> Vec<(SyntaxKind, SmolStr)> {
    trace!("lex({})", text);

    let tokens: [(m_lexer::TokenKind, &str); 4] = [
        PARTICIPANT_KW.into(),
        IDENTIFIER.into(),
        EOL.into(),
        WHITESPACE.into(),
    ];
    // trace!("Tokens {:?}", tokens);
    let lexer = m_lexer::LexerBuilder::new()
        .error_token(ERROR.into())
        .tokens(&tokens)
        .build();

    // convert m_lexer's token kind to our syntax kind
    fn to_syntax_kind(t: m_lexer::TokenKind) -> SyntaxKind {
        // trace!("Lexer token convert to syntax kind {}", t.0);
        return num::FromPrimitive::from_u16(t.0).unwrap();
    }
    lexer
        .tokenize(text)
        .into_iter()
        .map(|t| (t.len, to_syntax_kind(t.kind)))
        .scan(0usize, |start_offset, (len, kind)| {
            let s: SmolStr = text[*start_offset..*start_offset + len].into();
            *start_offset += len;
            Some((kind, s))
        })
        .collect()
}
