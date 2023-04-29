use crate::icons::VetorIcons;
use raylib::prelude::*;
use std::ffi::CString;

pub struct Button {
    pub activated: bool,
    pub icon: Option<CString>,
    pub start: Vector2,
    pub len: f32,
}

impl Button {
    pub fn new(start: Vector2) -> Button {
        Button {
            activated: false,
            len: 30.0,
            icon: None,
            start,
        }
    }

    pub fn set_icon(&mut self, handle: &mut RaylibDrawHandle, icon: VetorIcons) {
        self.icon = Some(CString::new(handle.gui_icon_text(icon.into(), None)).unwrap());
    }

    pub fn dynamic_new(row: i32, col: i32, start: Vector2, width: f32) -> Button {
        let gap = width * 0.05;
        let icon_width = (width / 2.0) - gap;

        let factorx = (col as f32 * icon_width) + if col > 0 { gap * 1.5 } else { gap / 2.0 };
        let factory = (row as f32 * icon_width) + gap * (row + 1) as f32;

        Button {
            activated: false,
            len: icon_width,
            icon: None,
            start: Vector2::new(start.x + factorx, start.y + factory),
        }
    }
}
