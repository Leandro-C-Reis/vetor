pub mod frame;

use self::frame::*;
use super::util::button::Button;
use crate::{
    archives::{self, figure_to_raw},
    cstr,
    figure::Figure,
    maths::*,
};
use raylib::{
    ffi::{CheckCollisionPointRec, ImageBlurGaussian},
    prelude::*,
};
use regex::Regex;
use std::{
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
    cmp::Ordering,
    collections::HashMap,
    ffi::CString,
    fs::{self, File},
    io::{Read, Write},
    rc::Rc,
};

pub struct Animation {
    figures: Vec<Rc<RefCell<Figure>>>,
    frames: Vec<Frame>,
    selected_frame: usize,
    main_texture: Rc<RefCell<RenderTexture2D>>,
    frame_scroll: f32,
    frame_position: Vector2,
    sidebar: Rectangle,
    sidebar_play: Button,
    save: Button,
}

impl Animation {
    pub fn new(handle: &mut RaylibHandle, thread: &RaylibThread) -> Animation {
        let sidebar = rrect(0, 30, 100, handle.get_screen_height() - 30);
        let frame_position = rvec2(sidebar.width, 30);

        let mut first_frame = Frame::new(
            handle,
            thread,
            handle.get_screen_width() as u32 - frame_position.x as u32,
            handle.get_screen_height() as u32 - 100 - frame_position.y as u32 as u32,
        );
        first_frame.is_selected = true;

        let mut figure = archives::import_raw_figure("men.raw.fig");

        figure.center_to(rvec2(
            first_frame.texture.try_borrow().ok().unwrap().width() / 2,
            first_frame.texture.try_borrow().ok().unwrap().height() / 2,
        ));

        let mut animation = Animation {
            selected_frame: 0,
            frame_scroll: 0.0,
            sidebar_play: Button::new(rvec2(sidebar.x, sidebar.y).add(rvec2(10, 10))),
            save: Button::new(rvec2(sidebar.x, sidebar.y).add(rvec2(10, 50))),
            main_texture: first_frame.texture.clone(),
            figures: vec![],
            frames: vec![first_frame],
            sidebar,
            frame_position,
        };

        animation.sidebar_play.text = Some(cstr!("Add Frame"));
        animation.save.text = Some(cstr!("Salvar"));
        animation.push_figure(figure.clone());
        animation.push_figure(figure);
        animation.update(handle, thread);
        animation.frames[0].render_screen(&mut handle.begin_drawing(thread), thread);
        animation.frames[0].render_miniature(handle, thread);
        animation
    }

    pub fn update(&mut self, mut handle: &mut RaylibHandle, thread: &RaylibThread) {
        if handle.is_key_pressed(KeyboardKey::KEY_DELETE) {
            self.remove_frame();
        }

        let frame_count = self.frames.len() as i32;
        let mut frame = &mut self.frames[self.selected_frame];

        // Update figures and animation state
        for index in 0..frame.figure_animation.len() {
            match frame.figure_animation[index].figure.try_borrow_mut() {
                Ok(mut figure) => {
                    figure.update(handle, self.frame_position);
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

        if self.sidebar_play.activated {
            self.push_frame(handle, thread);
        }

        if self.save.activated {
            self.save_raw("./src/assets/animations/unnamed.raw.anim");
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

        let main_texture = self.main_texture.try_borrow().ok().unwrap();
        // Draw main screen texture
        draw_handle.draw_texture_rec(
            main_texture.texture(),
            rrect(
                0,
                0,
                main_texture.texture.width,
                -main_texture.texture.height,
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

            self.save.activated = draw_handle.gui_button(
                rrect(
                    self.save.start.x,
                    self.save.start.y,
                    self.sidebar.width - 20.0,
                    30,
                ),
                Some(self.save.text.clone().unwrap().as_c_str()),
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

    pub fn remove_frame(&mut self) {
        self.frames.remove(self.selected_frame);
        self.selected_frame = self.frames.len() - 1;

        let mut frame = &mut self.frames[self.selected_frame];
        frame.is_selected = true;
        self.main_texture = frame.texture.clone();
        self.frame_scroll = 0.0;
    }

    /// Save animation into a file with raw format
    pub fn save_raw(&mut self, file: &str) {
        let mut file = File::create(file).ok().unwrap();

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
    }

    pub fn from_raw(file: &str, handle: &mut RaylibHandle, thread: &RaylibThread) -> Animation {
        let file = fs::read_to_string(file).ok().unwrap();
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
        let center = rvec2(
            handle.get_screen_width() / 2,
            handle.get_screen_height() / 2,
        );

        for (i, frameStr) in frames.iter().enumerate() {
            last_frame.figure_animation = vec![];

            for state in frameStr.split_once("\n").unwrap().1.split("^") {
                if state.trim().len() == 0 {
                    continue;
                }

                let mut lines = state.lines();
                let index = lines.next().unwrap().parse::<usize>().ok().unwrap();
                let mut moved_edges = HashMap::new();

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
                figure.load_state(moved_edges.clone());

                last_frame.figure_animation.push(FigureAnimation {
                    global_index: index,
                    local_index: last_frame.figure_animation.len(),
                    figure: animation.figures[index].clone(),
                    moved_edges,
                });
            }

            let texture = handle
                .load_render_texture(
                    thread,
                    last_frame.texture.try_borrow().ok().unwrap().width() as u32,
                    last_frame.texture.try_borrow().ok().unwrap().height() as u32,
                )
                .ok()
                .unwrap();

            last_frame.is_selected = false;
            last_frame.render_screen(&mut handle.begin_drawing(thread), thread);
            last_frame.render_miniature(handle, thread);
            last_frame.save_state();

            if i < frames.len() - 1 {
                let mut new_frame = last_frame.clone(texture);
                new_frame.render_screen(&mut handle.begin_drawing(thread), thread);
                new_frame.render_miniature(handle, thread);
                new_frame.save_state();

                animation.main_texture = new_frame.texture.clone();
                animation.selected_frame = animation.frames.len();
                animation.frames.push(new_frame);
                last_frame = animation.frames.last_mut().unwrap();
            }
        }

        animation
    }
}
