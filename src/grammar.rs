#[allow(dead_code)]
use log::*;

//use num_derive::{FromPrimitive, ToPrimitive};
use rowan::GreenNode;
use rowan::GreenNodeBuilder;
use rowan::SmolStr;

use crate::ast;
use crate::syntax;
use crate::parsing::sequence::participant_decl;


#[allow(dead_code)]
pub(crate) enum Statement {
    /// A statement was successfully parsed
    Ok,
    /// Nothing was parsed, as no significant tokens remained
    UnexpectedEof,
    // An unexpected EOL was found
    // UnexpectedEol,
}

#[allow(dead_code)]
pub(crate) struct Parsed {
    green_node: GreenNode,
    #[allow(unused)]
    errors: Vec<String>,
}

#[allow(dead_code)]
impl Parsed {
    pub(crate) fn root(&self) -> ast::RootNode {
        ast::RootNode::cast(self.syntax()).unwrap()
    }

    pub(crate) fn syntax(&self) -> ast::SyntaxNode {
        ast::SyntaxNode::new_root(self.green_node.clone())
    }
}

#[allow(dead_code)]
pub(crate) struct Parser {
    /// input tokens, including whitespace, in *reverse* order.
    tokens: Vec<(syntax::SyntaxKind, SmolStr)>,
    /// the in-progress tree.
    builder: GreenNodeBuilder<'static>,
    /// the list of syntax errors we've accumulated so far.
    errors: Vec<String>,
}


use syntax::SyntaxKind::*;
impl Parser {

    pub(crate) fn new(tokens: Vec<(syntax::SyntaxKind, SmolStr)>) -> Self {
        Parser {
            tokens,
            builder: rowan::GreenNodeBuilder::new(),
            errors: Vec::new(),
        }
    }

    pub(crate) fn start_node(&mut self, kind: syntax::SyntaxKind) {
        self.builder.start_node(kind.into());
    }

    pub(crate) fn finish_node(&mut self) {
        self.builder.finish_node();
    }

    pub(crate) fn parse(mut self) -> Parsed {
        // Make sure that the root node covers all source
        self.builder.start_node(ROOT.into());
        while !self.at(EOF) {
            match self.statement() {
                Statement::UnexpectedEof => {
                    break;
                }
                // Statement::UnexpectedEol => {
                //     self.builder.start_node(ERROR.into());
                //     self.errors.push("unexpected EOL".to_string());
                //     self.consume_token(); // be sure to chug along in case of error
                //     self.builder.finish_node();
                // }
                Statement::Ok => (),
            }
        }
        // Don't forget to eat *trailing* whitespace
        self.skip_ws();
        self.builder.finish_node();
        // Turn the builder into a GreenNode
        Parsed {
            green_node: self.builder.finish(),
            errors: self.errors,
        }
    }

    pub fn consume(&mut self, kind: syntax::SyntaxKind) {
        assert_eq!(self.current(), kind);
        self.consume_token()
    }

    #[allow(dead_code)]
    pub(crate) fn consume_token(&mut self) {
        let (kind, text) = self.tokens.pop().unwrap();
        trace!(target: "parser", "Consuming token '{}' of type {:?}", text, kind);
        self.builder.token(kind.into(), text)
    }

    pub(crate) fn current(&self) -> syntax::SyntaxKind {
        self.tokens.last().map(|(kind, _)| *kind).unwrap_or(EOF)
    }

    pub(crate) fn skip_ws(&mut self) {
        while self.current() == WHITESPACE {
            self.consume_token()
        }
    }

    #[allow(dead_code)]
    fn statement<'a>(&'a mut self) -> Statement {
        // Eat leading whitespace
        self.skip_ws();
        // Either a list, an atom, a closing paren,
        // or an eof.
        trace!("Start Statement");
        let stmt = match self.current() {
            EOF => Statement::UnexpectedEof,
            ERROR => {
                self.start_node(ERROR);
                self.consume(ERROR);
                self.finish_node();
                Statement::Ok
            },
            PARTICIPANT_KW => participant_decl(self),
            _ => unreachable!(format!("uh oh {:?}", self.current()))
        };
        trace!("Finish Statement");
        stmt
    }

    pub fn at(&self, kind: syntax::SyntaxKind) -> bool {
        trace!("At {:?} ? current is {:?}", kind, self.current());
        if self.current() == kind {
            return true;
        }
        return false;
    }

}


