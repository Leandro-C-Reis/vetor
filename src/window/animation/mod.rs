pub mod frame;

use self::frame::*;
use super::util::button::Button;
use crate::{cstr, figure::Figure, imports, maths::*};
use raylib::{
    ffi::{CheckCollisionPointRec, ImageBlurGaussian},
    prelude::*,
};
use std::{borrow::BorrowMut, cell::RefCell, collections::HashMap, ffi::CString, rc::Rc};

pub struct Animation {
    figures: Vec<Rc<RefCell<Figure>>>,
    frames: Vec<Frame>,
    selected_frame: usize,
    main_texture: Rc<RefCell<RenderTexture2D>>,
    frame_scroll: f32,
    frame_position: Vector2,
    sidebar: Rectangle,
    sidebar_play: Button,
}

impl Animation {
    pub fn new(handle: &mut RaylibHandle, thread: &RaylibThread) -> Animation {
        let sidebar = rrect(0, 30, 100, handle.get_screen_height() - 30);
        let frame_position = rvec2(80, 30);

        let mut first_frame = Frame::new(
            handle,
            thread,
            handle.get_screen_width() as u32 - frame_position.x as u32,
            handle.get_screen_height() as u32 - 100 - frame_position.y as u32 as u32,
        );
        first_frame.is_selected = true;

        let figure = imports::bin::import_from_raw(
            "men.vec",
            rvec2(
                first_frame.texture.borrow().width() / 2,
                first_frame.texture.borrow().height() / 2,
            ),
        );

        let mut animation = Animation {
            selected_frame: 0,
            frame_scroll: 0.0,
            sidebar_play: Button::new(rvec2(sidebar.x, sidebar.y).add(rvec2(10, 10))),
            main_texture: first_frame.texture.clone(),
            figures: vec![],
            frames: vec![first_frame],
            sidebar,
            frame_position,
        };

        animation.sidebar_play.text = Some(cstr!("Add Frame"));
        animation.push_figure(figure.clone());
        animation.push_figure(figure);
        animation.update(handle, thread);
        animation.frames[0].render_screen(&mut handle.begin_drawing(thread), thread);
        animation.frames[0].render_miniature(handle, thread);
        animation
    }

    pub fn update(&mut self, mut handle: &mut RaylibHandle, thread: &RaylibThread) {
        let frame_count = self.frames.len() as i32;
        let mut frame = &mut self.frames[self.selected_frame];

        // Update figures and animation state
        for index in 0..frame.figure_animation.len() {
            match frame.figure_animation[index].figure.try_borrow_mut() {
                Ok(mut figure) => {
                    figure.update(handle, rvec2(80, 30));
                }
                _ => (),
            }

            if frame.figure_animation[index].figure.borrow().should_update {
                if frame.figure_animation[index]
                    .figure
                    .borrow()
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

        if self.sidebar_play.activated {
            let texture = handle
                .load_render_texture(
                    thread,
                    frame.texture.borrow().width() as u32,
                    frame.texture.borrow().height() as u32,
                )
                .ok()
                .unwrap();

            let mut new_frame = frame.clone(texture);
            new_frame.render_screen(&mut handle.begin_drawing(thread), thread);
            new_frame.render_miniature(handle, thread);
            new_frame.save_state();
            frame.is_selected = false;
            frame.render_miniature(handle, thread);
            frame.save_state();

            self.main_texture = new_frame.texture.clone();
            self.selected_frame = self.frames.len();
            self.frames.push(new_frame);
        }

        if handle.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            let frame_width = 150;
            let frame_gap = 10;
            let max_width = (frame_count * frame_width) + (frame_count - 1) * frame_gap;
            let mut scrollbar_height = 15;
            let width = handle.get_screen_width() - 80;
            let height = if max_width < width {
                scrollbar_height = 0;
                100
            } else {
                100 + scrollbar_height
            };
            let x = (self.sidebar.x + self.sidebar.width) as i32;
            let y = handle.get_screen_height() - height;
            let panel_content = rrect(x, y, width, height);

            // Draw frames
            for i in 0..frame_count {
                let moved_content = i * frame_width + i * frame_gap - self.frame_scroll as i32;

                // We dont need to render frames out of screen
                if moved_content < 0 - frame_width {
                    continue;
                } else if moved_content >= width {
                    break;
                };

                let frame_rect =
                    rrect(x + moved_content, y, frame_width, height - scrollbar_height);
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

        // Draw main screen texture
        draw_handle.draw_texture_rec(
            self.main_texture.borrow().texture(),
            rrect(
                0,
                0,
                self.main_texture.borrow().texture.width,
                -self.main_texture.borrow().texture.height,
            ),
            rvec2(self.sidebar.x + self.sidebar.width, self.sidebar.y),
            Color::RAYWHITE.fade(1.0),
        );

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

            self.sidebar_play.activated = draw_handle.gui_button(
                rrect(
                    self.sidebar_play.start.x,
                    self.sidebar_play.start.y,
                    self.sidebar.width - 20.0,
                    30,
                ),
                Some(self.sidebar_play.text.clone().unwrap().as_c_str()),
            );
        }

        // Draw animation frames
        {
            let frame_count = self.frames.len() as i32;
            let frame_width = 150;
            let frame_gap = 10;
            let max_width = (frame_count * frame_width) + (frame_count - 1) * frame_gap;
            let mut scrollbar_height = 15;
            let width = draw_handle.get_screen_width() - 80;
            let height = if max_width < width {
                scrollbar_height = 0;
                100
            } else {
                100 + scrollbar_height
            };
            let x = (self.sidebar.x + self.sidebar.width) as i32;
            let y = draw_handle.get_screen_height() - height;
            let panel_content = rrect(x, y, width, height);

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
            for i in 0..frame_count {
                let moved_content = i * frame_width + i * frame_gap - self.frame_scroll as i32;

                // We dont need to render frames out of screen
                if moved_content < 0 - frame_width {
                    continue;
                } else if moved_content >= width {
                    break;
                };

                // Draw frame
                scissor.draw_texture_rec(
                    self.frames[i as usize].miniature.as_ref().unwrap(),
                    rrect(0, 0, frame_width, height),
                    rvec2(x + moved_content, y),
                    Color::RAYWHITE.fade(1.0),
                );

                scissor.draw_rectangle_lines_ex(
                    rrect(x + moved_content, y, frame_width, height - scrollbar_height),
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

            // NOTE: Maybe the slider width will be change dynamically
            scissor.gui_set_style(
                GuiControl::SLIDER,
                GuiSliderProperty::SLIDER_WIDTH as i32,
                300,
            );

            // Avoid draw scrollbar when frames is less then width
            if max_width > width {
                let max_value = (frame_width * frame_count) as f32
                    + (frame_gap * (frame_count - 1)) as f32
                    - width as f32;

                // Fit frame scroll at mouse scrolling
                if panel_content.check_collision_point_rec(scissor.get_mouse_position()) {
                    self.frame_scroll = if scissor.get_mouse_wheel_move() != 0.0 {
                        let mut next_move = self.frame_scroll
                            + scissor.get_mouse_wheel_move() * -frame_width as f32;
                        if next_move < 0.0 {
                            next_move = 0.0;
                        } else if next_move > max_value + frame_width as f32 {
                            next_move = max_value;
                        }

                        next_move
                    } else {
                        self.frame_scroll
                    };
                }

                self.frame_scroll = scissor.gui_slider(
                    rrect(
                        x,
                        scissor.get_screen_height() - scrollbar_height,
                        width,
                        scrollbar_height,
                    ),
                    None,
                    None,
                    self.frame_scroll,
                    0.0,
                    max_value,
                );
            }
        }
    }

    pub fn push_figure(&mut self, figure: Figure) {
        self.figures.push(Rc::new(RefCell::new(figure)));

        let mut frame = &mut self.frames[self.selected_frame];
        frame.figure_animation.push(FigureAnimation {
            global_index: self.figures.len() - 1,
            local_index: frame.figure_animation.len(),
            moved_edges: HashMap::new(),
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
}
