#[macro_use]
extern crate nom;

mod parsing;

pub use parsing::parse_from_slice;

use nom::Needed;
use nom::simple_errors::Err;

use std::fmt::{self, Display, Formatter};
use std::fs::File;
use std::io::{self, Read};

// The magic number at the start of each file
const MAGIC_NUMBER: &str = "WEBVTT";

/// A start/end time of a vtt subtitle
#[derive(Debug, PartialEq)]
pub struct Time {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub milliseconds: u16
}

impl Display for Time {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{:02}:{:02}:{:02}.{:03}", self.hours, self.minutes, self.seconds, self.milliseconds)
    }
}

/// A subtitle and associated metadata
#[derive(Debug)]
pub struct Subtitle {
    pub start: Time,
    pub end: Time,
    pub text: String,
    pub note: Option<String>,
    pub positioning: Option<String>
}

impl Display for Subtitle {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}{} --> {}{}{}\n",
            self.note.as_ref().map(|comment| format!("NOTE {}\n\n", comment)).unwrap_or("".into()),
            self.start,
            self.end,
            self.positioning.as_ref().map(|positioning| format!(" {}\n", positioning)).unwrap_or("\n".into()),
            self.text)
    }
}

/// The subtitle file and metadata
#[derive(Debug)]
pub struct Vtt {
    pub subtitles: Vec<Subtitle>,
    pub language: Option<String>,
    pub kind: Option<String>,
    pub style: Option<String>
}

impl Display for Vtt {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}\n{}{}\n{}",
            MAGIC_NUMBER,
            self.kind.as_ref().map(|kind| format!("Kind: {}\n", kind)).unwrap_or("".into()),
            self.language.as_ref().map(|language| format!("Language: {}\n", language)).unwrap_or("".into()),
            self.subtitles.iter().map(|subtitle| format!("{}\n", subtitle)).collect::<String>()
        )
    }
}

/// Errors that can be raised by reaading or parsing a vtt file
#[derive(Debug)]
pub enum Error {
    ParsingError(Err),
    ParsingIncomplete(Needed),
    IO(io::Error)
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::IO(error)
    }
}

/// Parse a vtt file from a file buffer
pub fn parse_from_file(filename: &str) -> Result<Vtt, Error> {
    let mut file = File::open(filename)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    parse_from_slice(&buffer)
}

#[test]
fn simple() {
    let vtt = parse_from_file("tests/simple.vtt").unwrap();
    
    assert_eq!(vtt.kind, Some("captions".into()));
    assert_eq!(vtt.language, Some("en".into()));

    assert_eq!(vtt.subtitles[0].text, "<v Roger Bingham>We are in New York City".to_string());
    assert_eq!(vtt.subtitles[0].start, Time {
        hours: 0,
        minutes: 0,
        seconds: 9,
        milliseconds: 0
    });
    assert_eq!(vtt.subtitles[0].end, Time {
        hours: 0,
        minutes: 0,
        seconds: 11,
        milliseconds: 0
    });

    assert_eq!(vtt.subtitles[9].positioning, Some("align:end size:50%".into()));

    assert_eq!(
        vtt.subtitles[vtt.subtitles.len() - 1].text,
        "<v Roger Bingham>You know I'm so excited my glasses are falling off here.".to_string()
    );
}

#[test]
fn comments() {
    let vtt = parse_from_file("tests/comments.vtt").unwrap();
    assert_eq!(vtt.subtitles[1].note, Some("check next cue".into()));
}

#[test]
fn multiple_lines() {
    let vtt = parse_from_file("tests/multiple_lines.vtt").unwrap();
    assert_eq!(vtt.subtitles[1].text.lines().count(), 2);
}