use super::syntax::SyntaxKind::*;
use log::*;

macro_rules! ast_node {
    ($ast:ident, $kind:ident) => {
        #[derive(Debug,PartialEq, Eq, Hash)]
        #[repr(transparent)]
        pub struct $ast(SyntaxNode);
        impl $ast {
            #[allow(unused)]
            pub(crate) fn position(&self) -> rowan::TextRange {
                self.0.text_range()
            }
            #[allow(dead_code)]
            pub(crate) fn syntax(&self) -> &SyntaxNode {
                &self.0
            }
            #[allow(unused)]
            pub fn cast(node: SyntaxNode) -> Option<Self> {
                if node.kind() == $kind {
                    Some(Self(node))
                } else {
                    trace!("returning none");
                    None
                }
            }
        }
    };
}
pub type SyntaxNode = rowan::SyntaxNode<super::syntax::Lang>;
#[allow(unused)]
pub type SyntaxToken = rowan::SyntaxToken<super::syntax::Lang>;
#[allow(unused)]
pub type SyntaxElement = rowan::NodeOrToken<SyntaxNode, SyntaxToken>;
ast_node!(RootNode, ROOT);
ast_node!(ParticipantDeclNode, PARTICIPANT_DECL);
ast_node!(IdentifierNode, IDENTIFIER);

#[allow(dead_code)]
pub struct StatementNode(SyntaxNode);

#[allow(dead_code)]
pub enum StatementNodeKind {
    ParticipantDeclNodeKind(ParticipantDeclNode),
}

impl RootNode {
    #[allow(dead_code)]
    pub fn statements(&self) -> impl Iterator<Item = StatementNode> + '_ {
        self.0.children().filter_map(StatementNode::cast)
    }
    #[allow(dead_code)]
    pub fn participant_decls(&self) -> impl Iterator<Item = ParticipantDeclNode> + '_ {
        self.0.children().filter_map(ParticipantDeclNode::cast)
    }
}
impl IdentifierNode {
    #[allow(dead_code)]
    pub fn identifier(&self) -> rowan::SyntaxText {
        self.0.text().clone()
    }
}

impl ParticipantDeclNode {
    pub fn participant_name(&self) -> Option<IdentifierNode> {
        self.syntax().children().find_map(IdentifierNode::cast)
    }

    // #[allow(dead_code)]
    // pub fn text(&self) -> &rowan::SmolStr {
    //     trace!("child count is {:?}", self.0.children().count());
    //     self.0.children().for_each(|c| {
    //         trace!("child => {}", c.text())
    //     });
    //     trace!("green child count is {:?}", self.0.green().children().count());
    //     self.0.green().children().for_each(|c| {
    //         match c {
    //             rowan::NodeOrToken::Token(p) => trace!("green token => {}", p.text()),
    //             rowan::NodeOrToken::Node(p) => trace!("green node => {:?}", p),
    //             _ => unreachable!(),
    //         }
    //     });
    //     match &self.0.green().children().next() {
    //         Some(rowan::NodeOrToken::Token(token)) => token.text(),
    //         _ => unreachable!(),
    //     }
    // }
}

// statement node is a wrapper around 'n' different types of statements.
// as such, its implementation is 'special'
impl StatementNode {
    #[allow(dead_code)]
    fn cast(node: SyntaxNode) -> Option<Self> {
        if ParticipantDeclNode::cast(node.clone()).is_some() {
            Some(StatementNode(node))
        } else {
            None
        }
    }

    // fn kind(&self) -> StatementNodeKind {
    //     trace!("StatementNode::kind {:?}", self.0.kind());
    //     ParticipantDeclNode::cast(self.0.clone())
    //         .map(StatementNodeKind::ParticipantDeclNodeKind)
    //         //    //.or_else(|| List::cast(self.0.clone()).map(StatementNodeKind::List))
    //         .unwrap()
    // }
}
