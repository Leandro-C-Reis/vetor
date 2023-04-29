use raylib::prelude::*;
use std::env;

pub enum ColorStyle {
    BORDER_COLOR_NORMAL,
    BASE_COLOR_NORMAL,
    TEXT_COLOR_NORMAL,
    BORDER_COLOR_FOCUSED,
    BASE_COLOR_FOCUSED,
    TEXT_COLOR_FOCUSED,
    BORDER_COLOR_PRESSED,
    BASE_COLOR_PRESSED,
    TEXT_COLOR_PRESSED,
    BORDER_COLOR_DISABLED,
    BASE_COLOR_DISABLED,
    TEXT_COLOR_DISABLED,
}

impl From<ColorStyle> for Color {
    fn from(value: ColorStyle) -> Self {
        return match value {
            ColorStyle::BORDER_COLOR_NORMAL => Color::get_color(0x2f7486ff),
            ColorStyle::BASE_COLOR_NORMAL => Color::get_color(0x024658ff),
            ColorStyle::TEXT_COLOR_NORMAL => Color::get_color(0x51bfd3ff),
            ColorStyle::BORDER_COLOR_FOCUSED => Color::get_color(0x82cde0),
            ColorStyle::BASE_COLOR_FOCUSED => Color::get_color(0x3299b4ff),
            ColorStyle::TEXT_COLOR_FOCUSED => Color::get_color(0xb6e1ea),
            ColorStyle::BORDER_COLOR_PRESSED => Color::get_color(0xeb7630),
            ColorStyle::BASE_COLOR_PRESSED => Color::get_color(0xffbc51),
            ColorStyle::TEXT_COLOR_PRESSED => Color::get_color(0xd86f36),
            ColorStyle::BORDER_COLOR_DISABLED => Color::get_color(0x134b5aff),
            ColorStyle::BASE_COLOR_DISABLED => Color::get_color(0x02313dff),
            ColorStyle::TEXT_COLOR_DISABLED => Color::get_color(0x17505fff),
            _ => Color::WHITE,
        };
    }
}
