use serde::Deserialize;
use std::io::Write;
use termion::{color, cursor};

use crate::values::SCREEN;

#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(tag = "type")]
pub enum RenderRequest {
    #[serde(rename = "set char")]
    /// set a character at a specific location
    SetChar { x: u32, y: u32, c: char },
    #[serde(rename = "set colouredchar")]
    /// set a character at a specific locaton with fg and bg colours
    SetCharColoured {
        x: u32,
        y: u32,
        c: char,
        fg: Colour,
        bg: Colour,
    },
    #[serde(rename = "flush")]
    /// flush all changes
    Flush,
    #[serde(rename = "set cursorstyle")]
    /// change cursor style
    SetCursorStyle { style: CursorStyle },
    #[serde(rename = "hide cursor")]
    /// hide cursor
    HideCursor,
    #[serde(rename = "show cursor")]
    /// show cursor
    ShowCursor,

    #[serde(rename = "clear all")]
    /// clear entire screen
    ClearAll,

    #[serde(rename = "render multiple")]
    /// complete multiple render tasks at the same time - e.g. stacking changes
    RenderMultiple { tasks: Vec<Self> },
}

impl RenderRequest {
    pub fn draw(&self, mut flush: bool) {
        match self {
            Self::SetChar { x, y, c } => {
                write!(
                    unsafe { SCREEN.get().unwrap().lock().unwrap() },
                    "{}{c}",
                    termion::cursor::Goto(*x as u16 + 1, *y as u16 + 1)
                )
                .unwrap();
            }
            Self::SetCharColoured { x, y, c, fg, bg } => {
                write!(
                    unsafe { SCREEN.get().unwrap().lock().unwrap() },
                    "{}{}{}{c}{}{}",
                    color::Fg(*fg),
                    color::Bg(*bg),
                    termion::cursor::Goto(*x as u16 + 1, *y as u16 + 1),
                    color::Fg(termion::color::Reset),
                    color::Bg(termion::color::Reset),
                )
                .unwrap();
            }
            Self::Flush => flush = true,
            Self::SetCursorStyle { style } => match style {
                CursorStyle::BlinkingBar => write!(
                    unsafe { SCREEN.get().unwrap().lock().unwrap() },
                    "{}",
                    cursor::BlinkingBar
                ),
                CursorStyle::BlinkingBlock => write!(
                    unsafe { SCREEN.get().unwrap().lock().unwrap() },
                    "{}",
                    cursor::BlinkingBlock
                ),
                CursorStyle::BlinkingUnderline => write!(
                    unsafe { SCREEN.get().unwrap().lock().unwrap() },
                    "{}",
                    cursor::BlinkingUnderline
                ),
                CursorStyle::SteadyBar => {
                    write!(
                        unsafe { SCREEN.get().unwrap().lock().unwrap() },
                        "{}",
                        cursor::SteadyBar
                    )
                }
                CursorStyle::SteadyBlock => write!(
                    unsafe { SCREEN.get().unwrap().lock().unwrap() },
                    "{}",
                    cursor::SteadyBlock
                ),
                CursorStyle::SteadyUnderline => write!(
                    unsafe { SCREEN.get().unwrap().lock().unwrap() },
                    "{}",
                    cursor::SteadyUnderline
                ),
            }
            .unwrap(),
            Self::HideCursor => write!(
                unsafe { SCREEN.get().unwrap().lock().unwrap() },
                "{}",
                cursor::Hide
            )
            .unwrap(),
            Self::ShowCursor => write!(
                unsafe { SCREEN.get().unwrap().lock().unwrap() },
                "{}",
                cursor::Show
            )
            .unwrap(),
            Self::RenderMultiple { tasks } => {
                for task in tasks.clone() {
                    task.draw(false);
                }
            }
            Self::ClearAll => write!(
                unsafe { SCREEN.get().unwrap().lock().unwrap() },
                "{}",
                termion::clear::All
            )
            .unwrap(),
        }

        if flush {
            unsafe { SCREEN.get().unwrap().lock().unwrap() }
                .flush()
                .unwrap();
        }
    }
}

#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum CursorStyle {
    #[serde(rename = "blinking bar")]
    BlinkingBar,
    #[serde(rename = "blinking block")]
    BlinkingBlock,
    #[serde(rename = "blinking underline")]
    BlinkingUnderline,
    #[serde(rename = "steady bar")]
    SteadyBar,
    #[serde(rename = "steady block")]
    SteadyBlock,
    #[serde(rename = "steady underline")]
    SteadyUnderline,
}

#[derive(Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(tag = "type")]
pub enum Colour {
    #[serde(rename = "black")]
    Black,
    #[serde(rename = "blue")]
    Blue,
    #[serde(rename = "cyan")]
    Cyan,
    #[serde(rename = "green")]
    Green,
    #[serde(rename = "magenta")]
    Magenta,
    #[serde(rename = "red")]
    Red,
    #[serde(rename = "white")]
    White,
    #[serde(rename = "yellow")]
    Yellow,

    #[serde(rename = "lightblack")]
    LightBlack,
    #[serde(rename = "lightblue")]
    LightBlue,
    #[serde(rename = "lightcyan")]
    LightCyan,
    #[serde(rename = "lightgreen")]
    LightGreen,
    #[serde(rename = "lightmagenta")]
    LightMagenta,
    #[serde(rename = "lightred")]
    LightRed,
    #[serde(rename = "lightwhite")]
    LightWhite,
    #[serde(rename = "lightyellow")]
    LightYellow,

    #[serde(rename = "reset")]
    Reset,
    #[serde(rename = "ansi")]
    Ansi { value: u8 },
    #[serde(rename = "rgb")]
    Rgb { red: u8, green: u8, blue: u8 },
}

impl termion::color::Color for Colour {
    fn write_fg(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Black => termion::color::Black.write_fg(f),
            Self::Blue => termion::color::Blue.write_fg(f),
            Self::Cyan => termion::color::Cyan.write_fg(f),
            Self::Green => termion::color::Green.write_fg(f),
            Self::Red => termion::color::Red.write_fg(f),
            Self::Magenta => termion::color::Magenta.write_fg(f),
            Self::White => termion::color::White.write_fg(f),
            Self::Yellow => termion::color::Yellow.write_fg(f),

            Self::LightBlack => termion::color::LightBlack.write_fg(f),
            Self::LightBlue => termion::color::LightBlue.write_fg(f),
            Self::LightCyan => termion::color::LightCyan.write_fg(f),
            Self::LightGreen => termion::color::LightGreen.write_fg(f),
            Self::LightMagenta => termion::color::LightMagenta.write_fg(f),
            Self::LightRed => termion::color::LightRed.write_fg(f),
            Self::LightWhite => termion::color::LightWhite.write_fg(f),
            Self::LightYellow => termion::color::LightYellow.write_fg(f),

            Self::Reset => termion::color::Reset.write_fg(f),
            Self::Rgb { red, green, blue } => termion::color::Rgb(*red, *green, *blue).write_fg(f),
            Self::Ansi { value } => termion::color::AnsiValue(*value).write_fg(f),
        }
    }

    fn write_bg(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Black => termion::color::Black.write_bg(f),
            Self::Blue => termion::color::Blue.write_bg(f),
            Self::Cyan => termion::color::Cyan.write_bg(f),
            Self::Green => termion::color::Green.write_bg(f),
            Self::Red => termion::color::Red.write_bg(f),
            Self::Magenta => termion::color::Magenta.write_bg(f),
            Self::White => termion::color::White.write_bg(f),
            Self::Yellow => termion::color::Yellow.write_bg(f),

            Self::LightBlack => termion::color::LightBlack.write_bg(f),
            Self::LightBlue => termion::color::LightBlue.write_bg(f),
            Self::LightCyan => termion::color::LightCyan.write_bg(f),
            Self::LightGreen => termion::color::LightGreen.write_bg(f),
            Self::LightMagenta => termion::color::LightMagenta.write_bg(f),
            Self::LightRed => termion::color::LightRed.write_bg(f),
            Self::LightWhite => termion::color::LightWhite.write_bg(f),
            Self::LightYellow => termion::color::LightYellow.write_bg(f),

            Self::Reset => termion::color::Reset.write_bg(f),
            Self::Rgb { red, green, blue } => termion::color::Rgb(*red, *green, *blue).write_bg(f),
            Self::Ansi { value } => termion::color::AnsiValue(*value).write_bg(f),
        }
    }
}
