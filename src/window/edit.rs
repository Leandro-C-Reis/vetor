use super::{util::button::Button, BACKGROUND};
use crate::{
    archives::{self, FileEncoding},
    cstr,
    figure::{edge::*, *},
    icons::VetorIcons,
    maths::*,
};
use native_dialog::FileDialog;
use raylib::{prelude::*, texture::RenderTexture2D};
use std::{ffi::CString, fs, path::Path};

pub struct Edit {
    figure: Figure,
    texture: RenderTexture2D,
    main_position: Vector2,
    main_scroll: Vector2,
    previous_mouse_pos: Vector2,

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
    encoding: FileEncoding,
}

impl Edit {
    pub fn new(handle: &mut RaylibHandle, thread: &RaylibThread, texture: RenderTexture2D) -> Edit {
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

        let screen_center = rvec2(
            handle.get_screen_width() / 2,
            handle.get_screen_height() / 2,
        );

        let texture_center = rvec2(
            -(BACKGROUND.0 as i32 / 2) + (screen_center.x) as i32,
            -(BACKGROUND.1 as i32 / 2) + screen_center.y as i32,
        );

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
            figure: Figure::new(vec![Edge::new(
                -texture_center.sub(screen_center),
                -texture_center.sub(screen_center).sub(rvec2(0, 100)),
                -1,
                1,
            )]),
            texture,
            main_position: rvec2(sidebar_width, start.y),
            previous_mouse_pos: handle.get_mouse_position(),
            main_scroll: texture_center,
            save_figure: Button::new(start.add(rvec2(5, 260))),
            encoding: FileEncoding::ZLIB,
        }
    }

    pub fn update(&mut self, handle: &RaylibHandle) {
        if self.save_figure.activated {
            return;
        }

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

        self.figure
            .update(handle, Vector2::zero().add(self.main_scroll));

        if !self.figure.pressed && handle.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            self.figure.should_update = false;
        }

        if !self.figure.pressed && handle.is_mouse_button_up(MouseButton::MOUSE_BUTTON_LEFT) {
            self.figure.should_update = true;
        }
    }

    pub fn draw(&mut self, handle: &mut RaylibDrawHandle, thread: &RaylibThread) {
        let height = handle.get_screen_height() - self.start.y as i32;
        // ===== Drawing figure =====
        {
            let mut draw_texture = handle.begin_texture_mode(thread, &mut self.texture);
            self.figure.draw(&mut draw_texture);
        }
        // ===== Drawing main Texture Screen =====
        {
            let mut texture_rec = rrect(
                0,
                0,
                self.texture.texture.width,
                self.texture.texture.height,
            );

            let main_rec = rrect(
                self.main_position.x,
                self.main_position.y,
                handle.get_screen_width() as f32 - self.main_position.x,
                handle.get_screen_height() as f32 - self.main_position.y,
            );

            let (scissor_rec, main_scroll) =
                handle.gui_scroll_panel(main_rec, None, texture_rec, self.main_scroll);
            self.main_scroll = main_scroll;

            // Draw main screen texture
            let mut scissor = handle.begin_scissor_mode(
                scissor_rec.x as i32,
                scissor_rec.y as i32,
                scissor_rec.width as i32,
                scissor_rec.height as i32,
            );

            let mouse_pos = scissor.get_mouse_position();

            // Drag background scroll
            if scissor.is_mouse_button_down(MouseButton::MOUSE_BUTTON_MIDDLE)
                && scissor_rec.check_collision_point_rec(mouse_pos)
            {
                let delta = mouse_pos.sub(self.previous_mouse_pos);
                self.main_scroll.x += delta.x;
                self.main_scroll.y += delta.y;

                let max_scroll_x = texture_rec.width - scissor_rec.width;
                let max_scroll_y = texture_rec.height - scissor_rec.height;

                if self.main_scroll.x > 0.0 {
                    self.main_scroll.x = 0.0;
                } else if -self.main_scroll.x >= max_scroll_x {
                    self.main_scroll.x = -max_scroll_x + 1.0;
                }

                if self.main_scroll.y > 0.0 {
                    self.main_scroll.y = 0.0;
                } else if -self.main_scroll.y >= max_scroll_y {
                    self.main_scroll.y = -max_scroll_y + 1.0;
                }
            }

            texture_rec.x -= self.main_scroll.x;
            texture_rec.y += self.main_scroll.y;
            texture_rec.height = -texture_rec.height;
            scissor.draw_texture_rec(
                self.texture.texture(),
                texture_rec,
                Vector2::new(0.0, 0.0),
                Color::RAYWHITE.fade(1.0),
            );

            self.previous_mouse_pos = mouse_pos;
        }
        if self.save_figure.activated {
            self.draw_save_dialog(handle, thread);
        }
        // ===== END Drawing main Texture Screen =====
        // ===== Drawing sidebar edit menu =====
        {
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

            self.save_figure.activated = handle.gui_toggle(
                rrect(
                    self.save_figure.start.x,
                    self.save_figure.start.y,
                    self.sidebar_width - 10.0,
                    30,
                ),
                Some(self.save_figure.text.clone().unwrap().as_c_str()),
                self.save_figure.activated,
            );
        }
        // ===== END Drawing sidebar edit menu =====
    }

    fn draw_save_dialog(&mut self, draw_handle: &mut RaylibDrawHandle, thread: &RaylibThread) {
        let w = draw_handle.get_screen_width();
        let h = draw_handle.get_screen_height();

        // Draw transparent background
        draw_handle.draw_rectangle(
            self.sidebar_width as i32,
            self.start.y as i32,
            w,
            h,
            Color::get_color(draw_handle.gui_get_style(
                GuiControl::DEFAULT,
                GuiControlProperty::BASE_COLOR_NORMAL as i32,
            ) as u32)
            .fade(0.3),
        );

        let dialog_rect = rrect((w / 2) - 150, (h / 2) - 100, 300, 90);

        self.save_figure.activated =
            !draw_handle.gui_window_box(dialog_rect, Some(rstr!("Salvar como:")));

        let save = draw_handle.gui_combo_box(
            rrect(dialog_rect.x + 25.0, dialog_rect.y + 40.0, 120, 30),
            Some(rstr!("raw;zlib;gzip")),
            self.encoding as i32,
        );

        self.encoding = match save {
            0 => FileEncoding::RAW,
            1 => FileEncoding::ZLIB,
            2 => FileEncoding::GZIP,
            _ => FileEncoding::ZLIB,
        };

        if draw_handle.gui_button(
            rrect(dialog_rect.x + 154.0, dialog_rect.y + 40.0, 120, 30),
            Some(rstr!("Salvar")),
        ) {
            self.save("unnamed");
            self.save_figure.activated = false;
        }
    }

    fn save(&mut self, filename: &str) {
        let extension = match self.encoding {
            FileEncoding::RAW => "vfr",
            FileEncoding::GZIP => "vfg",
            FileEncoding::ZLIB => "vfz",
        };

        let path = FileDialog::new()
            .set_filename(&(filename.to_owned() + "." + extension))
            .add_filter("Vetor Figure", &["vfr", "vfg", "vfz"])
            .show_save_single_file()
            .expect("Cannot save file");

        if path.is_none() {
            return;
        }

        archives::export_figure(
            path.unwrap().to_str().unwrap(),
            self.figure.clone(),
            self.encoding,
        );
    }
}
