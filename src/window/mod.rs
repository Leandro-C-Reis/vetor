pub mod animation;
pub mod edit;
pub mod tab;
pub mod util;

use self::animation::*;
use self::edit::*;
use self::tab::Tab;
use self::util::button::*;
use crate::imports;
use crate::{cstr, maths::*};
use crate::{
    figure::{
        edge::{Edge, EdgeDrawMode, EdgeFormat},
        Figure,
    },
    icons::VetorIcons,
    log,
    maths::vector2_rotate,
};
use raylib::{ffi::CheckCollisionPointRec, prelude::*};
use std::{cell::RefCell, ffi::CString, fs, path::Path, rc::Rc};

pub struct Window {
    pub tabs: Vec<Rc<RefCell<Tab>>>,
    pub selected_tab: Rc<RefCell<Tab>>,
}

impl Window {
    pub fn new(handle: &mut RaylibHandle, thread: &RaylibThread) -> Window {
        let mut figure = imports::bin::import_from_raw(
            "men.vec",
            Vector2::new(
                (handle.get_screen_width() / 2) as f32,
                (handle.get_screen_height() / 2) as f32,
            ),
        );

        let texture = handle
            .load_render_texture(
                &thread,
                handle.get_screen_width() as u32,
                handle.get_screen_height() as u32,
            )
            .ok()
            .unwrap();

        let edit_tab = Rc::new(RefCell::new(Tab::Edit(Edit::new(figure.clone(), texture))));
        let animation_tab = Rc::new(RefCell::new(Tab::Animation(Animation::new(
            handle
                .load_render_texture(
                    &thread,
                    handle.get_screen_width() as u32,
                    handle.get_screen_height() as u32,
                )
                .ok()
                .unwrap(),
        ))));

        match &mut *animation_tab.borrow_mut() {
            Tab::Animation(page) => {
                page.push_figure(figure.clone());
                page.push_figure(figure.clone());
            }
            _ => (),
        }

        Window {
            tabs: vec![edit_tab.clone(), animation_tab.clone()],
            selected_tab: edit_tab,
        }
    }

    pub fn update(&mut self, handle: &RaylibHandle) {
        if handle.is_mouse_button_pressed(MouseButton::MOUSE_LEFT_BUTTON) {
            let mouse_pos = handle.get_mouse_position();
            let collision = unsafe {
                CheckCollisionPointRec(
                    mouse_pos.into(),
                    rrect(0, 0, handle.get_screen_width(), 30).into(),
                )
            };

            if collision {
                let width = 100;
                let height = 30;

                for (i, tab) in self.tabs.iter().enumerate() {
                    let tab_collision = unsafe {
                        CheckCollisionPointRec(
                            mouse_pos.into(),
                            rrect(i as i32 * width, 0, width, height).into(),
                        )
                    };

                    if tab_collision {
                        self.selected_tab = tab.clone();
                        break;
                    }
                }
            }
        }

        let mut tab = (*self.selected_tab).borrow_mut();

        match &mut *tab {
            Tab::Edit(page) => {
                page.update(handle);
            }
            Tab::Animation(page) => {
                page.update(handle);
            }
        }
    }

    pub fn draw(&self, handle: &mut RaylibDrawHandle, thread: &RaylibThread) {
        // Draw current selected tab
        match &mut *(*self.selected_tab).borrow_mut() {
            Tab::Edit(page) => {
                page.draw(handle, thread);
            }
            Tab::Animation(page) => {
                page.draw(handle, thread);
            }
        }

        // Draw tab menu
        handle.draw_rectangle(0, 0, handle.get_screen_width(), 30, Color::LIGHTGRAY);
        for (i, tab) in self.tabs.iter().enumerate() {
            // Remove border and Align label text to center
            handle.gui_set_style(
                GuiControl::BUTTON,
                GuiControlProperty::BORDER_WIDTH as i32,
                0,
            );
            handle.gui_set_style(
                GuiControl::LABEL,
                GuiControlProperty::TEXT_ALIGNMENT as i32,
                GuiTextAlignment::GUI_TEXT_ALIGN_CENTER as i32,
            );

            let text = match &*tab.borrow() {
                Tab::Edit { .. } => "Editar",
                Tab::Animation { .. } => "Animar",
            };
            let icon =
                CString::new(handle.gui_icon_text(VetorIcons::ICON_CROSS.into(), None)).unwrap();
            let width = 100.0;
            let x = width * i as f32;

            // Check if tab is the current selected.
            if tab.as_ptr() == self.selected_tab.as_ptr() {
                // Color = #5bb2d9
                handle.draw_rectangle(x as i32, 0, width as i32, 30, Color::new(91, 178, 217, 255));
            }

            handle.draw_rectangle_lines(x as i32, 0, width as i32, 30, Color::GRAY);
            handle.gui_label_button(
                rrect(x, 0.0, width, 30.0),
                Some(&CString::new(text).unwrap()),
            );
            handle.gui_button(rrect(x + 75.0, 5.0, 20.0, 20.0), Some(&icon));
        }
    }
}
