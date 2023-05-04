pub mod frame;

use self::frame::*;
use crate::{figure::Figure, imports};
use raylib::{ffi::ImageBlurGaussian, prelude::*};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub struct Animation {
    figures: Vec<Figure>,
    frames: Vec<Frame>,
    selected_frame: usize,
    main_texture: Rc<RefCell<RenderTexture2D>>,
    frame_scroll: f32,
    frame_position: Vector2,
    sidebar_position: Vector2,
}

impl Animation {
    pub fn new(handle: &mut RaylibHandle, thread: &RaylibThread) -> Animation {
        let sidebar_position = Vector2::new(0.0, 30.0);
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
            main_texture: first_frame.texture.clone(),
            figures: vec![],
            frames: vec![first_frame],
            sidebar_position,
            frame_position,
        };

        animation.push_figure(figure.clone());
        animation.push_figure(figure);
        animation.update(handle, thread);
        animation.frames[0].render_screen(&mut handle.begin_drawing(thread), thread);
        animation.frames[0].render_miniature(handle, thread);
        animation
    }

    pub fn update(&mut self, mut handle: &mut RaylibHandle, thread: &RaylibThread) {
        let mut frame = &mut self.frames[self.selected_frame];

        // Update figures and animation state
        for index in 0..frame.figure_animation.len() {
            let mut animation = &mut frame.figure_animation[index];

            match animation.figure.as_mut() {
                Some(figure) => {
                    figure.update(handle, rvec2(80, 30));

                    if figure.should_update {
                        animation.moved_edges = self.figures[animation.global_index]
                            .diff(figure.clone())
                            .unwrap();

                        if figure.selected.is_some() {
                            frame.disable_except(index);
                        }
                    }
                }
                None => {
                    animation.figure = Some(self.figures[animation.global_index].clone());
                }
            }
        }

        if handle.is_mouse_button_up(MouseButton::MOUSE_BUTTON_LEFT) {
            frame.enable_all();
        }
    }

    pub fn draw(&mut self, draw_handle: &mut RaylibDrawHandle, thread: &RaylibThread) {
        let mut frame = &mut self.frames[self.selected_frame];
        let sidebar_width = 80.0;

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
            rvec2(
                self.sidebar_position.x + sidebar_width,
                self.sidebar_position.y,
            ),
            Color::RAYWHITE.fade(1.0),
        );

        // Draw sidebar
        {
            // Background
            draw_handle.draw_rectangle(
                self.sidebar_position.x as i32,
                self.sidebar_position.y as i32,
                sidebar_width as i32,
                draw_handle.get_screen_height() - self.sidebar_position.y as i32,
                Color::get_color(draw_handle.gui_get_style(
                    GuiControl::DEFAULT,
                    GuiDefaultProperty::BACKGROUND_COLOR as i32,
                ) as u32),
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
            let x = (self.sidebar_position.x + sidebar_width) as i32;
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
                scissor.draw_rectangle_lines(
                    x + moved_content,
                    y,
                    frame_width,
                    height - scrollbar_height,
                    Color::BLACK,
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
        self.figures.push(figure.clone());

        let mut frame = &mut self.frames[self.selected_frame];
        frame.figure_animation.push(FigureAnimation {
            global_index: self.figures.len() - 1,
            local_index: frame.figure_animation.len(),
            moved_edges: HashMap::new(),
            figure: Some(figure),
        });
    }
}
