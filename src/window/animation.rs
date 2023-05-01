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
    frame_texture: RenderTexture2D,
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

        let width = handle.get_screen_width() - 80;
        let height = 110;
        let frame_texture = handle
            .load_render_texture(&thread, width as u32, height as u32)
            .ok()
            .unwrap();

        Animation {
            figures: vec![],
            frames: vec![first_frame],
            selected_frame: 0,
            main_texture,
            frame_texture,
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

        if handle.is_mouse_button_up(MouseButton::MOUSE_LEFT_BUTTON) {
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

        let width = self.frame_texture.width();
        let height = self.frame_texture.height();
        let x = (self.start.x + sidebar_width) as i32;
        let y = handle.get_screen_height() - height;

        // Draw animation frames
        {
            let mut draw_texture = handle.begin_texture_mode(thread, &mut self.frame_texture);

            // Background
            draw_texture.draw_rectangle(
                0,
                0,
                width,
                height,
                Color::from(ColorStyle::BASE_COLOR_FOCUSED),
            );

            let frame_width = 150;
            let frame_gap = 10;
            let scrollbar_height = 10;
            for i in 0..5 {
                // We dont need to render frames out of screen
                if i * frame_width + i * frame_gap > width {
                    break;
                };

                // Draw frame
                draw_texture.draw_rectangle_lines(
                    i * frame_width + i * frame_gap,
                    scrollbar_height,
                    frame_width,
                    height - scrollbar_height,
                    Color::BLACK,
                );
            }

            // Draw scrollbar
            draw_texture.draw_rectangle(0, 0, width, scrollbar_height, Color::RAYWHITE);
        }
        // Draw frame animation texture
        handle.draw_texture(
            self.frame_texture.texture(),
            x,
            y,
            Color::RAYWHITE.fade(1.0),
        );
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
