use nom_locate::LocatedSpan;
use nom::AsBytes;

pub type Span<'a> = LocatedSpan<&'a [u8]>;

pub trait ToS {
    fn to_s(&self) -> String;
}

impl<'a> ToS for Span<'a> {
    fn to_s (&self) -> String {
        String::from_utf8(self.as_bytes().to_vec()).unwrap()
    }
}

#[derive(Debug)]
pub struct Connection<'a> {
    pub pos: Span<'a>,
    //pub symbol: String,
}

#[derive(Debug)]
pub struct ParticipantConnection<'a> {
    pub span: Span<'a>,
    pub p1: Participant<'a>,
    pub connection: Connection<'a>,
    pub p2: Participant<'a>,
    pub desc: Option<String>,
}

#[derive(Debug)]
pub struct Participant<'a> {
    pub pos: Span<'a>,
    pub name: String,
}

#[derive(Debug)]
pub struct ParticipantDecl<'a> {
    pub pos: Span<'a>,
    pub name_pos: Span<'a>,
    pub stereotype: String,
    pub participant: Participant<'a>,
}

#[derive(Debug)]
pub struct PumlDoc<'a> {
    pub startuml_pos: Span<'a>,
    pub enduml_pos: Span<'a>,
    pub participant_decls: Vec<ParticipantDecl<'a>>,
}

impl Default for PumlDoc<'_> {
    fn default() -> Self {
        PumlDoc {
            startuml_pos: Span::new(&b""[..]),
            enduml_pos: Span::new(&b""[..]),
            participant_decls: vec![],
        }
    }
}
