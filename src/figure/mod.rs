pub mod edge;
use self::edge::{Edge, EdgeDrawOption, EdgeFormat};
use crate::{log, window};
use raylib::{
    prelude::{RaylibRenderTexture2D, *},
    texture::RenderTexture2D,
    RaylibHandle, RaylibThread,
};
use std::{cmp::Ordering, collections::HashMap, env, ops::Index};

#[derive(Debug, Copy, Clone, PartialEq)]
enum FigMode {
    CONSTRUCTOR = 1,
    ANIMATION = 2,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Figure {
    tree: Vec<Edge>,
    mode: FigMode,
    pub should_update: bool,
    pub draw_option: EdgeDrawOption,
    pub selected: Option<usize>,
    pub tmp_edge: Option<Edge>,
    pub pressed: bool,
    pub presset_root: bool,
}

impl Figure {
    pub fn new(tree: Vec<Edge>) -> Figure {
        let mut figure = Figure {
            tree,
            presset_root: false,
            selected: None,
            mode: FigMode::CONSTRUCTOR,
            draw_option: EdgeDrawOption::new(),
            tmp_edge: None,
            pressed: false,
            should_update: true,
        };
        figure.sort();
        figure
    }

    // 1. === Update and Draw ===
    pub fn update(&mut self, handle: &RaylibHandle, start_position: Vector2) {
        self.selected = None;

        if self.should_update {
            for i in 0..self.tree.len() {
                let mut edge: Edge = self.tree[i].clone();

                let pressed_before = self.pressed;

                self.tree[i] = edge.update(
                    handle,
                    &self.tree,
                    &mut self.pressed,
                    &mut self.presset_root,
                    start_position,
                );

                if pressed_before != self.pressed {
                    self.selected = Some(i);
                }
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

        if self.draw_option.point {
            for edge in self.tree.iter() {
                edge.draw_points(draw_texture);
            }
        }
    }

    // 2. === Helper functions ===
    pub fn get_children(&self, index: isize) -> Vec<usize> {
        self.tree
            .to_vec()
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

        self.tree = indexed
            .iter()
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

    /// Recursive function that re-map and organize the tree in vector space.
    /// * TODO: I think the children function can be optimized with something like
    /// a Rc<RefCell<Edge>> and mantain a vector view of the edges. Will cost much time
    /// to refactor.
    ///
    fn indexed_tree(&self, index: isize) -> Vec<usize> {
        let mut children = self.get_children(index);
        let mut sector = children.to_vec();

        children.reverse();
        for child in children {
            sector = [sector, self.indexed_tree(child as isize)].concat();
        }

        sector
    }

    /// Get edge at given index
    pub fn get(&self, index: usize) -> &Edge {
        &self.tree[index]
    }

    /// Get mutable edge at given index
    pub fn get_mut(&mut self, index: usize) -> &mut Edge {
        &mut self.tree[index]
    }

    /// Unselect parent and children edges
    pub fn clear_edge_and_children(&mut self, index: usize) {
        let parent = self.get(index).parent;
        self.get_mut(index).pressed_start = false;
        self.get_mut(index).pressed_end = false;
        self.get_mut(index).moved_angle = 0.0;

        for index in self.get_children(parent) {
            let child = self.get_mut(index);
            child.pressed_start = false;
            child.pressed_end = false;
        }
    }

    /// Copy edge to temporary space
    pub fn copy_tmp(&mut self, index: usize) {
        let mut edge = self.tree[index];
        let start = edge.start;
        edge.start = edge.end;
        edge.end = start;
        edge.update_angle();
        edge.moved_angle = 0.0;

        self.tmp_edge = Some(edge);
    }

    pub fn scan(&self) -> HashMap<usize, (Vector2, Vector2)> {
        let mut compare_map = HashMap::new();

        for (index, edge) in self.tree.iter().enumerate() {
            compare_map.insert(index, (edge.start, edge.end));
        }

        compare_map
    }

    pub fn load_state(&mut self, diff: HashMap<usize, (Vector2, Vector2)>) {
        for (index, vertex) in diff.iter() {
            let mut edge = &mut self.tree[*index];
            edge.start = vertex.0;
            edge.end = vertex.1;
            edge.update_angle();
        }
    }

    // 3. === Controllers ===
    pub fn toggle_type(&mut self, index: usize) {
        match self.tree[index].format {
            EdgeFormat::CIRCLE => {
                self.tree[index].format = EdgeFormat::LINE;
            }
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

            let parent = Edge::new(start, grandfather.end, index as isize, 1);

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

    pub fn insert(&mut self, edge: Edge) {
        self.tree.push(edge);
        self.sort();
    }

    /// Delete edge on given index and update edges indexing parents
    pub fn delete(&mut self, index: usize) {
        println!("------ Deleting edge on index: {} ------", index);
        let edge = self.tree[index];

        log!("Index, Parent: {} , {}", index, edge.parent);
        log!("Children: {:?}", self.get_children(index as isize));

        for child in self.get_children(index as isize) {
            self.tree[child].parent = edge.parent;

            if edge.parent == -1 {
                let brother = self.tree.iter().find(|e| e.parent == -1).unwrap();

                self.tree[child].start = brother.start;
                self.tree[child].end = self.tree[child].get_real_end();
            }
        }

        log!("Len: {}", self.tree.len());
        log!("Tree: {:?}", self.indexed_tree(-1));

        let mut changed_indexes = HashMap::new();

        for i in index..self.tree.len() {
            changed_indexes.insert(i, i as isize - 1);

            match changed_indexes.get(&i) {
                Some(p) => {
                    log!("Changed index: {} => {}", i, p);
                    for child in self.get_children(i as isize) {
                        self.tree[child].parent = *p;
                    }
                }
                _ => (),
            }
        }

        self.tree.remove(index);

        log!("Len: {}", self.tree.len());
        log!("Tree: {:?}", self.indexed_tree(-1));
        log!("Changed Indexes: {:?}", changed_indexes);

        println!("------ Edge {} removed ------", index);
    }
}
