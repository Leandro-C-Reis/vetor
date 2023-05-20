use super::util::button::Button;
use crate::{
    archives::{self, FileEncoding},
    cstr,
    figure::{edge::*, *},
    icons::VetorIcons,
    maths::*,
};
use raylib::{prelude::*, texture::RenderTexture2D};
use std::{ffi::CString, fs, path::Path};

pub struct Edit {
    figure: Figure,
    framebuffer: RenderTexture2D,
    sidebar_width: f32,
    start: Vector2,
    btn_pressed: bool,
    circle: Button,
    insert: Button,
    hexagon: Button,
    copy: Button,
    toggle_type: Button,
    delete: Button,
    divide: Button,
    circle_fill: Button,
    root: Button,
    format: Button,

    save_figure: Button,
}

impl Edit {
    pub fn new(figure: Figure, texture: RenderTexture2D) -> Edit {
        let sidebar_width = 100.0;
        let start = Vector2::new(0.0, 30.0);

        let circle = Button::dynamic_new(0, 0, start, sidebar_width);
        let insert = Button::dynamic_new(0, 1, start, sidebar_width);
        let hexagon = Button::dynamic_new(1, 0, start, sidebar_width);
        let copy = Button::dynamic_new(1, 1, start, sidebar_width);
        let toggle_type = Button::dynamic_new(2, 0, start, sidebar_width);
        let delete = Button::dynamic_new(2, 1, start, sidebar_width);
        let divide = Button::dynamic_new(3, 0, start, sidebar_width);
        let circle_fill = Button::dynamic_new(3, 1, start, sidebar_width);
        let root = Button::dynamic_new(4, 0, start, sidebar_width);
        let format = Button::dynamic_new(4, 1, start, sidebar_width);

        Edit {
            btn_pressed: false,
            circle,
            insert,
            hexagon,
            copy,
            toggle_type,
            delete,
            divide,
            circle_fill,
            root,
            format,
            start,
            sidebar_width,
            figure,
            framebuffer: texture,
            save_figure: Button::new(start.add(rvec2(5, 260))),
        }
    }

    pub fn update(&mut self, handle: &RaylibHandle) {
        if self.insert.activated || self.circle.activated {
            match self.figure.tmp_edge {
                Some(mut edge) => {
                    edge.end = handle.get_mouse_position();
                    edge.width = edge.start.distance_to(edge.end);
                    edge.fixed_angle = edge.end.angle_to(edge.start);
                    edge.format = if self.circle.activated {
                        EdgeFormat::CIRCLE
                    } else {
                        EdgeFormat::LINE
                    };

                    if handle.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
                        self.figure.insert(edge);
                        self.figure.tmp_edge = None;
                        self.figure.selected = None;
                        self.figure.pressed = false;
                        self.figure.presset_root = false;
                        self.figure.draw_option.point = true;
                        self.figure.should_update = true;

                        self.insert.activated = false;
                        self.circle.activated = false;
                        self.btn_pressed = false;
                    } else {
                        self.figure.tmp_edge = Some(edge);
                    }
                }
                None => {
                    if self.figure.pressed {
                        let index = self.figure.selected.unwrap();
                        let pressed = *self.figure.get(index);

                        let end = if self.figure.presset_root {
                            pressed.start
                        } else {
                            pressed.end
                        };
                        let parent = if self.figure.presset_root {
                            pressed.parent
                        } else {
                            index as isize
                        };

                        self.figure.tmp_edge = Some(Edge::new(end, end, parent, 1));
                        self.figure.draw_option.point = false;
                        self.figure.presset_root = false;
                        self.figure.should_update = false;

                        for index in self.figure.get_children(pressed.parent) {
                            let child = self.figure.get_mut(index);
                            child.pressed_start = false;
                            child.pressed_end = false;
                        }
                    }
                }
            }
        }

        if self.copy.activated {
            match self.figure.tmp_edge {
                Some(mut edge) => {
                    // Edge will move with mouse before insert
                    edge.start = handle.get_mouse_position();
                    edge.end = vector2_rotate(edge.width, edge.fixed_angle).add(edge.start);
                    edge.update_angle();
                    edge.moved_angle = 0.0;
                    edge.pressed_start = false;
                    edge.pressed_end = false;

                    if !self.figure.should_update
                        && handle.is_mouse_button_up(MouseButton::MOUSE_BUTTON_LEFT)
                    {
                        self.figure.should_update = true;
                    }

                    if self.figure.pressed {
                        let index = self.figure.selected.unwrap();

                        edge.parent = index as isize;
                        self.figure.insert(edge);
                        self.figure.tmp_edge = None;
                        self.figure.selected = None;
                        self.figure.pressed = false;
                        self.figure.presset_root = false;
                        self.figure.draw_option.point = true;
                        self.figure.should_update = true;
                        self.figure.clear_edge_and_children(index);

                        self.copy.activated = false;
                        self.btn_pressed = false;
                    } else {
                        self.figure.tmp_edge = Some(edge);
                    }
                }
                None => {
                    if self.figure.pressed {
                        let index = self.figure.selected.unwrap();

                        self.figure.copy_tmp(index);
                        self.figure.selected = None;
                        self.figure.pressed = false;
                        self.figure.presset_root = false;
                        self.figure.should_update = false;
                        self.figure.clear_edge_and_children(index);
                    }
                }
            }
        }

        if self.figure.pressed {
            if self.toggle_type.activated {
                match self.figure.selected {
                    Some(index) => self.figure.toggle_type(index),
                    _ => (),
                }

                self.toggle_type.activated = false;
                self.btn_pressed = false;
            }

            if self.divide.activated {
                match self.figure.selected {
                    Some(index) => self.figure.divide(index),
                    _ => (),
                }

                self.divide.activated = false;
                self.btn_pressed = false;
            }

            if self.delete.activated {
                match self.figure.selected {
                    Some(index) => self.figure.delete(index),
                    _ => (),
                }

                self.delete.activated = false;
                self.btn_pressed = false;
            }

            if self.format.activated {
                match self.figure.selected {
                    Some(index) => {
                        let edge = self.figure.get_mut(index);

                        if edge.format == EdgeFormat::LINE {
                            edge.draw_mode = if edge.draw_mode == EdgeDrawMode::DEFAULT {
                                EdgeDrawMode::LINE_BORDER_FLAT
                            } else {
                                EdgeDrawMode::DEFAULT
                            };
                        }
                    }
                    _ => (),
                }

                self.format.activated = false;
                self.btn_pressed = false;
            }

            if self.circle_fill.activated {
                match self.figure.selected {
                    Some(index) => {
                        let edge = self.figure.get_mut(index);

                        if edge.format == EdgeFormat::CIRCLE {
                            edge.draw_mode = if edge.draw_mode == EdgeDrawMode::DEFAULT {
                                EdgeDrawMode::CIRCLE_CLEAN
                            } else if edge.draw_mode == EdgeDrawMode::CIRCLE_CLEAN {
                                EdgeDrawMode::CIRCLE_FULL
                            } else {
                                EdgeDrawMode::DEFAULT
                            }
                        }
                    }
                    _ => (),
                }

                self.circle_fill.activated = false;
                self.btn_pressed = false;
            }
        }

        self.figure.update(handle, rvec2(0, 0));

        if !self.figure.pressed && handle.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            self.figure.should_update = false;
        }

        if !self.figure.pressed && handle.is_mouse_button_up(MouseButton::MOUSE_BUTTON_LEFT) {
            self.figure.should_update = true;
        }

        if self.save_figure.activated {
            archives::export_figure("unnamed", self.figure.clone(), FileEncoding::ZLIB);
        }
    }

    pub fn draw(&mut self, handle: &mut RaylibDrawHandle, thread: &RaylibThread) {
        let height = handle.get_screen_height() - self.start.y as i32;
        // ===== Drawing figure =====
        {
            let mut draw_texture = handle.begin_texture_mode(thread, &mut self.framebuffer);
            self.figure.draw(&mut draw_texture);
        }
        handle.draw_texture_rec(
            self.framebuffer.texture(),
            rrect(
                0,
                0,
                self.framebuffer.texture.width,
                -self.framebuffer.texture.height,
            ),
            Vector2::new(0.0, 0.0),
            Color::RAYWHITE.fade(1.0),
        );
        // ===== Drawing sidebar edit menu =====

        // Draw sidebar background
        handle.draw_rectangle(
            self.start.x as i32,
            self.start.y as i32,
            self.sidebar_width as i32,
            height,
            Color::get_color(handle.gui_get_style(
                GuiControl::DEFAULT,
                GuiDefaultProperty::BACKGROUND_COLOR as i32,
            ) as u32),
        );

        if self.circle.text.is_none() {
            self.circle.set_icon(handle, VetorIcons::ICON_CIRCLE);
        }
        if self.insert.text.is_none() {
            self.insert.set_icon(handle, VetorIcons::ICON_LINE);
        }
        if self.hexagon.text.is_none() {
            self.hexagon.set_icon(handle, VetorIcons::ICON_HEXAGON);
        }
        if self.copy.text.is_none() {
            self.copy.set_icon(handle, VetorIcons::ICON_COPY);
        }
        if self.toggle_type.text.is_none() {
            self.toggle_type
                .set_icon(handle, VetorIcons::ICON_CIRCLE_LINED);
        }
        if self.delete.text.is_none() {
            self.delete.set_icon(handle, VetorIcons::ICON_CROSS_BOLD);
        }
        if self.divide.text.is_none() {
            self.divide.set_icon(handle, VetorIcons::ICON_DIVIDE);
        }
        if self.circle_fill.text.is_none() {
            self.circle_fill
                .set_icon(handle, VetorIcons::ICON_UNDEFINED);
        }
        if self.root.text.is_none() {
            self.root.set_icon(handle, VetorIcons::ICON_ROOT);
        }
        if self.format.text.is_none() {
            self.format.set_icon(handle, VetorIcons::ICON_VERTEX_FORMAT);
        }
        if self.save_figure.text.is_none() {
            self.save_figure.text = Some(cstr!("Salvar"));
        }

        for btn in [
            &mut self.circle,
            &mut self.insert,
            &mut self.hexagon,
            &mut self.copy,
            &mut self.toggle_type,
            &mut self.delete,
            &mut self.divide,
            &mut self.circle_fill,
            &mut self.root,
            &mut self.format,
        ] {
            handle.gui_set_style(
                GuiControl::TOGGLE,
                GuiControlProperty::TEXT_ALIGNMENT as i32,
                GuiTextAlignment::TEXT_ALIGN_CENTER as i32,
            );

            let btn_press = handle.gui_toggle(
                rrect(btn.start.x, btn.start.y, btn.len, btn.len),
                Some(btn.text.clone().unwrap().as_c_str()),
                btn.activated,
            );

            if btn_press && !self.btn_pressed {
                self.btn_pressed = true;
                btn.activated = true;
            }

            if !btn_press && self.btn_pressed && btn.activated {
                btn.activated = false;
                self.btn_pressed = false;
            }
        }

        self.save_figure.activated = handle.gui_button(
            rrect(
                self.save_figure.start.x,
                self.save_figure.start.y,
                self.sidebar_width - 10.0,
                30,
            ),
            Some(self.save_figure.text.clone().unwrap().as_c_str()),
        );
        // ===== END Drawing sidebar edit menu =====
    }
}
