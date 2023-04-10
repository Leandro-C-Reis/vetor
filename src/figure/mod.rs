#[allow(adts)]

pub mod edge;

use std::{ops::Index, cmp::Ordering, collections::HashMap};
use raylib::{RaylibHandle, prelude::{*, RaylibRenderTexture2D},texture::RenderTexture2D, RaylibThread};
use crate::window::Tab;
use self::edge::{Edge, EdgeFormat, EdgeDrawOption};

#[derive(Copy, Clone, PartialEq)]
enum FigMode {
    CONSTRUCTOR = 1,
    ANIMATION = 2
}

pub struct Figure {
    tree: Vec<Edge>,
    presset_root: bool,
    mode: FigMode,
    draw_option: EdgeDrawOption,
    pub selected: Option<usize>,
    pub tmp_edge: Option<Edge>,
    pub pressed: bool,
}

impl Figure {
    pub fn new(tree: Vec<Edge>) -> Figure {
        Figure {
            tree,
            presset_root: false,
            selected: None,
            mode: FigMode::CONSTRUCTOR,
            draw_option: EdgeDrawOption::new(),
            tmp_edge: None,
            pressed: false
        }
    }

    pub fn update(&mut self, handle: &RaylibHandle) {
        self.selected = None;

        if self.tmp_edge.is_none() {
            for i in 0..self.tree.len() {
                let mut edge: Edge = self.tree[i].clone();
    
                let pressed_before = self.pressed;
    
                self.tree[i] = edge.update(handle, &self.tree, &mut self.pressed, &mut self.presset_root);
    
                if pressed_before != self.pressed {
                    self.selected = Some(i);
                }
            }
        } else {
            let tmp = self.tmp_edge.as_mut().unwrap();
            tmp.end = handle.get_mouse_position();
            tmp.width = tmp.start.distance_to(tmp.end);
            tmp.fixed_angle = tmp.end.angle_to(tmp.start);

            if handle.is_mouse_button_pressed(MouseButton::MOUSE_LEFT_BUTTON) {
                self.tree.push(tmp.clone());
                self.tree[tmp.parent as usize].pressed_end = false;
                self.tmp_edge = None;
                self.sort();
            }
        }
    }

    pub fn draw(&mut self, draw_texture: &mut RaylibTextureMode<RaylibDrawHandle>) {
        draw_texture.clear_background(Color::RAYWHITE);
        for edge in self.tree.iter() {
            match self.mode {
                FigMode::ANIMATION => {
                    edge.draw(draw_texture, self.draw_option);
                }
                FigMode::CONSTRUCTOR => {
                    edge.draw(draw_texture, self.draw_option);
                }
            }
        }

        if self.mode == FigMode::CONSTRUCTOR {
            if self.tmp_edge.is_some() {
                self.tmp_edge.unwrap().draw(draw_texture, self.draw_option);
            }
        }
    }

    fn get_children(&self, index: isize) -> Vec<usize> {
        self.tree.to_vec()
            .into_iter()
                .enumerate()
                .filter(|(_, e)| e.parent == index)
                .map(|(i, _)| i)
            .collect::<Vec<usize>>()
    }
    fn sort(&mut self) {
        // Start from -1 as root parent to search.
        let indexed = self.indexed_tree(-1);
        let mut changed_indexes = HashMap::new();

        self.tree = indexed.iter()
            .enumerate()
            .map(|(i, e)| {
                let mut edge = self.tree[*e];
                
                if *e != i {
                    changed_indexes.insert(*e, i as isize);
                }

                let parent = changed_indexes.get(&(edge.parent as usize));

                // Update parent index 
                if parent.is_some() {
                    edge.parent = *parent.unwrap();
                }

                edge
            })
        .collect();
    }
    fn indexed_tree(&self, index: isize) -> Vec<usize> {
        let mut children = self.get_children(index);
        let mut sector = children.to_vec();

        children.reverse();
        for child in children {
            sector = [sector, self.indexed_tree(child as isize)].concat();
        }

        sector
    }

    pub fn toggle_type(&mut self, index: usize) {
        match self.tree[index].format {
            EdgeFormat::CIRCLE => {
                self.tree[index].format = EdgeFormat::LINE;
            },
            EdgeFormat::LINE => {
                self.tree[index].format = EdgeFormat::CIRCLE;
            }
        };
    }

    pub fn divide(&mut self, index: usize) {
        let grandfather = self.tree[index];

        if grandfather.format == EdgeFormat::LINE {
            let children = self.get_children(index as isize);
            
            let start = grandfather.start.lerp(grandfather.end, 0.5);

            let parent = Edge::new(
                start,
                grandfather.end,
                index as isize,
                1
            );

            let idx = self.tree.len();
            self.tree.push(parent);

            // Update grandfather
            self.tree[index].end = start;
            self.tree[index].width = grandfather.start.distance_to(start);
            
            // Update cildren
            for child in children {
                self.tree[child].parent = idx as isize;
            }

            self.sort();
        }
    }

    pub fn insert(&mut self, index: usize) {
        let edge = self.tree[index];

        if self.tmp_edge.is_none() {
            self.tmp_edge = Some(
                Edge::new(edge.end, edge.end, index as isize, 1)
            );
        }
    }
}