use *;

use nom::{eol, digit, space, IResult};

use std::str::{FromStr, from_utf8};

// A debug function to print slices as strings
fn _debug(slice: &[u8]) -> IResult<&[u8], ()> {
    println!("{:?}", from_utf8(slice).unwrap());
    IResult::Done(slice, ())
}

// Parse u8s and u16s
named!(u8_digit<u8>,   map_res!(map_res!(digit, from_utf8), u8::from_str));
named!(u16_digit<u16>, map_res!(map_res!(digit, from_utf8), u16::from_str));

named!(line<String>, map_res!(map_res!(take_until!("\n"), from_utf8), String::from_str));
named!(block<String>, map_res!(map_res!(take_until_and_consume!("\n\n"), from_utf8), String::from_str));
named!(style_block<String>, map_res!(map_res!(take_until_and_consume!("##\n"), from_utf8), String::from_str));

// The hour is optional
named!(parse_time<Time>, alt!(
    do_parse!(
        hours:   u8_digit >> char!(':') >>
        minutes: u8_digit >> char!(':') >>
        seconds: u8_digit >> char!('.') >>
        milliseconds: u16_digit >>
        (Time {
            hours, minutes, seconds, milliseconds
        })
    )
    |
    do_parse!(
        minutes: u8_digit >> char!(':') >>
        seconds: u8_digit >> char!('.') >>
        milliseconds: u16_digit >>
        (Time {
            minutes, seconds, milliseconds,
            hours: 0
        })
    )
));


named!(parse_subtitle<Subtitle>, do_parse!(
    // Try and parse an associated note (comment)
    note: opt!(do_parse!(
        tag!("NOTE") >>
        alt!(eol | space) >>
        note: block >>
        (note)
    )) >>
    // Parse the start and end times
    start: parse_time >> tag!(" --> ") >> end: parse_time >>
    // Parse the positioning text (if any)
    positioning: opt!(do_parse!(space >> pos: line >> (pos))) >> eol >>
    // Parse the text block
    text: block >>
    (Subtitle {
        start, end, text, note, positioning
    })
));


named!(parse_vtt<Vtt>, do_parse!(
    tag!(MAGIC_NUMBER) >> eol >>
    kind: opt!(do_parse!(
        tag!("Kind: ") >>
        kind: line >> eol >>
        (kind)
    )) >>
    language: opt!(do_parse!(
        tag!("Language: ") >>
        language: line >> eol >>
        (language)
    )) >>
    style: opt!(do_parse!(
        tag!("Style:") >> eol >>
        style: style_block >>
        (style)
    )) >>
    eol >>
    subtitles: many1!(parse_subtitle) >>
    (Vtt {
        subtitles, language, kind, style
    })
));

pub fn parse_from_slice(slice: &[u8]) -> Result<Vtt, Error> {
    match parse_vtt(slice) {
        IResult::Done(_, vtt) => Ok(vtt),
        IResult::Incomplete(needed) => Err(Error::Incomplete(needed)),
        IResult::Error(err) => Err(Error::ParsingError(err))
    }
}