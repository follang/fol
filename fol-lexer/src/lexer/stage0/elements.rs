use crate::point;
use fol_stream::{CharacterProvider, FileStream};
use fol_types::{Con, Vod, Win, SLIDER};
use std::fmt;

// Stage 0 owns raw character windowing only.
// It preserves per-character locations and inserts the minimum synthetic
// boundary markers needed so later token stages never infer cross-file joins.
type Part<T> = (T, point::Location);

pub struct Elements {
    chars: Box<dyn Iterator<Item = Con<Part<char>>>>,
    win: Win<Con<Part<char>>>,
    _in_count: usize,
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
        let u = if index > SLIDER { 0 } else { index };
        self.next_vec()[u].clone()
    }
    ///prev vector
    pub fn prev_vec(&self) -> Vec<Con<Part<char>>> {
        let mut rev = self.win.0.clone();
        rev.reverse();
        rev
    }
    pub fn seek(&self, index: usize) -> Con<Part<char>> {
        let u = if index > SLIDER { 0 } else { index };
        self.prev_vec()[u].clone()
    }

    pub fn init(file: &mut FileStream) -> Self {
        let mut prev = Vec::with_capacity(SLIDER);
        let mut next = Vec::with_capacity(SLIDER);
        let mut chars = Box::new(gen(file));
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
    pub fn debug(&self) -> Vod {
        println!("{}\t{}", self.curr()?.1, self.curr()?.0);
        Ok(())
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
        if self.win.1.clone().is_ok() {
            write!(
                f,
                "{} {}",
                self.win.1.clone().unwrap().1,
                self.win.1.clone().unwrap().0
            )
        } else {
            write!(f, "ERROR")
        }
    }
}

pub fn gen(file: &mut FileStream) -> impl Iterator<Item = Con<Part<char>>> {
    // Collect all characters from the file stream
    let mut chars = Vec::new();
    let mut previous_file: Option<String> = None;
    let mut previous_char: Option<char> = None;
    while let Some((ch, stream_loc)) = file.next_char() {
        if previous_file.as_ref() != stream_loc.file.as_ref()
            && previous_char.is_some_and(|prev| !prev.is_whitespace())
            && !ch.is_whitespace()
        {
            let mut boundary = point::Location::from_stream_location(&stream_loc);
            boundary.adjust(1, 0);
            chars.push(Ok(('\n', boundary)));
        }

        let loc = point::Location::from_stream_location(&stream_loc);
        chars.push(Ok((ch, loc)));
        previous_file = stream_loc.file.clone();
        previous_char = Some(ch);
    }

    // Downstream stages may fold trailing separators around EOF, so the
    // synthetic marker itself stays anchored to an explicit out-of-band point.
    let mut eof_loc = point::Location::default();
    eof_loc.adjust(1, 0);
    chars.push(Ok(('\0', eof_loc)));

    chars.into_iter()
}
