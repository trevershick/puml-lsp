use bytes::Buf;
use bytes::BufMut;
use bytes::BytesMut;
use log::*;
use std::io;
use std::option::Option;
use std::result::Result;
use tokio_util::codec::{Decoder, Encoder};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum LspEvent {
    Message(String),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct LspCodec {}

impl Encoder<LspEvent> for LspCodec {
    type Error = io::Error;
    fn encode(&mut self, _data: LspEvent, buf: &mut BytesMut) -> Result<(), io::Error> {
        match _data {
            LspEvent::Message(m) => {
                let header = format!("Content-Length: {}\r\n\r\n", m.len());
                buf.reserve(header.len() + m.len());
                buf.put(header.as_bytes());
                buf.put(m.as_bytes());
            }
        }
        Ok(())
    }
}

fn parse_header<'a>(input: &'a [u8]) -> nom::IResult<&'a [u8], String> {
    use nom::error::ErrorKind;

    let (input, _) = nom::bytes::complete::tag("Content-Length: ")(input)?;
    let (input, s_len) = nom::character::complete::digit1(input)?;
    let content_length = std::str::from_utf8(s_len)
        .unwrap()
        .parse::<usize>()
        .map_err(|_x| nom::Err::Error((input, ErrorKind::Digit)))?;
    trace!(target: "parse_header", "Content-Length: {}", content_length);
    let (input, _) = nom::character::complete::crlf(input)?;
    let (input, _) = nom::character::complete::crlf(input)?;
    parse(input, content_length)
}

fn parse<'a>(input: &[u8], count: usize) -> nom::IResult<&[u8], String> {
    use nom::error::ErrorKind;
    let (input, json) = nom::bytes::complete::take(count)(input)?;
    std::str::from_utf8(json)
        .map(|u8s| (input, u8s.to_owned().clone()))
        .map_err(|_x| nom::Err::Error((input, ErrorKind::Eof)))
}

impl Decoder for LspCodec {
    type Error = io::Error;
    type Item = LspEvent;

    fn decode(&mut self, b: &mut BytesMut) -> std::result::Result<Option<LspEvent>, io::Error> {
        trace!(target: "decode", "bytes len {}", b.len());
        if b.len() == 0 {
            return Ok(None);
        }

        //let z = b.clone();
        if let Ok((input, payload)) = parse_header(&b) {
            b.advance(b.len());
            return Ok(Some(LspEvent::Message(payload)));
        }
        return Ok(None);
        /*Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Can't parse json",
        ))*/
    }
}

impl LspCodec {
    pub fn new() -> Self {
        LspCodec {}
    }
}
