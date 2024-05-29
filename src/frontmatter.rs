/// This code was borrowed from @fasterthanlime.
use serde::{Serialize, Deserialize};

#[derive(Eq, PartialEq, Deserialize, Default, Debug, Serialize, Clone)]
pub struct Frontmatter {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub date: Option<String>,
    pub version: Option<String>,
    pub customer: Option<String>,
    pub policy: Option<String>,
    pub document: Option<String>,
    pub author: Option<Vec<String>>,
    pub include_toc: Option<bool>,
    pub keywords: Option<Vec<String>>,
}

enum State {
    SearchForStart,
    ReadingMarker { count: usize, end: bool },
    ReadingFrontMatter { buf: String, line_start: bool },
    SkipNewline { end: bool },
}

impl Frontmatter {
    pub fn parse(input: &str) -> Result<(Frontmatter, usize), Box<dyn std::error::Error>> {
        let mut state = State::SearchForStart;

        let mut payload = None;
        let offset;

        let mut chars = input.char_indices();
        'parse: loop {
            let (idx, ch) = match chars.next() {
                Some(x) => x,
                None => return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "EOF"))?,
            };
            match &mut state {
                State::SearchForStart => match ch {
                    '-' => {
                        state = State::ReadingMarker {
                            count: 1,
                            end: false,
                        };
                    }
                    '\n' | '\t' | ' ' => {
                        // ignore whitespace
                    }
                    _ => {
                        Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Start of frontmatter not found"))?;
                    }
                },
                State::ReadingMarker { count, end } => match ch {
                    '-' => {
                        *count += 1;
                        if *count == 3 {
                            state = State::SkipNewline { end: *end };
                        }
                    }
                    _ => {
                        Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Malformed frontmatter marker"))?;
                    }
                },
                State::SkipNewline { end } => match ch {
                    '\n' => {
                        if *end {
                            offset = idx + 1;
                            break 'parse;
                        } else {
                            state = State::ReadingFrontMatter {
                                buf: String::new(),
                                line_start: true,
                            };
                        }
                    }
                    x => {
                        let err = format!("Expected newline, got {:?}", x);
                        Err(std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
                    }
                },
                State::ReadingFrontMatter { buf, line_start } => match ch {
                    '-' if *line_start => {
                        let mut state_temp = State::ReadingMarker {
                            count: 1,
                            end: true,
                        };
                        std::mem::swap(&mut state, &mut state_temp);
                        if let State::ReadingFrontMatter { buf, .. } = state_temp {
                            payload = Some(buf);
                        } else {
                            unreachable!();
                        }
                    }
                    ch => {
                        buf.push(ch);
                        *line_start = ch == '\n';
                    }
                },
            }
        }

        // unwrap justification: option set in state machine, Rust can't statically analyze it
        let payload = payload.unwrap();

        let fm: Self = serde_yaml::from_str(&payload)?;

        Ok((fm, offset))
    }
}