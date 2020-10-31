use log::*;

use crate::grammar::Parser;
use crate::grammar::Statement;
use crate::syntax::SyntaxKind::*;

fn participant_name(parser: &mut Parser) {
    assert_eq!(parser.current(), IDENTIFIER);
    parser.start_node(IDENTIFIER);
    parser.consume(IDENTIFIER);
    parser.finish_node();
}

pub(crate) fn participant_decl(parser: &mut Parser) -> Statement {
    assert_eq!(parser.current(), PARTICIPANT_KW);
    trace!("Starting decl node");
    parser.start_node(PARTICIPANT_DECL);
    parser.consume_token(); // 'participant'

    parser.skip_ws();
    if parser.at(IDENTIFIER) {
        participant_name(parser);
    }

    // expect an identifier
    loop {
        match parser.current() {
            EOF => {
                parser.finish_node();
                return Statement::Ok;
            }
            EOL => {
                parser.consume_token();
                parser.finish_node();
                return Statement::Ok;
            }
            _ => {
                parser.consume_token();
            }
        }
    }
}

mod tests {
    #[allow(unused_imports)]
    use log::*;

    #[test]
    fn test_participant_decl() {
        env_logger::init();
        let text = "\tparticipant kelly\nparticipant bob\n";
        let root = crate::parse(text).root();
        let res = root
            .participant_decls()
            .filter_map(|it| it.participant_name())
            .map(|it| it.identifier())
            .collect::<Vec<_>>();
        trace!("Participant Names - {:?}", res);

        let res = root
            .participant_decls()
            .map(|it| it.position())
            .collect::<Vec<_>>();
        trace!("DECL Positions - {:?}", res);

        let res = root
            .participant_decls()
            .filter_map(|it| it.participant_name())
            .map(|it| it.position())
            .collect::<Vec<_>>();
        trace!("Participant Positions - {:?}", res);
        // let res = root
        //     .participant_decls()
        //     .map(|it| it.eval())
        //     .collect::<Vec<_>>();
        // trace!("{:?}", res);
        // assert_eq!(res, vec![Some(6)])
    }
}
