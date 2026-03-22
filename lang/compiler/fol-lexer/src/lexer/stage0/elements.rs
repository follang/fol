use crate::point;
use crate::Con;
use fol_stream::{CharacterProvider, FileStream};
use fol_types::{Win, SLIDER};
use std::fmt;

// Stage 0 owns raw character windowing only.
// It preserves per-character locations and inserts explicit source-boundary
// markers so later token stages never infer cross-file joins from fake text.
type Part<T> = (T, point::Location);

pub const SOURCE_BOUNDARY_CHAR: char = '\u{001D}';

pub struct Elements {
    chars: Box<dyn Iterator<Item = Con<Part<char>>>>,
    win: Win<Con<Part<char>>>,
    _in_count: usize,
}

struct Gen {
    file: FileStream,
    previous_file: Option<String>,
    pending: Option<Con<Part<char>>>,
    emitted_eof: bool,
}

impl Iterator for Gen {
    type Item = Con<Part<char>>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(part) = self.pending.take() {
            return Some(part);
        }

        if let Some((ch, stream_loc)) = self.file.next_char() {
            let current_file = stream_loc.file.clone();
            let loc = point::Location::from_stream_location(&stream_loc);

            if self.previous_file.is_some() && self.previous_file.as_ref() != current_file.as_ref()
            {
                let mut boundary = point::Location::from_stream_location(&stream_loc);
                boundary.adjust(stream_loc.row, 0);
                self.pending = Some(Ok((ch, loc)));
                self.previous_file = current_file;
                return Some(Ok((SOURCE_BOUNDARY_CHAR, boundary)));
            }

            self.previous_file = current_file;
            return Some(Ok((ch, loc)));
        }

        if self.emitted_eof {
            return None;
        }

        // Downstream stages may fold trailing separators around EOF, so the
        // synthetic marker itself stays anchored to an explicit out-of-band point.
        let mut eof_loc = point::Location::default();
        eof_loc.adjust(1, 0);
        self.emitted_eof = true;
        Some(Ok(('\0', eof_loc)))
    }
}

impl Elements {
    fn shift_window(&mut self, incoming: Con<Part<char>>) -> Con<Part<char>> {
        let _ = self.win.0.remove(0);
        self.win.0.push(self.win.1.clone());
        self.win.1 = self.win.2.remove(0);
        self.win.2.push(incoming);
        self.win.1.clone()
    }
    pub fn curr(&self) -> Con<Part<char>> {
        self.win.1.clone()
    }
    ///next vector
    pub fn next_vec(&self) -> Vec<Con<Part<char>>> {
        self.win.2.clone()
    }
    pub fn peek(&self, index: usize) -> Con<Part<char>> {
        let u = if index >= SLIDER { SLIDER - 1 } else { index };
        self.next_vec()[u].clone()
    }
    ///prev vector
    pub fn prev_vec(&self) -> Vec<Con<Part<char>>> {
        let mut rev = self.win.0.clone();
        rev.reverse();
        rev
    }
    pub fn seek(&self, index: usize) -> Con<Part<char>> {
        let u = if index >= SLIDER { SLIDER - 1 } else { index };
        self.prev_vec()[u].clone()
    }

    pub fn init(file: &mut FileStream) -> Self {
        let mut prev = Vec::with_capacity(SLIDER);
        let mut next = Vec::with_capacity(SLIDER);
        let mut chars = Box::new(gen(std::mem::take(file)));
        for _ in 0..SLIDER {
            prev.push(Ok(('\0', point::Location::default())))
        }
        for _ in 0..SLIDER {
            next.push(
                chars
                    .next()
                    .unwrap_or(Ok(('\0', point::Location::default()))),
            )
        }
        Self {
            chars,
            win: (prev, Ok(('\0', point::Location::default())), next),
            _in_count: SLIDER,
        }
    }

    pub fn bump(&mut self) -> Option<Con<Part<char>>> {
        match self.chars.next() {
            Some(v) => Some(self.shift_window(v)),
            None => {
                if self._in_count > 0 {
                    let next = Ok(('\0', point::Location::default()));
                    let current = self.shift_window(next);
                    self._in_count -= 1;
                    Some(current)
                } else {
                    None
                }
            }
        }
    }
}

impl Iterator for Elements {
    type Item = Con<Part<char>>;
    fn next(&mut self) -> Option<Self::Item> {
        self.bump()
    }
}

impl fmt::Display for Elements {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.win.1 {
            Ok((ch, loc)) => write!(f, "{loc} {ch}"),
            Err(_) => write!(f, "ERROR"),
        }
    }
}

pub fn gen(file: FileStream) -> impl Iterator<Item = Con<Part<char>>> {
    Gen {
        file,
        previous_file: None,
        pending: None,
        emitted_eof: false,
    }
}
