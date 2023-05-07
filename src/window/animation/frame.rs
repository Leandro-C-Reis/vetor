use raylib::{ffi::LoadRenderTexture, prelude::*, RaylibHandle, RaylibThread};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::figure::Figure;

#[derive(Debug, Clone, PartialEq)]
pub struct FigureAnimation {
    pub global_index: usize,
    pub local_index: usize,
    pub moved_edges: HashMap<usize, (Vector2, Vector2)>,
    pub figure: Rc<RefCell<Figure>>,
}

#[derive(Debug)]
pub struct Frame {
    pub texture: Rc<RefCell<RenderTexture2D>>,
    pub miniature: Option<Texture2D>,
    pub figure_animation: Vec<FigureAnimation>,
    pub is_selected: bool,
}

impl Frame {
    pub fn new(handle: &mut RaylibHandle, thread: &RaylibThread, width: u32, height: u32) -> Frame {
        Frame {
            figure_animation: vec![],
            is_selected: false,
            miniature: None,
            texture: Rc::new(RefCell::new(
                handle
                    .load_render_texture(thread, width, height)
                    .ok()
                    .unwrap(),
            )),
        }
    }

    pub fn clone(&mut self, texture: RenderTexture2D) -> Frame {
        Frame {
            is_selected: self.is_selected,
            figure_animation: self.figure_animation.clone(),
            miniature: None,
            texture: Rc::new(RefCell::new(texture)),
        }
    }

    pub fn disable_except(&mut self, index: usize) {
        for i in 0..self.figure_animation.len() {
            if i == index {
                continue;
            }

            self.figure_animation[i].figure.borrow_mut().should_update = false;
        }
    }

    pub fn enable_all(&mut self) {
        for animation in &mut self.figure_animation {
            animation.figure.borrow_mut().should_update = true;
        }
    }

    // Render screen texture
    pub fn render_screen(&mut self, draw_handle: &mut RaylibDrawHandle, thread: &RaylibThread) {
        let mut texture = self.texture.borrow_mut();
        let mut draw_texture = draw_handle.begin_texture_mode(thread, &mut texture);

        // Draw figures on texture
        for animation in &mut self.figure_animation {
            animation.figure.borrow_mut().draw(&mut draw_texture);
        }
    }

    /// Scan and resize screen texture to miniature texture
    pub fn render_miniature(&mut self, mut handle: &mut RaylibHandle, thread: &RaylibThread) {
        let mut image = self.texture.borrow().texture().load_image().unwrap();
        image.flip_vertical();
        image.resize(150, 100);

        self.miniature = Some(handle.load_texture_from_image(thread, &image).ok().unwrap());
    }

    pub fn save_state(&mut self) {
        for anim in &mut self.figure_animation {
            anim.moved_edges = anim.figure.borrow().scan();
        }
    }
}
