use super::model;

#[allow(unused_imports)]
use fluid::prelude::*;
use model::ToS;
use nom::branch::alt;
use nom::bytes::complete::{tag, take_while, take_while1};
use nom::character::complete::{char, space0, space1};
use nom::character::is_alphabetic;
use nom::combinator::{map, opt};
use nom::multi::many0;
use nom::{AsBytes, IResult};
use nom_locate::position;

#[allow(dead_code)]
enum PumlLineEvent<'a> {
    ParticipantDecl(model::ParticipantDecl<'a>),
    ParticipantConnection(model::ParticipantConnection<'a>),
}

#[derive(Debug)]
enum PumlObject<'a> {
    ParticipantType(String, model::Span<'a>),
    Participant(model::Participant<'a>, model::Span<'a>),
    Unrecognized(String, model::Span<'a>),
}

type Statement<'a> = (bool, Vec<PumlObject<'a>>);
/*enum Statement<'a> {
    Valid(Vec<PumlObject<'a>>),
    Invalid(Vec<PumlObject<'a>>),
}*/

#[allow(dead_code)]
fn participant_type(input: model::Span) -> IResult<model::Span, PumlObject> {
    let (input, value) = tag("participant")(input)?;
    let (input, pos) = position(input)?;
    Ok((input, PumlObject::ParticipantType(value.to_s(), pos)))
}

#[allow(dead_code)]
fn almost_participant_decl(input: model::Span) -> IResult<model::Span, Statement> {
    let valid = true;
    let mut stmt = Vec::<PumlObject>::new();

    let (input, _) = space0(input)?;
    let (input, pt) = participant_type(input)?;
    stmt.push(pt);

    let (input, _) = opt(space1)(input)?;
    let (valid, input, waiting_for) = match participant_name(input) {
        Ok((input, p)) => {
            let (input, name_pos) = position(input)?;
            stmt.push(PumlObject::Participant(p, name_pos));
            (valid, input, None)
        }
        Err(_) => (false, input, Some("participant name"))
    };

    let (input, _trash) = take_while(|ch| ch != 10)(input)?;
    let (input, _trash_pos) = position(input)?;
    let (input, _eol) = char('\n')(input)?;
    if _trash.as_bytes().len() > 0 {
        stmt.push(PumlObject::Unrecognized(_trash.to_s(), _trash_pos));
    }
    Ok((input, (valid, stmt)))
}

#[allow(dead_code)]
fn participant_decl(input: model::Span) -> IResult<model::Span, model::ParticipantDecl> {
    let (input, _) = space0(input)?;
    let (input, _value) = tag("participant")(input)?;
    let (input, pos) = position(input)?;

    let (input, _) = space1(input)?;

    let (input, participant) = participant_name(input)?;
    let (input, name_pos) = position(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char('\n')(input)?;

    Ok((
        input,
        model::ParticipantDecl {
            pos,
            name_pos,
            stereotype: String::from("participant"),
            participant,
        },
    ))
}

#[allow(dead_code)]
fn connection(input: model::Span) -> IResult<model::Span, model::Connection> {
    let (input, _) = tag("->")(input)?;
    let (input, pos) = position(input)?;
    Ok((input, model::Connection { pos }))
}

#[allow(dead_code)]
fn connection_description(input: model::Span) -> IResult<model::Span, String> {
    let (input, _) = char(':')(input)?;
    let (input, _) = space0(input)?;
    let (input, value) = take_while(|ch| ch != 10)(input)?;
    Ok((input, value.to_s()))
}

#[allow(dead_code)]
fn participant_connection(
    input: model::Span,
) -> IResult<model::Span, model::ParticipantConnection> {
    let (input, _) = space0(input)?;

    let (input, p1) = participant_name(input)?;
    let (input, _) = space0(input)?;
    let (input, connection) = connection(input)?;
    let (input, _) = space0(input)?;
    let (input, p2) = participant_name(input)?;
    let (input, _) = space0(input)?;

    let (input, desc) = opt(connection_description)(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char('\n')(input)?;

    Ok((
        input,
        model::ParticipantConnection {
            span: input,
            p1,
            connection,
            p2,
            desc,
        },
    ))
}

fn participant_name(input: model::Span) -> IResult<model::Span, model::Participant> {
    //Alice -> Bob: Authentication Request
    let (input, value) = take_while1(is_alphabetic)(input)?;
    let (input, pos) = position(input)?;
    // TODO this can't be how this is done
    let name = std::str::from_utf8(value.as_bytes()).unwrap().to_string();
    Ok((input, model::Participant { pos, name }))
}

#[allow(dead_code)]
fn puml_doc(input: model::Span) -> IResult<model::Span, model::PumlDoc> {
    let (input, _) = space0(input)?;
    let (input, _) = tag("@startuml")(input)?;
    let (input, startuml_pos) = position(input)?;
    let (input, _) = char('\n')(input)?;

    let puml_events = alt((
        map(participant_decl, PumlLineEvent::ParticipantDecl),
        map(participant_connection, PumlLineEvent::ParticipantConnection),
    ));
    let (input, events) = many0(puml_events)(input)?;

    let (input, _) = space0(input)?;
    let (input, _) = tag("@enduml")(input)?;
    let (input, enduml_pos) = position(input)?;
    let (input, _) = char('\n')(input)?;

    let mut doc = model::PumlDoc {
        startuml_pos,
        enduml_pos,
        ..model::PumlDoc::default()
    };

    for e in events {
        match e {
            PumlLineEvent::ParticipantDecl(p) => {
                doc.participant_decls.push(p);
            }
            PumlLineEvent::ParticipantConnection(_c) => {}
        }
    }
    Ok((input, doc))
}

mod tests {
    use super::*;
    #[test]
    fn basic_test() {
        let doc = &b"@startuml
        participant Alice
        participant Bob
        Alice -> Bob: test
        @enduml\n"[..];
        //Bob --> Alice: Authentication Response \
        //Alice -> Bob: Another authentication Request \
        //Alice <-- Bob: Another authentication Response \
        //@enduml \
        //";
        let result: IResult<model::Span, model::PumlDoc> = puml_doc(model::Span::new(doc));
        print_err(&result);
        match result {
            Ok((span, _pumldoc)) => {
                span.as_bytes().len().should().be_equal_to(0);
            }
            _ => {}
        }
    }

    #[test]
    fn test_participant_conn() {
        let doc = &b"Alice -> Bob : test\n"[..];
        participant_connection(model::Span::new(doc)).unwrap();
    }

    #[test]
    fn test_participant_decl() {
        let doc = &b"participant Alice\n"[..];
        participant_decl(model::Span::new(doc)).unwrap();
    }

    #[allow(unused_macros)]
    macro_rules! extract {
        ($value: expr, $variant:ident) => {
            match $value {
                $variant(a,_) => Some(a),
                _ => None,
            }
        };
    }

    #[test]
    fn test_incomplete_participant_decl() {
        use PumlObject::*;
        let doc = &b"participant\n"[..];
        let (_, stmt) = almost_participant_decl(model::Span::new(doc)).unwrap();
        assert_eq!(stmt.0, false);

        assert_eq!(stmt.1.len(), 1 as usize);
        assert_eq!("participant", extract!(&stmt.1[0], ParticipantType).unwrap());
    }

    #[test]
    fn test_complete_participant_decl() {
        use PumlObject::*;
        let doc = &b"participant harry\n"[..];
        let (_input, stmt) = almost_participant_decl(model::Span::new(doc)).unwrap();
        assert_eq!(stmt.0, true);
        assert_eq!(stmt.1.len(), 2 as usize);
        assert_eq!("participant", extract!(&stmt.1[0], ParticipantType).unwrap());
        assert_eq!("harry", extract!(&stmt.1[1], Participant).unwrap().name);
    }

    #[test]
    fn test_participant_name() {
        let doc = &b"Alice"[..];
        participant_name(model::Span::new(doc)).unwrap();
    }

    #[test]
    fn test_connection() {
        let doc = &b"->"[..];
        connection(model::Span::new(doc)).unwrap();
    }

    #[allow(dead_code)]
    fn print_err(result: &IResult<model::Span, model::PumlDoc>) {
        match result {
            Err(nom::Err::Error((input, _x))) | Err(nom::Err::Failure((input, _x))) => {
                println!(
                    "Error @ line {}, offset {}, fragment: \n {}",
                    input.location_line(),
                    input.location_offset(),
                    String::from_utf8(input.fragment().to_vec()).unwrap()
                );
                assert!(false);
            }
            _ => {}
        }
    }

}
