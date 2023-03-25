use std::{fs};

use raylib::prelude::Vector2;

use crate::{figure::{edge::*, Figure}};

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

        match points.get_mut(parent) {
            Some(point) => point.chidren.push(index),
            None => ()
        }

        points.push(Point { x: x + center.x, y: y + center.y, parent, index, chidren: vec![], typ });
    }

    let mut figure_tree: Vec<Edge> = vec![];

    for i in 0..points.len() {
        let point = points[i].clone();

        if point.index == 0 {
            continue;
        }

        let p1 = points[point.parent].clone();
        let parent = figure_tree.iter()
            .position(|x| x.p2.index == point.parent)
            .map(|x| x as isize);
            
        figure_tree.push(Edge::new(p1, point.clone(), parent.unwrap_or_else(|| -1), point.typ));
    }

    Figure::new(figure_tree)
}