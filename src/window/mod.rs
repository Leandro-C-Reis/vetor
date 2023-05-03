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
        let animation_tab = Rc::new(RefCell::new(Tab::Animation(Animation::new(handle, thread))));

        Window {
            tabs: vec![edit_tab.clone(), animation_tab.clone()],
            selected_tab: animation_tab,
        }
    }

    pub fn update(&mut self, handle: &mut RaylibHandle, thread: &RaylibThread) {
        if handle.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
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
                page.update(handle, thread);
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
        handle.draw_rectangle(
            0,
            0,
            handle.get_screen_width(),
            30,
            Color::get_color(handle.gui_get_style(
                GuiControl::DEFAULT,
                GuiControlProperty::BASE_COLOR_FOCUSED as i32,
            ) as u32),
        );
        for (i, tab) in self.tabs.iter().enumerate() {
            let text = match &*tab.borrow() {
                Tab::Edit { .. } => rstr!("Editar"),
                Tab::Animation { .. } => rstr!("Animar"),
            };
            let icon = cstr!(handle.gui_icon_text(VetorIcons::ICON_CROSS.into(), None));
            let width = 100.0;
            let x = width * i as f32;

            let is_selected = tab.as_ptr() == self.selected_tab.as_ptr();

            handle.gui_set_style(
                GuiControl::TOGGLE,
                GuiControlProperty::TEXT_ALIGNMENT as i32,
                GuiTextAlignment::TEXT_ALIGN_LEFT as i32,
            );
            handle.gui_set_style(
                GuiControl::TOGGLE,
                GuiControlProperty::BORDER_WIDTH as i32,
                1,
            );
            handle.gui_set_style(
                GuiControl::TOGGLE,
                GuiControlProperty::TEXT_PADDING as i32,
                10,
            );
            handle.gui_toggle(rrect(x, 0.0, width, 30.0), Some(text), is_selected);

            handle.gui_set_style(
                GuiControl::BUTTON,
                GuiControlProperty::TEXT_ALIGNMENT as i32,
                GuiTextAlignment::TEXT_ALIGN_CENTER as i32,
            );
            handle.gui_set_style(
                GuiControl::BUTTON,
                GuiControlProperty::TEXT_PADDING as i32,
                0,
            );
            handle.gui_set_style(
                GuiControl::BUTTON,
                GuiControlProperty::BORDER_WIDTH as i32,
                1,
            );
            handle.gui_button(rrect(x + 75.0, 5.0, 20.0, 20.0), Some(icon.as_c_str()));
        }
    }
}
