use std::{fs};

use raylib::prelude::Vector2;

use crate::{figure::{edge::*, Figure}};

#[derive(Clone)]
struct Point {
    pub x: f32,
    pub y: f32,
    pub typ: isize,
    pub parent: usize,
    pub index: usize,
}

pub fn import_from_raw(file: &str, center: Vector2) -> Figure {
    let file = fs::read_to_string(file).expect(&format!("Should be able to read the file: {}", file));
    let mut points: Vec<Point> = Vec::new();

    for row in file.split('\n').collect::<Vec<_>>() {
        if row.starts_with("//") {continue};

        let edge = row.split(',').collect::<Vec<_>>();
        let typ = edge[0].parse::<isize>().expect("x must be a numeric int unsigned 8");
        let x = edge[1].parse::<f32>().expect("x must be a numeric float 32");
        let y = edge[2].parse::<f32>().expect("y must be a numeric float 32");
        let parent = edge[3].parse::<usize>().expect("parent must be a numeric int 32");
        let index = edge[4].parse::<usize>().expect("index must be a numeric int 32");

        points.push(Point { x: x + center.x, y: y + center.y, parent, index, typ });
    }

    let mut figure_tree: Vec<Edge> = vec![];
    let mut indexes: Vec<usize> = vec![];
    
    for i in 0..points.len() {
        let point = points[i].clone();

        if point.index == 0 {
            continue;
        }

        let p1 = points[point.parent].clone();
        let parent = indexes.iter()
            .position(|index| *index == point.parent)
            .map(|x| x as isize);
        
        indexes.push(point.index);
        figure_tree.push(Edge::new(Vector2::new(p1.x, p1.y), Vector2::new(point.x, point.y), parent.unwrap_or_else(|| -1), point.typ));
    }

    Figure::new(figure_tree)
}