use std::f32::consts::PI;
use raylib::{prelude::{RaylibDraw, Color, Vector2}, ffi::{Rectangle}};

use self::edge::*;

pub mod edge;

pub fn create_figure(points: Vec<Point>) -> Vec<Edge> {
    let mut line_tree: Vec<Edge> = vec![];

    for i in 0..points.len() {
        let point = points[i].clone();

        if point.index == 0 {
            continue;
        }

        let p1 = points[point.parent].clone();
        let parent = line_tree.iter()
            .position(|x| x.p2.index == point.parent)
            .map(|x| x as isize);
            
        line_tree.push(Edge::new(p1, point.clone(), parent.unwrap_or_else(|| -1), point.typ));
    }

    line_tree
}


pub trait Draw: RaylibDraw {
    fn drawli(&mut self, line: &Edge) {
        let radian = line.start.angle_to(line.end);
        let rotation = radian * 180.0 / PI as f32;
        let distance = line.start.distance_to(line.end);

        let rect = Rectangle {
            x: line.start.x as f32,
            y: line.start.y as f32,
            width: distance,
            height: 20.0,
        };

        let mut point_color = Color::RED;

        if line.parent == -1 {
            point_color = Color::ORANGE;
        }

        self.draw_rectangle_pro(rect, Vector2 { x:0 as f32, y: 10 as f32 }, rotation, Color::BLACK);
        self.draw_circle_v(line.start, 10.0, Color::BLACK);
        self.draw_circle_v(line.end, 10.0, Color::BLACK);
        self.draw_circle_v(line.start, 5.0, point_color);
        self.draw_circle_v(line.end, 5.0, point_color);
    }

    fn drawci(&mut self, circle: &Edge) {
        let radius = circle.width / 2.0;
        let center = vector2_add(vector2_rotate(radius, circle.start.angle_to(circle.end)), circle.end);

        self.draw_ring(
            Vector2 { x: center.x, y: center.y},
            30.0,
            50.0,
            0.0,
            360.0,
            0,
            Color::BLACK);
        self.draw_circle_v(circle.start, 5.0, Color::RED);
        self.draw_circle_v(circle.end, 5.0, Color::RED);
    }

}

impl<T> Draw for T where T: RaylibDraw {}