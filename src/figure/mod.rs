pub mod edge;
use raylib::{RaylibHandle, prelude::RaylibDrawHandle};

use self::edge::Edge;

pub struct Figure {
    tree: Vec<Edge>
}

impl Figure {
    pub fn new(tree: Vec<Edge>) -> Figure {
        Figure {
            tree
        }
    }

    pub fn update(&mut self, handle: &RaylibHandle, point_pressed: &mut bool, pressed_root: &mut bool) {
        for i in 0..self.tree.len() {
            let mut edge: Edge = self.tree[i].clone();
            self.tree[i] = edge.update(&handle, &self.tree, point_pressed, pressed_root);
        }
    }

    pub fn draw(&self, draw_handle: &mut RaylibDrawHandle) {
        for edge in self.tree.iter() {
            edge.draw(draw_handle);
        }
    }
}