use num;
use log::*;
use num::FromPrimitive;
use num_derive::{FromPrimitive,ToPrimitive};

#[derive(FromPrimitive, ToPrimitive, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
#[repr(u16)]
pub enum SyntaxKind {
    EOF = 0,

    WHITESPACE,
    IDENTIFIER,
    EOL,
    ERROR,

    // synthetic nodes
    ROOT,

    // keywords
    PARTICIPANT_KW,

    // composite nodes
    STATEMENT,
    PARTICIPANT_DECL,
}
impl From<SyntaxKind> for &str {
    fn from(k: SyntaxKind) -> Self {
        //trace!("from for syntax kind, kind = {:?}", k);
        match k {
            SyntaxKind::PARTICIPANT_KW => r"participant",
            SyntaxKind::IDENTIFIER => r"[a-zA-Z][a-zA-Z0-9]*",
            SyntaxKind::EOL => "\n",
            SyntaxKind::WHITESPACE => r"[^\S\r\n]+",
            _ => unreachable!("You're using a syntax token kind that's not lexable"),
        }
    }
}

/// Some boilerplate is needed, as rowan settled on using its own
/// `struct SyntaxKind(u16)` internally, instead of accepting the
/// user's `enum SyntaxKind` as a type parameter.
///
/// First, to easily pass the enum variants into rowan via `.into()`:
impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Lang {}
/// Second, implementing the `Language` trait teaches rowan to convert between
/// these two SyntaxKind types, allowing for a nicer SyntaxNode API where
/// "kinds" are values from our `enum SyntaxKind`, instead of plain u16 values.
impl rowan::Language for Lang {
    type Kind = SyntaxKind;
    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        SyntaxKind::from_u16(raw.0).unwrap()
    }
    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}
