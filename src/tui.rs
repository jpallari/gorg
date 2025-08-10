use crate::text;
use std::{
    io::{self, Write},
    os::fd::AsFd,
};

use termion::raw::IntoRawMode;
use termion::{
    event::{Event, Key},
    raw::RawTerminal,
};

const QUERY_MAX_CHAR_LEN: u16 = 1000;
const QUERY_MAX_BYTE_LEN: u16 = 4 * QUERY_MAX_CHAR_LEN;
const PROMPT_STRING: &'static str = ">>> ";

pub enum PromptUIEvent {
    Exit,
    PromptUpdated,
    CursorUpdated,
    SelectionUpdated,
    SelectionDone,
}

pub struct PromptUI<W: Write + AsFd> {
    writer: RawTerminal<W>,
    text_input: Vec<char>,
    text_cursor: usize,
    temp_buffer: String,
    selected_item: u16,
    max_items: u16,
    lines_printed: u16,
}

#[derive(Copy, Clone)]
enum TextMovementDirection {
    Left,
    Right,
}

#[derive(Copy, Clone)]
enum TextMovementAmount {
    End,
    Char,
    Word,
}

impl<W: Write + AsFd> Drop for PromptUI<W> {
    fn drop(&mut self) {
        if let Err(err) = self.quit() {
            eprintln!("Failed to quit prompt UI: {}", err);
        }
    }
}

impl<W: Write + AsFd> PromptUI<W> {
    pub fn text_input(&self) -> &[char] {
        &self.text_input
    }

    pub fn selected_item(&self) -> u16 {
        self.selected_item
    }

    pub fn new(writer: W, initial_text_input: &str) -> io::Result<PromptUI<W>> {
        let mut text_input: Vec<char> = Vec::with_capacity(QUERY_MAX_CHAR_LEN.into());
        text_input.extend(initial_text_input.chars().take(QUERY_MAX_CHAR_LEN.into()));
        let cursor_pos = text_input.len();
        let writer = writer.into_raw_mode()?;

        Ok(PromptUI {
            writer,
            text_input,
            lines_printed: 0,
            text_cursor: cursor_pos,
            temp_buffer: String::with_capacity(QUERY_MAX_BYTE_LEN.into()),
            selected_item: 0,
            max_items: 0,
        })
    }

    fn text(&mut self, line: &str) -> io::Result<()> {
        self.writer.write(line.as_bytes())?;
        Ok(())
    }

    fn prompt(&mut self) -> io::Result<()> {
        self.writer.write(PROMPT_STRING.as_bytes())?;
        self.temp_buffer.clear();
        self.temp_buffer.extend(self.text_input.iter());
        self.writer.write(&self.temp_buffer.as_bytes())?;
        self.finish_line()?;
        Ok(())
    }

    fn finish_line(&mut self) -> io::Result<()> {
        self.writer.write("\r\n".as_bytes())?;
        self.lines_printed += 1;
        Ok(())
    }

    fn done(&mut self) -> io::Result<()> {
        let cursor_pos_bytes: u16 = self.text_input[..(self.text_cursor) as usize]
            .iter()
            .map(|c| c.len_utf8() as u16)
            .sum();
        write!(
            self.writer,
            "{}{}",
            termion::cursor::Up(self.lines_printed),
            termion::cursor::Right(PROMPT_STRING.len() as u16 + cursor_pos_bytes)
        )?;
        self.writer.flush()?;
        Ok(())
    }

    pub fn quit(&mut self) -> io::Result<()> {
        self.reset()?;
        self.writer.flush()?;
        Ok(())
    }

    fn reset(&mut self) -> io::Result<()> {
        // Always clear the first line in case it contains input
        write!(self.writer, "\r{}", termion::clear::CurrentLine)?;

        if self.lines_printed > 0 {
            for _ in 0..self.lines_printed {
                write!(
                    self.writer,
                    "\r{}{}",
                    termion::clear::CurrentLine,
                    termion::cursor::Down(1)
                )?;
            }
            write!(self.writer, "{}", termion::cursor::Up(self.lines_printed))?;
            self.lines_printed = 0;
        }
        Ok(())
    }

    pub fn render<'a, T: Iterator<Item = &'a str>>(&mut self, items: T) -> io::Result<()> {
        let (width, height) = termion::terminal_size().unwrap_or((80, 80));

        self.max_items = 0;
        self.reset()?;
        self.prompt()?;

        for (index, item) in items.enumerate().take(height as usize - 2) {
            self.max_items += 1;
            let prefix = if index == self.selected_item as usize {
                "  * "
            } else {
                "    "
            };
            self.text(prefix)?;
            let item_len = item.len().min((width as usize).max(10) - prefix.len());
            self.text(&item[..item_len])?;
            self.finish_line()?;
        }

        self.done()?;
        Ok(())
    }

    pub fn handle_event(&mut self, event: Event) -> Option<PromptUIEvent> {
        match event {
            Event::Key(Key::Char('\n')) => Some(PromptUIEvent::SelectionDone),
            Event::Key(Key::Backspace) => {
                if self.delete_char() {
                    self.selected_item = 0;
                    Some(PromptUIEvent::PromptUpdated)
                } else {
                    None
                }
            }
            Event::Key(Key::Ctrl('h'))
            | Event::Key(Key::Alt('\u{7f}'))
            | Event::Key(Key::Ctrl('w')) => {
                if self.delete_word() {
                    self.selected_item = 0;
                    Some(PromptUIEvent::PromptUpdated)
                } else {
                    None
                }
            }
            Event::Key(Key::Up) | Event::Key(Key::Ctrl('p')) => {
                if self.selected_item > 0 {
                    self.selected_item -= 1;
                    Some(PromptUIEvent::SelectionUpdated)
                } else {
                    None
                }
            }
            Event::Key(Key::Down) | Event::Key(Key::Ctrl('n')) => {
                if self.selected_item + 1 < self.max_items {
                    self.selected_item += 1;
                    Some(PromptUIEvent::SelectionUpdated)
                } else {
                    None
                }
            }
            Event::Key(Key::Left) | Event::Key(Key::Ctrl('b')) => {
                self.move_cursor(TextMovementDirection::Left, TextMovementAmount::Char);
                Some(PromptUIEvent::CursorUpdated)
            }
            Event::Key(Key::Right) | Event::Key(Key::Ctrl('f')) => {
                self.move_cursor(TextMovementDirection::Right, TextMovementAmount::Char);
                Some(PromptUIEvent::CursorUpdated)
            }
            Event::Key(Key::CtrlLeft) | Event::Key(Key::AltLeft) | Event::Key(Key::Alt('b')) => {
                self.move_cursor(TextMovementDirection::Left, TextMovementAmount::Word);
                Some(PromptUIEvent::CursorUpdated)
            }
            Event::Key(Key::CtrlRight) | Event::Key(Key::AltRight) | Event::Key(Key::Alt('f')) => {
                self.move_cursor(TextMovementDirection::Right, TextMovementAmount::Word);
                Some(PromptUIEvent::CursorUpdated)
            }
            Event::Key(Key::Home) | Event::Key(Key::Ctrl('a')) => {
                self.move_cursor(TextMovementDirection::Left, TextMovementAmount::End);
                Some(PromptUIEvent::CursorUpdated)
            }
            Event::Key(Key::End) | Event::Key(Key::Ctrl('e')) => {
                self.move_cursor(TextMovementDirection::Right, TextMovementAmount::End);
                Some(PromptUIEvent::CursorUpdated)
            }
            Event::Key(Key::Ctrl('c')) | Event::Key(Key::Ctrl('d')) => Some(PromptUIEvent::Exit),
            Event::Key(Key::Char(ch)) => {
                self.insert_char(ch);
                self.selected_item = 0;
                Some(PromptUIEvent::PromptUpdated)
            }
            _ => None,
        }
    }

    fn delete_char(&mut self) -> bool {
        if self.text_input.is_empty() || self.text_cursor == 0 {
            return false;
        }
        self.text_cursor -= 1;
        self.text_input.remove(self.text_cursor);
        true
    }

    fn delete_word(&mut self) -> bool {
        if self.text_input.is_empty() {
            return false;
        }

        let next_cursor_pos = move_cursor(
            &self.text_input,
            self.text_cursor,
            TextMovementDirection::Left,
            TextMovementAmount::Word,
        );
        if self.text_cursor == next_cursor_pos {
            return false;
        }

        self.text_input.drain(next_cursor_pos..self.text_cursor);
        self.text_cursor = next_cursor_pos;
        true
    }

    fn insert_char(&mut self, ch: char) {
        if self.text_input.len() + 1 > self.text_input.capacity() {
            // Max capacity reached for prompt
            return;
        }

        if self.text_cursor == self.text_input.len() {
            self.text_input.push(ch);
        } else {
            self.text_input.insert(self.text_cursor, ch);
        }

        self.text_cursor += 1;
    }

    fn move_cursor(&mut self, direction: TextMovementDirection, amount: TextMovementAmount) {
        self.text_cursor = move_cursor(&self.text_input, self.text_cursor, direction, amount);
    }
}

fn move_cursor(
    text: &[char],
    cursor: usize,
    direction: TextMovementDirection,
    amount: TextMovementAmount,
) -> usize {
    use TextMovementAmount::*;
    use TextMovementDirection::*;

    if text.is_empty() {
        return 0;
    }

    match (direction, amount) {
        (Left, End) => 0,
        (Right, End) => text.len(),
        (Left, Char) => {
            if cursor > 0 {
                cursor - 1
            } else {
                0
            }
        }
        (Right, Char) => {
            if cursor < text.len() {
                cursor + 1
            } else {
                cursor
            }
        }
        (Left, Word) => {
            if cursor > 0 {
                let mut iter = text[..cursor].iter().rev().enumerate();
                let (_, first_char) = iter.next().expect("First char must be found");
                let res = if text::is_punctuation(*first_char) {
                    iter.skip_while(|(_, ch)| text::is_punctuation(**ch))
                        .find(|(_, ch)| text::is_punctuation(**ch))
                } else {
                    iter.find(|(_, ch)| text::is_punctuation(**ch))
                };
                match res {
                    Some((offset, _)) => cursor - offset,
                    None => 0,
                }
            } else {
                cursor
            }
        }
        (Right, Word) => {
            if cursor < text.len() {
                let mut iter = text[cursor..].iter().enumerate();
                let (_, first_char) = iter.next().expect("First char must be found");
                let res = if text::is_punctuation(*first_char) {
                    iter.skip_while(|(_, ch)| text::is_punctuation(**ch))
                        .find(|(_, ch)| text::is_punctuation(**ch))
                } else {
                    iter.find(|(_, ch)| text::is_punctuation(**ch))
                };
                match res {
                    Some((offset, _)) => cursor + offset,
                    None => text.len(),
                }
            } else {
                cursor
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_cursor_word_right_from_punctuation() {
        let dir = TextMovementDirection::Right;
        let amount = TextMovementAmount::Word;
        let s: Vec<char> = Vec::from_iter("ground control to major tom".chars());
        assert_eq!(14, move_cursor(&s, 6, dir, amount));
        assert_eq!(17, move_cursor(&s, 14, dir, amount));
        assert_eq!(23, move_cursor(&s, 17, dir, amount));
        assert_eq!(27, move_cursor(&s, 23, dir, amount));
    }

    #[test]
    fn move_cursor_word_right_from_char() {
        let dir = TextMovementDirection::Right;
        let amount = TextMovementAmount::Word;
        let s: Vec<char> = Vec::from_iter("ground control to major tom".chars());
        assert_eq!(6, move_cursor(&s, 0, dir, amount));
        assert_eq!(6, move_cursor(&s, 3, dir, amount));
        assert_eq!(14, move_cursor(&s, 7, dir, amount));
        assert_eq!(14, move_cursor(&s, 13, dir, amount));
        assert_eq!(23, move_cursor(&s, 18, dir, amount));
        assert_eq!(23, move_cursor(&s, 20, dir, amount));
        assert_eq!(27, move_cursor(&s, 24, dir, amount));
        assert_eq!(27, move_cursor(&s, 25, dir, amount));
        assert_eq!(27, move_cursor(&s, 27, dir, amount));
    }

    #[test]
    fn move_cursor_word_left_from_punctuation() {
        let dir = TextMovementDirection::Left;
        let amount = TextMovementAmount::Word;
        let s: Vec<char> = Vec::from_iter("ground control to major tom".chars());
        assert_eq!(0, move_cursor(&s, 6, dir, amount));
        assert_eq!(7, move_cursor(&s, 14, dir, amount));
        assert_eq!(15, move_cursor(&s, 17, dir, amount));
        assert_eq!(18, move_cursor(&s, 23, dir, amount));
        assert_eq!(24, move_cursor(&s, 27, dir, amount));
    }

    #[test]
    fn move_cursor_word_left_from_char() {
        let dir = TextMovementDirection::Left;
        let amount = TextMovementAmount::Word;
        let s: Vec<char> = Vec::from_iter("ground control to major tom".chars());
        assert_eq!(0, move_cursor(&s, 5, dir, amount));
        assert_eq!(0, move_cursor(&s, 3, dir, amount));
        assert_eq!(0, move_cursor(&s, 7, dir, amount));
        assert_eq!(7, move_cursor(&s, 8, dir, amount));
        assert_eq!(7, move_cursor(&s, 10, dir, amount));
        assert_eq!(7, move_cursor(&s, 13, dir, amount));
        assert_eq!(24, move_cursor(&s, 25, dir, amount));
        assert_eq!(24, move_cursor(&s, 26, dir, amount));
    }
}
