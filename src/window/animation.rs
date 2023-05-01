use crate::{figure::Figure, styles::ColorStyle};
use raylib::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
struct FigureAnimation {
    global_index: usize,
    local_index: usize,
    moved_edges: HashMap<usize, (Vector2, Vector2)>,
    figure: Option<Figure>,
}

#[derive(Debug, Clone, PartialEq)]
struct Frame {
    figure_animation: Vec<FigureAnimation>,
    is_selected: bool,
}

impl Frame {
    fn new() -> Frame {
        Frame {
            figure_animation: vec![],
            is_selected: false,
        }
    }

    fn disable_except(&mut self, index: usize) {
        for i in 0..self.figure_animation.len() {
            if i == index {
                continue;
            }

            self.figure_animation[i]
                .figure
                .as_mut()
                .unwrap()
                .should_update = false;
        }
    }

    fn enable_all(&mut self) {
        for animation in &mut self.figure_animation {
            animation.figure.as_mut().unwrap().should_update = true;
        }
    }
}

pub struct Animation {
    start: Vector2,
    figures: Vec<Figure>,
    frames: Vec<Frame>,
    selected_frame: usize,
    main_texture: RenderTexture2D,
    frame_scroll: f32,
}

impl Animation {
    pub fn new(handle: &mut RaylibHandle, thread: &RaylibThread) -> Animation {
        let start = Vector2::new(0.0, 30.0);
        let mut first_frame = Frame::new();
        first_frame.is_selected = true;

        let main_texture = handle
            .load_render_texture(
                &thread,
                handle.get_screen_width() as u32,
                handle.get_screen_height() as u32,
            )
            .ok()
            .unwrap();

        Animation {
            figures: vec![],
            frames: vec![first_frame],
            selected_frame: 0,
            frame_scroll: 0.0,
            main_texture,
            start,
        }
    }

    pub fn update(&mut self, handle: &RaylibHandle) {
        let mut frame = &mut self.frames[self.selected_frame];

        // Update figures and animation state
        for index in 0..frame.figure_animation.len() {
            let mut animation = &mut frame.figure_animation[index];

            match animation.figure.as_mut() {
                Some(figure) => {
                    figure.update(handle);

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

    pub fn draw(&mut self, handle: &mut RaylibDrawHandle, thread: &RaylibThread) {
        let mut frame = &mut self.frames[self.selected_frame];

        // Draw figures on texture
        {
            let mut draw_texture = handle.begin_texture_mode(thread, &mut self.main_texture);

            for animation in &mut frame.figure_animation {
                if animation.figure.is_some() {
                    animation.figure.as_mut().unwrap().draw(&mut draw_texture);
                }
            }
        }

        // Draw main screen texture
        handle.draw_texture_rec(
            self.main_texture.texture(),
            rrect(
                0.0,
                0.0,
                self.main_texture.texture.width,
                -self.main_texture.texture.height,
            ),
            Vector2::new(0.0, 0.0),
            Color::RAYWHITE.fade(1.0),
        );

        let sidebar_width = 80.0;

        // Draw sidebar
        {
            // Background
            handle.draw_rectangle(
                self.start.x as i32,
                self.start.y as i32,
                sidebar_width as i32,
                handle.get_screen_height() - self.start.y as i32,
                Color::from(ColorStyle::BASE_COLOR_NORMAL),
            );
        }

        // Draw animation frames
        {
            let width = handle.get_screen_width() - 80;
            let height = 115;
            let x = (self.start.x + sidebar_width) as i32;
            let y = handle.get_screen_height() - height;
            let frame_count = 15;
            let frame_width = 150;
            let frame_gap = 10;
            let scrollbar_height = 15;
            let panel_content = rrect(x, y, width, height);

            // Avoid render out of scissor area
            let mut scissor = handle.begin_scissor_mode(
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
                Color::from(ColorStyle::BASE_COLOR_FOCUSED),
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
                scissor.draw_rectangle_lines(
                    x + moved_content,
                    y,
                    frame_width,
                    height - scrollbar_height,
                    Color::BLACK,
                );
                scissor.draw_text(
                    &format!("{}", i + 1),
                    x + moved_content + (frame_width / 2),
                    y + (height / 2) - 12,
                    24,
                    Color::BLACK,
                );
            }

            // NOTE: Maybe the slider width will be change dynamically
            scissor.gui_set_style(
                GuiControl::SLIDER,
                GuiSliderProperty::SLIDER_WIDTH as i32,
                300,
            );

            let max_value = (frame_width * frame_count) as f32
                + (frame_gap * (frame_count - 1)) as f32
                - width as f32;

            // Fit frame scroll at mouse scrolling
            if panel_content.check_collision_point_rec(scissor.get_mouse_position()) {
                self.frame_scroll = if scissor.get_mouse_wheel_move() != 0.0 {
                    let mut next_move = self.frame_scroll + scissor.get_mouse_wheel_move() * -150.0;
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
            )
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
