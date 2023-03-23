use std::{fs};

use crate::{figure::edge::*};

pub fn import_from_raw(file: &str) -> Vec<Point> {
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

        points.push(Point { x,y,parent, index, chidren: vec![], typ });
    }

    points
}