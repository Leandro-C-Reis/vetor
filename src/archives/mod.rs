use crate::{
    figure::{edge::*, Figure},
    window::animation::Animation,
};
use raylib::prelude::{rvec2, Vector2};
use std::{cmp::Ordering, collections::HashMap, fs, io::Write};

#[derive(Clone, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub typ: isize,
    pub parent: usize,
    pub index: usize,
}

pub fn import_raw_figure(file: &str) -> Figure {
    let file =
        fs::read_to_string(file).expect(&format!("Should be able to read the file: {}", file));
    raw_to_figure(file.as_str())
}

pub fn export_raw_figure(file: &str, mut figure: Figure) {
    let mut file = fs::File::create("./src/assets/figures/".to_owned() + file)
        .ok()
        .unwrap();

    let mut pts = figure_to_raw(figure);
    for point in pts {
        file.write(
            format!(
                "{},{},{},{},{}\n",
                point.typ, point.x, point.y, point.parent, point.index
            )
            .as_bytes(),
        )
        .expect("Cannot write points to file");
    }
}

pub fn raw_to_figure(raw: &str) -> Figure {
    let mut points: Vec<Point> = Vec::new();

    for row in raw.split('\n').collect::<Vec<_>>() {
        if row.starts_with("//") {
            continue;
        };
        if row.starts_with("#") {
            continue;
        };

        let edge = row.split(',').collect::<Vec<_>>();
        if edge.len() < 5 {
            continue;
        };

        let typ = edge[0]
            .parse::<isize>()
            .expect("x must be a numeric int unsigned 8");
        let x = edge[1]
            .parse::<f32>()
            .expect("x must be a numeric float 32");
        let y = edge[2]
            .parse::<f32>()
            .expect("y must be a numeric float 32");
        let parent = edge[3]
            .parse::<usize>()
            .expect("parent must be a numeric int 32");
        let index = edge[4]
            .parse::<usize>()
            .expect("index must be a numeric int 32");

        points.push(Point {
            x,
            y,
            parent,
            index,
            typ,
        });
    }

    let mut figure_tree: Vec<Edge> = vec![];
    let mut indexes: Vec<usize> = vec![];

    for i in 0..points.len() {
        let point = points[i].clone();

        if point.index == 0 {
            continue;
        }

        let p1 = points[point.parent].clone();
        let parent = indexes
            .iter()
            .position(|index| *index == point.parent)
            .map(|x| x as isize);

        indexes.push(point.index);
        figure_tree.push(Edge::new(
            Vector2::new(p1.x, p1.y),
            Vector2::new(point.x, point.y),
            parent.unwrap_or_else(|| -1),
            point.typ,
        ));
    }

    Figure::new(figure_tree)
}

pub fn figure_to_raw(mut figure: Figure) -> Vec<Point> {
    figure.center_to(rvec2(0, 0));
    let mut points = HashMap::new();
    let mut indexes = HashMap::new();

    points.insert(
        0,
        Point {
            x: figure.tree[0].start.x,
            y: figure.tree[0].start.y,
            typ: figure.tree[0].format.into(),
            parent: 0,
            index: 0,
        },
    );

    for (i, edge) in figure.tree.iter().enumerate() {
        let index = points.len();
        indexes.insert(i, index);

        points.insert(
            index,
            Point {
                x: edge.end.x,
                y: edge.end.y,
                typ: edge.format.into(),
                parent: if edge.parent == -1 {
                    0
                } else {
                    *indexes.get(&(edge.parent as usize)).unwrap()
                },
                index,
            },
        );
    }

    let mut pts: Vec<Point> = points.values().map(|x| x.clone()).collect();

    pts.sort_by(|p1, p2| {
        let diff = p1.index as i32 - p2.index as i32;

        if diff == 0 {
            return Ordering::Equal;
        } else if diff < 0 {
            return Ordering::Less;
        }

        Ordering::Greater
    });

    pts
}
