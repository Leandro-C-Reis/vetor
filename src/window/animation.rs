use crate::figure::Figure;
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
    figures: Vec<Figure>,
    frames: Vec<Frame>,
    selected_frame: usize,
    framebuffer: RenderTexture2D,
}

impl Animation {
    pub fn new(framebuffer: RenderTexture2D) -> Animation {
        let mut first_frame = Frame::new();
        first_frame.is_selected = true;

        Animation {
            figures: vec![],
            frames: vec![first_frame],
            selected_frame: 0,
            framebuffer,
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

        {
            let mut draw_texture = handle.begin_texture_mode(thread, &mut self.framebuffer);

            for animation in &mut frame.figure_animation {
                if animation.figure.is_some() {
                    animation.figure.as_mut().unwrap().draw(&mut draw_texture);
                }
            }
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
