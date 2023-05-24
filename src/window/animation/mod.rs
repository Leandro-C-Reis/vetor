pub mod frame;

use self::frame::*;
use super::{util::button::Button, BACKGROUND};
use crate::{
    archives::{self, FileEncoding},
    cstr,
    figure::Figure,
    icons::VetorIcons,
    maths::*,
};
use flate2::{
    read::{GzDecoder, ZlibDecoder},
    write::ZlibEncoder,
    Compression,
};
use native_dialog::FileDialog;
use raylib::{
    ffi::{CheckCollisionPointRec, GetMonitorHeight, GetMonitorWidth, ImageBlurGaussian, WaitTime},
    prelude::*,
};
use std::{
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
    cmp::Ordering,
    collections::HashMap,
    ffi::CString,
    fmt::Display,
    fs::{self, File},
    io::{Read, Write},
    path::Path,
    process::{Command, Stdio},
    rc::Rc,
};

#[derive(Debug, Clone, Copy)]
enum ExportFormat {
    MP4 = 0,
    GIF = 1,
}

struct Caroussel {
    value: f32,
    x: i32,
    width: i32,
    display_gap: i32,
    display_width: i32,
    display_height: i32,
    scrollbar_height: i32,
}

pub struct Animation {
    figures: Vec<Rc<RefCell<Figure>>>,
    frames: Vec<Frame>,
    selected_frame: usize,
    // Main
    previous_mouse_pos: Vector2,
    main_texture: Rc<RefCell<RenderTexture2D>>,
    main_scroll: Vector2,
    main_position: Vector2,
    video_camera: Rectangle,
    // Frame scroll
    frame_caroussel: Caroussel,
    // Sidebar
    sidebar: Rectangle,
    save_frame: Button,
    save_animation: Button,
    add_figure: Button,
    // Play Animation
    play: Button,
    previous_time: f64,
    framerate: f32,
    // Export Dialog
    export_format: ExportFormat,
    save_format: FileEncoding,
}

impl Animation {
    pub fn new(handle: &mut RaylibHandle, thread: &RaylibThread) -> Animation {
        let sidebar = rrect(0, 30, 100, handle.get_screen_height() - 30);
        let frame_position = rvec2(sidebar.width, 30);

        let mut first_frame = Frame::new(handle, thread, BACKGROUND.0, BACKGROUND.1);
        first_frame.is_selected = true;

        let mut figure =
            archives::import_figure("./src/assets/figures/men.vfr", archives::FileEncoding::RAW);

        figure.center_to(rvec2(
            first_frame.texture.try_borrow().ok().unwrap().width() / 2,
            first_frame.texture.try_borrow().ok().unwrap().height() / 2,
        ));

        let video_width = 1080;
        let video_height = 720;
        let start = rvec2(sidebar.x, sidebar.y).add(rvec2(15, 20));
        let video_camera = rrect(
            (first_frame
                .texture
                .clone()
                .try_borrow()
                .ok()
                .unwrap()
                .width()
                / 2)
                - video_width / 2,
            (first_frame
                .texture
                .clone()
                .try_borrow()
                .ok()
                .unwrap()
                .height()
                / 2)
                - video_height / 2,
            video_width,
            video_height,
        );

        let main_center = rvec2(
            (handle.get_screen_width() / 2) + frame_position.x as i32,
            (handle.get_screen_height() / 2) + frame_position.y as i32,
        );

        let texture_center = rvec2(
            -(BACKGROUND.0 as i32 / 2) + (main_center.x) as i32,
            -(BACKGROUND.1 as i32 / 2) + main_center.y as i32,
        );

        let mut animation = Animation {
            export_format: ExportFormat::GIF,
            save_format: FileEncoding::RAW,
            selected_frame: 0,
            frame_caroussel: Caroussel {
                value: 0.0,
                x: (sidebar.x + sidebar.width) as i32,
                width: handle.get_screen_width() - sidebar.width as i32,
                display_gap: 10,
                display_width: 150,
                display_height: 100,
                scrollbar_height: 15,
            },
            previous_time: 0.0,
            framerate: 5.0,
            add_figure: Button::new(rvec2(sidebar.x, sidebar.y).add(rvec2(10, 160))),
            save_frame: Button::new(rvec2(sidebar.x, sidebar.y).add(rvec2(10, 200))),
            save_animation: Button::dynamic_new(0, 0, start, sidebar.width - 30.0),
            play: Button::dynamic_new(0, 1, start, sidebar.width - 30.0),
            main_texture: first_frame.texture.clone(),
            main_scroll: texture_center,
            video_camera,
            previous_mouse_pos: Vector2::zero(),
            figures: vec![],
            frames: vec![first_frame],
            sidebar,
            main_position: frame_position,
        };

        animation.save_frame.text = Some(cstr!("Add Frame"));
        animation.save_animation.set_icon(
            &mut handle.begin_drawing(thread),
            VetorIcons::ICON_FILE_EXPORT,
        );
        animation.play.set_icon(
            &mut handle.begin_drawing(thread),
            VetorIcons::ICON_PLAYER_PLAY,
        );
        animation.push_figure(figure.clone());
        animation.push_figure(figure);
        animation.update(handle, thread);
        animation.frames[0].render_screen(&mut handle.begin_drawing(thread), thread);
        animation.frames[0].render_miniature(
            handle,
            thread,
            animation.frame_caroussel.display_width,
            animation.frame_caroussel.display_height,
            animation.video_camera,
        );
        animation
    }

    pub fn update(&mut self, mut handle: &mut RaylibHandle, thread: &RaylibThread) {
        if self.play.activated {
            return self.play(handle, thread);
        }

        if self.save_animation.activated {
            return;
        }

        if self.add_figure.activated {
            let path = FileDialog::new()
                .set_location("./src/assets/figures")
                .add_filter("Vetor Figures", &["vfr"])
                .show_open_single_file()
                .expect("Cannot load file with filesytem");

            if path.is_some() {
                let mut figure = archives::import_figure(
                    path.unwrap().to_str().unwrap(),
                    archives::FileEncoding::RAW,
                );
                figure.center_to(rvec2(BACKGROUND.0 / 2, BACKGROUND.1 / 2));
                self.push_figure(figure);
            }
        }

        if handle.is_key_pressed(KeyboardKey::KEY_DELETE) {
            self.remove_frame();
        }

        let frame_count = self.frames.len() as i32;
        let mut frame = &mut self.frames[self.selected_frame];

        // Update figures and animation state
        for index in 0..frame.figure_animation.len() {
            match frame.figure_animation[index].figure.try_borrow_mut() {
                Ok(mut figure) => {
                    figure.update(handle, self.main_position.add(self.main_scroll));
                }
                _ => (),
            }

            if frame.figure_animation[index]
                .figure
                .try_borrow()
                .ok()
                .unwrap()
                .should_update
            {
                if frame.figure_animation[index]
                    .figure
                    .try_borrow()
                    .ok()
                    .unwrap()
                    .selected
                    .is_some()
                {
                    frame.disable_except(index);
                    break;
                }
            }
        }

        if handle.is_mouse_button_up(MouseButton::MOUSE_BUTTON_LEFT) {
            frame.enable_all();
        }

        if self.save_frame.activated {
            self.push_frame(handle, thread);
        }

        self.frame_caroussel.width = handle.get_screen_width() - self.sidebar.width as i32;
        if handle.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            let max_width = (frame_count * self.frame_caroussel.display_width)
                + (frame_count - 1) * self.frame_caroussel.display_gap;
            let height = if max_width < self.frame_caroussel.width {
                self.frame_caroussel.display_height
            } else {
                self.frame_caroussel.display_height + self.frame_caroussel.scrollbar_height
            };
            let y = handle.get_screen_height() - height;
            let panel_content = rrect(
                self.frame_caroussel.x,
                y,
                self.frame_caroussel.width,
                height,
            );

            // Draw frames
            for i in 0..frame_count {
                let moved_content = i * self.frame_caroussel.display_width
                    + i * self.frame_caroussel.display_gap
                    - self.frame_caroussel.value as i32;

                // We dont need to render frames out of screen
                if moved_content < 0 - self.frame_caroussel.display_width {
                    continue;
                } else if moved_content >= self.frame_caroussel.width {
                    break;
                };

                let frame_rect = rrect(
                    self.frame_caroussel.x + moved_content,
                    y,
                    self.frame_caroussel.display_width,
                    height - self.frame_caroussel.scrollbar_height,
                );
                if unsafe {
                    CheckCollisionPointRec(handle.get_mouse_position().into(), frame_rect.into())
                } && handle.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
                {
                    self.select_frame(i as usize);
                }
            }
        }
    }

    pub fn draw(&mut self, draw_handle: &mut RaylibDrawHandle, thread: &RaylibThread) {
        let mut frame = &mut self.frames[self.selected_frame];

        frame.render_screen(draw_handle, thread);

        let max_scroll_width = (self.frames.len() as i32 * self.frame_caroussel.display_width)
            + (self.frames.len() - 1) as i32 * self.frame_caroussel.display_gap;

        // Draw Main Frame
        {
            let mut main_texture = self.main_texture.try_borrow_mut().ok().unwrap();
            // Draw video camera
            {
                let mut draw = draw_handle.begin_texture_mode(thread, &mut main_texture);

                draw.draw_rectangle_lines_ex(self.video_camera, 1.0, Color::BLACK);
            }
            let mut texture_rec = rrect(
                0,
                0,
                main_texture.texture.width,
                main_texture.texture.height,
            );

            let main_rec = rrect(
                self.main_position.x,
                self.main_position.y,
                draw_handle.get_screen_width() as f32 - self.sidebar.width,
                draw_handle.get_screen_height() as f32
                    - self.main_position.y
                    - (self.frame_caroussel.display_height as f32
                        + if max_scroll_width < self.frame_caroussel.width {
                            0.0
                        } else {
                            self.frame_caroussel.scrollbar_height as f32
                        }),
            );

            let (scissor_rec, main_scroll) =
                draw_handle.gui_scroll_panel(main_rec, None, texture_rec, self.main_scroll);
            self.main_scroll = main_scroll;

            // Draw main screen texture
            let mut scissor = draw_handle.begin_scissor_mode(
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
            // Invert texture rect height to apply correct perspective
            texture_rec.height = -texture_rec.height;
            scissor.draw_texture_rec(
                main_texture.texture(),
                texture_rec,
                self.main_position,
                Color::RAYWHITE.fade(1.0),
            );

            self.previous_mouse_pos = mouse_pos;
        }

        // Draw sidebar
        {
            self.sidebar.height = draw_handle.get_screen_height() as f32 - self.sidebar.y;
            // Background
            draw_handle.draw_rectangle_rec(
                self.sidebar,
                Color::get_color(draw_handle.gui_get_style(
                    GuiControl::DEFAULT,
                    GuiDefaultProperty::BACKGROUND_COLOR as i32,
                ) as u32),
            );

            draw_handle.gui_group_box(
                rrect(
                    self.sidebar.x + 10.0,
                    self.sidebar.y + 10.0,
                    self.sidebar.width - 20.0,
                    self.sidebar.y + 70.0,
                ),
                Some(rstr!("Controles")),
            );

            draw_handle.gui_set_style(
                GuiControl::TOGGLE,
                GuiControlProperty::TEXT_ALIGNMENT as i32,
                GuiTextAlignment::TEXT_ALIGN_CENTER as i32,
            );

            draw_handle.gui_set_style(
                GuiControl::SLIDER,
                GuiSliderProperty::SLIDER_PADDING as i32,
                0,
            );

            draw_handle.gui_set_style(
                GuiControl::SLIDER,
                GuiSliderProperty::SLIDER_WIDTH as i32,
                10,
            );

            draw_handle.gui_label(
                rrect(15, self.sidebar.y + 55.0, self.sidebar.width - 30.0, 30),
                Some(cstr!(format!("FPS {}", self.framerate)).as_c_str()),
            );

            self.framerate = draw_handle.gui_slider(
                rrect(
                    self.sidebar.x + 15.0,
                    self.sidebar.y + 85.0,
                    self.sidebar.width - 30.0,
                    20.0,
                ),
                None,
                None,
                self.framerate,
                1.0,
                30.0,
            ) as i32 as f32;

            self.save_frame.activated = draw_handle.gui_button(
                rrect(
                    self.save_frame.start.x,
                    self.save_frame.start.y,
                    self.sidebar.width - 20.0,
                    30,
                ),
                Some(self.save_frame.text.clone().unwrap().as_c_str()),
            );

            self.save_animation.activated = draw_handle.gui_toggle(
                rrect(
                    self.save_animation.start.x,
                    self.save_animation.start.y,
                    self.save_animation.len,
                    30,
                ),
                Some(self.save_animation.text.clone().unwrap().as_c_str()),
                self.save_animation.activated,
            ) && !self.play.activated;

            let mut play_toggle = self.play.activated;
            self.play.activated = draw_handle.gui_toggle(
                rrect(self.play.start.x, self.play.start.y, self.play.len, 30),
                Some(self.play.text.clone().unwrap().as_c_str()),
                self.play.activated,
            );

            play_toggle = self.play.activated != play_toggle;

            if play_toggle {
                if self.play.activated {
                    for figure in &self.figures {
                        figure.try_borrow_mut().ok().unwrap().draw_option.point = false;
                    }

                    self.previous_time = draw_handle.get_time();
                    self.select_frame(0);
                } else {
                    for figure in &self.figures {
                        figure.try_borrow_mut().ok().unwrap().draw_option.point = true;
                    }

                    self.select_frame((self.frames.len() - 1) as usize);
                }
            }

            draw_handle.gui_button(
                rrect(10, self.sidebar.y + 120.0, self.sidebar.width - 20.0, 30),
                Some(rstr!("Background")),
            );

            self.add_figure.activated = draw_handle.gui_button(
                rrect(
                    self.add_figure.start.x,
                    self.add_figure.start.y,
                    self.sidebar.width - 20.0,
                    30,
                ),
                Some(rstr!("Add Figure")),
            );
        }

        // Draw animation frames
        {
            let mut scrollbar_height = self.frame_caroussel.scrollbar_height;
            let height = if max_scroll_width < self.frame_caroussel.width {
                scrollbar_height = 0;
                self.frame_caroussel.display_height
            } else {
                self.frame_caroussel.display_height + scrollbar_height
            };
            let y = draw_handle.get_screen_height() - height;
            let panel_content = rrect(
                self.frame_caroussel.x,
                y,
                self.frame_caroussel.width,
                height,
            );

            // Avoid render out of scissor area
            let mut scissor = draw_handle.begin_scissor_mode(
                panel_content.x as i32,
                panel_content.y as i32,
                panel_content.width as i32,
                panel_content.height as i32,
            );

            // Background
            scissor.draw_rectangle(
                panel_content.x as i32,
                panel_content.y as i32,
                panel_content.width as i32,
                panel_content.height as i32,
                Color::get_color(scissor.gui_get_style(
                    GuiControl::DEFAULT,
                    GuiControlProperty::BASE_COLOR_FOCUSED as i32,
                ) as u32),
            );

            // Draw frames
            for i in 0..self.frames.len() {
                let moved_content = i as i32 * self.frame_caroussel.display_width
                    + i as i32 * self.frame_caroussel.display_gap
                    - self.frame_caroussel.value as i32;

                // We dont need to render frames out of screen
                if moved_content < 0 - self.frame_caroussel.display_width {
                    continue;
                } else if moved_content >= self.frame_caroussel.width {
                    break;
                };

                // Draw frame
                scissor.draw_texture_rec(
                    self.frames[i as usize].miniature.as_ref().unwrap(),
                    rrect(0, 0, self.frame_caroussel.display_width, height),
                    rvec2(self.frame_caroussel.x + moved_content, y),
                    Color::RAYWHITE.fade(1.0),
                );

                scissor.draw_rectangle_lines_ex(
                    rrect(
                        self.frame_caroussel.x + moved_content,
                        y,
                        self.frame_caroussel.display_width,
                        height - scrollbar_height,
                    ),
                    3.0,
                    if i as usize == self.selected_frame {
                        Color::get_color(scissor.gui_get_style(
                            GuiControl::DEFAULT,
                            GuiControlProperty::BORDER_COLOR_PRESSED as i32,
                        ) as u32)
                    } else {
                        Color::get_color(scissor.gui_get_style(
                            GuiControl::DEFAULT,
                            GuiControlProperty::BORDER_COLOR_NORMAL as i32,
                        ) as u32)
                    },
                );
            }

            // Avoid draw scrollbar when frames is less then width
            if max_scroll_width > self.frame_caroussel.width {
                let max_value = (self.frame_caroussel.display_width * self.frames.len() as i32)
                    as f32
                    + (self.frame_caroussel.display_gap * (self.frames.len() as i32 - 1)) as f32
                    - self.frame_caroussel.width as f32;

                scissor.gui_set_style(
                    GuiControl::SLIDER,
                    GuiSliderProperty::SLIDER_WIDTH as i32,
                    (self.frame_caroussel.width - (max_scroll_width - self.frame_caroussel.width))
                        .max(50),
                );

                // Fit frame scroll at mouse scrolling
                if panel_content.check_collision_point_rec(scissor.get_mouse_position()) {
                    self.frame_caroussel.value = if scissor.get_mouse_wheel_move() != 0.0 {
                        let mut next_move = self.frame_caroussel.value
                            + scissor.get_mouse_wheel_move()
                                * -self.frame_caroussel.display_width as f32;
                        if next_move < 0.0 {
                            next_move = 0.0;
                        } else if next_move > max_value + self.frame_caroussel.display_width as f32
                        {
                            next_move = max_value;
                        }

                        next_move
                    } else {
                        self.frame_caroussel.value
                    };
                }

                self.frame_caroussel.value = scissor.gui_slider(
                    rrect(
                        self.frame_caroussel.x,
                        scissor.get_screen_height() - scrollbar_height,
                        self.frame_caroussel.width,
                        scrollbar_height,
                    ),
                    None,
                    None,
                    self.frame_caroussel.value,
                    0.0,
                    max_value,
                );
            }
        }

        if self.save_animation.activated {
            self.draw_export_dialog(draw_handle, thread);
        }
    }

    fn draw_export_dialog(&mut self, draw_handle: &mut RaylibDrawHandle, thread: &RaylibThread) {
        let w = draw_handle.get_screen_width();
        let h = draw_handle.get_screen_height();

        // Draw transparent background
        draw_handle.draw_rectangle(
            (self.sidebar.x + self.sidebar.width) as i32,
            self.sidebar.y as i32,
            w,
            h,
            Color::get_color(draw_handle.gui_get_style(
                GuiControl::DEFAULT,
                GuiControlProperty::BASE_COLOR_NORMAL as i32,
            ) as u32)
            .fade(0.3),
        );

        let dialog_rect = rrect((w / 2) - 150, (h / 2) - 100, 300, 136);

        self.save_animation.activated =
            !draw_handle.gui_window_box(dialog_rect, Some(rstr!("Exportar como:")));

        let export = draw_handle.gui_combo_box(
            rrect(dialog_rect.x + 25.0, dialog_rect.y + 40.0, 120, 30),
            Some(rstr!("mp4;gif")),
            self.export_format as i32,
        );

        self.export_format = match export {
            0 => ExportFormat::MP4,
            1 => ExportFormat::GIF,
            _ => ExportFormat::MP4,
        };

        if draw_handle.gui_button(
            rrect(dialog_rect.x + 25.0, dialog_rect.y + 80.0, 120, 30),
            Some(rstr!("Exportar")),
        ) {
            match self.export_format {
                ExportFormat::GIF => {
                    self.export("unnamed", "gif");
                }
                ExportFormat::MP4 => {
                    self.export("unnamed", "mp4");
                }
            }
            println!();

            self.save_animation.activated = false;
        }

        let mut save = match self.save_format {
            FileEncoding::RAW => 0,
            FileEncoding::GZIP => 1,
            FileEncoding::ZLIB => 2,
        };

        save = draw_handle.gui_combo_box(
            rrect(dialog_rect.x + 154.0, dialog_rect.y + 40.0, 120, 30),
            Some(rstr!("raw;gzip;zlib")),
            save as i32,
        );

        self.save_format = match save {
            0 => FileEncoding::RAW,
            1 => FileEncoding::GZIP,
            _ => FileEncoding::ZLIB,
        };

        if draw_handle.gui_button(
            rrect(dialog_rect.x + 154.0, dialog_rect.y + 80.0, 120, 30),
            Some(rstr!("Salvar")),
        ) {
            self.save("unnamed");
            self.save_animation.activated = false;
        }
    }

    pub fn push_figure(&mut self, figure: Figure) {
        self.figures.push(Rc::new(RefCell::new(figure.clone())));

        let mut frame = &mut self.frames[self.selected_frame];
        frame.figure_animation.push(FigureAnimation {
            global_index: self.figures.len() - 1,
            local_index: frame.figure_animation.len(),
            moved_edges: figure.scan(),
            figure: self.figures.last().unwrap().clone(),
        });
    }

    fn select_frame(&mut self, index: usize) {
        let mut frame = &mut self.frames[self.selected_frame];
        frame.is_selected = false;
        frame = &mut self.frames[index];
        frame.is_selected = true;
        self.main_texture = frame.texture.clone();
        self.selected_frame = index;

        for fig in &mut frame.figure_animation {
            fig.figure
                .try_borrow_mut()
                .ok()
                .unwrap()
                .load_state(fig.moved_edges.clone());
        }
    }

    pub fn push_frame(&mut self, handle: &mut RaylibHandle, thread: &RaylibThread) {
        let mut frame = &mut self.frames[self.selected_frame];

        let texture = handle
            .load_render_texture(
                thread,
                frame.texture.try_borrow().ok().unwrap().width() as u32,
                frame.texture.try_borrow().ok().unwrap().height() as u32,
            )
            .ok()
            .unwrap();

        frame.chage_figure_draw(false);

        let mut new_frame = frame.clone(texture);
        new_frame.render_screen(&mut handle.begin_drawing(thread), thread);
        new_frame.render_miniature(
            handle,
            thread,
            self.frame_caroussel.display_width,
            self.frame_caroussel.display_height,
            self.video_camera,
        );
        new_frame.save_state();
        frame.is_selected = false;
        frame.render_miniature(
            handle,
            thread,
            self.frame_caroussel.display_width,
            self.frame_caroussel.display_height,
            self.video_camera,
        );
        frame.save_state();

        frame.chage_figure_draw(true);

        self.main_texture = new_frame.texture.clone();
        self.selected_frame = self.frames.len();
        self.frames.push(new_frame);
    }

    pub fn remove_frame(&mut self) {
        self.frames.remove(self.selected_frame);
        self.selected_frame = self.frames.len() - 1;

        let mut frame = &mut self.frames[self.selected_frame];
        frame.is_selected = true;
        self.main_texture = frame.texture.clone();
        self.frame_caroussel.value = 0.0;
    }

    fn play(&mut self, handle: &mut RaylibHandle, thread: &RaylibThread) {
        let current_time = handle.get_time();
        let frame_time = current_time - self.previous_time;
        let wait_time: f64 = (1.0 / self.framerate as f64) - frame_time;

        if wait_time <= 0.0 {
            let index = if self.selected_frame == self.frames.len() - 1 {
                0
            } else {
                self.selected_frame + 1
            };
            self.select_frame(index);
            self.previous_time = current_time;
        }
    }

    // External files:
    /// Save animation into a file with raw format
    fn save(&mut self, filename: &str) {
        let extension = match self.save_format {
            FileEncoding::RAW => "var",
            FileEncoding::GZIP => "vag",
            FileEncoding::ZLIB => "vaz",
        };

        let path = FileDialog::new()
            .set_filename(&(filename.to_owned() + "." + extension))
            .add_filter("Vetor Animation", &["var", "vag", "vaz"])
            .show_save_single_file()
            .expect("Cannot save file");

        if path.is_none() {
            return;
        }

        let mut file = ZlibEncoder::new(vec![], Compression::default());

        // Save Figures
        for (i, figRef) in self.figures.iter().enumerate() {
            let figure = figRef.try_borrow().ok().unwrap();
            let points = archives::figure_to_raw(figure.clone());

            for point in points {
                file.write(
                    format!(
                        "{},{},{},{},{}\n",
                        point.typ, point.x, point.y, point.parent, point.index
                    )
                    .as_bytes(),
                )
                .expect("Cannot write points to file");
            }

            // Add interssection only in between figures
            if i != self.figures.len() - 1 {
                file.write("^\n".as_bytes()).ok();
            };
        }

        for (frame_index, frame) in self.frames.iter_mut().enumerate() {
            file.write(format!("@Frame {}\n", frame_index).as_bytes())
                .ok();
            for figState in &mut frame.figure_animation {
                file.write(format!("^{}\n", figState.global_index).as_bytes())
                    .ok();

                figState
                    .figure
                    .try_borrow_mut()
                    .ok()
                    .unwrap()
                    .load_state(figState.moved_edges.clone());
                figState.moved_edges = figState.figure.try_borrow().ok().unwrap().scan();
                let mut moved_edges: Vec<_> = figState.moved_edges.iter().collect();

                moved_edges.sort_by(|a, b| {
                    if a.0 == b.0 {
                        return Ordering::Equal;
                    } else if a.0 < b.0 {
                        return Ordering::Less;
                    }

                    Ordering::Greater
                });

                for (_, edge) in moved_edges {
                    file.write(
                        format!("{},{},{},{}\n", edge.0.x, edge.0.y, edge.1.x, edge.1.y).as_bytes(),
                    )
                    .ok();
                }
            }
        }

        fs::write(path.unwrap(), file.finish().unwrap()).ok();
    }

    /// Load animation scenes from raw file
    pub fn load(
        path: &str,
        handle: &mut RaylibHandle,
        thread: &RaylibThread,
        encoding: FileEncoding,
    ) -> Animation {
        let buffer = fs::read(path).ok().unwrap();
        let file = match encoding {
            FileEncoding::RAW => String::from_utf8(buffer).unwrap(),
            FileEncoding::GZIP => {
                let mut decoder = GzDecoder::new(buffer.as_slice());
                let mut raw = String::new();
                decoder.read_to_string(&mut raw);
                raw
            }
            FileEncoding::ZLIB => {
                let mut decoder = ZlibDecoder::new(buffer.as_slice());
                let mut raw = String::new();
                decoder.read_to_string(&mut raw);
                raw
            }
        };

        let split: Vec<_> = file.split("@Frame").collect();
        let (figs, frames) = (split[0], &split[1..]);
        let mut animation = Animation::new(handle, thread);

        animation.figures = vec![];
        animation.frames[0].figure_animation = vec![];

        for fig in figs.split("^\n") {
            animation
                .figures
                .push(Rc::new(RefCell::new(archives::raw_to_figure(fig))));
        }

        let mut last_frame = animation.frames.first_mut().unwrap();

        let center = rvec2(BACKGROUND.0 / 2, BACKGROUND.1 / 2);

        // Map and mount each frame
        for (i, frameStr) in frames.iter().enumerate() {
            last_frame.figure_animation = vec![];

            // Map and mount each Figure in figure_animation
            for state in frameStr.split_once("\n").unwrap().1.split("^") {
                if state.trim().len() == 0 {
                    continue;
                }

                let mut lines = state.lines();
                let index = lines.next().unwrap().parse::<usize>().ok().unwrap();
                let mut moved_edges = HashMap::new();

                // Map and moutn each figure edge state
                for line in lines {
                    let edge: Vec<_> = line.split(",").collect();

                    if edge.len() < 4 {
                        continue;
                    }

                    let start = rvec2(
                        edge[0].parse::<i32>().unwrap(),
                        edge[1].parse::<i32>().unwrap(),
                    );
                    let end = rvec2(
                        edge[2].parse::<i32>().unwrap(),
                        edge[3].parse::<i32>().unwrap(),
                    );

                    moved_edges.insert(moved_edges.len() as usize, (start, end));
                }

                let mut figure = animation.figures[index].try_borrow_mut().ok().unwrap();

                for (_, e) in moved_edges.iter_mut() {
                    e.0 = e.0.add(center);
                    e.1 = e.1.add(center);
                }

                figure.load_state(moved_edges.clone());

                last_frame.figure_animation.push(FigureAnimation {
                    global_index: index,
                    local_index: last_frame.figure_animation.len(),
                    figure: animation.figures[index].clone(),
                    moved_edges,
                });
            }

            last_frame.save_state();
            last_frame.chage_figure_draw(false);
            last_frame.render_screen(&mut handle.begin_drawing(thread), thread);
            last_frame.render_miniature(
                handle,
                thread,
                animation.frame_caroussel.display_width,
                animation.frame_caroussel.display_height,
                animation.video_camera,
            );
            last_frame.chage_figure_draw(true);
            last_frame.render_screen(&mut handle.begin_drawing(thread), thread);

            if i < frames.len() - 1 {
                let texture = handle
                    .load_render_texture(
                        thread,
                        last_frame.texture.try_borrow().ok().unwrap().width() as u32,
                        last_frame.texture.try_borrow().ok().unwrap().height() as u32,
                    )
                    .ok()
                    .unwrap();

                let mut new_frame = last_frame.clone(texture);
                last_frame.is_selected = false;

                animation.main_texture = new_frame.texture.clone();
                animation.selected_frame = animation.frames.len();
                animation.frames.push(new_frame);
                last_frame = animation.frames.last_mut().unwrap();
            }
        }

        animation
    }

    pub fn export(&mut self, file: &str, format: &str) {
        let filename = &format!("{}.{}", file, format);
        let path = FileDialog::new()
            .set_filename(filename)
            .add_filter("Video", &["gif", "mp4"])
            .show_save_single_file()
            .expect("Cannot save file");

        if path.is_none() {
            return;
        }

        fs::remove_file(path.clone().unwrap()).err();

        let mut ffmpeg = Command::new("ffmpeg")
            .args(["-framerate", &self.framerate.to_string(), "-i", "-"])
            .args([path.unwrap().to_str().unwrap()])
            .stdin(Stdio::piped())
            .spawn()
            .expect("Cannot spawn ffmpeg command");

        let stdin = ffmpeg.stdin.as_mut().unwrap();

        // NOTE: I don't like the ideia of saving image into a file,
        // but this works now and can it be refactored latter.
        let tmp_file = "tmp.png";

        for frame in &mut self.frames {
            let texture = frame.texture.try_borrow().ok().unwrap();
            let mut image = texture.load_image().unwrap();
            image.flip_vertical();
            image.crop(self.video_camera);
            image.export_image(tmp_file);
            stdin.write_all(fs::read(tmp_file).ok().unwrap().as_slice());
        }

        fs::remove_file(tmp_file);

        stdin.flush().expect("Cannot flush ffmpeg stdin");
        ffmpeg.wait();
    }
}
