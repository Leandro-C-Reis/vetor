use std::f32::consts::PI;

use raylib::{prelude::{RaylibDraw, Color}, ffi::{Rectangle, Vector2}};

use crate::LI;

use self::{line::Line, circle::Circle};

pub mod line;
pub mod node;
pub mod vector;
pub mod point;
pub mod circle;

pub trait Draw: RaylibDraw {
    fn drawln(&mut self, line: &Line) {
        let radian = line.start.angle_to(line.end);
        let rotation = radian * 180.0 / PI as f32;
        let distance = line.start.distance_to(line.end);

        let rect = Rectangle {
            x: line.start.x as f32,
            y: line.start.y as f32,
            width: distance,
            height: 20.0,
        };

        
        self.draw_rectangle_pro(rect, Vector2 { x:0 as f32, y: 10 as f32 }, rotation, Color::BLACK);
        self.draw_circle_v(line.start, 10.0, Color::BLACK);
        self.draw_circle_v(line.end, 10.0, Color::BLACK);
        self.draw_circle_v(line.start, 5.0, Color::ORANGE);
        self.draw_circle_v(line.end, 5.0, Color::ORANGE);
    }

    fn drawli(&mut self, line: &LI) {
        let radian = line.start.angle_to(line.end);
        let rotation = radian * 180.0 / PI as f32;
        let distance = line.start.distance_to(line.end);

        let rect = Rectangle {
            x: line.start.x as f32,
            y: line.start.y as f32,
            width: distance,
            height: 20.0,
        };

        self.draw_rectangle_pro(rect, Vector2 { x:0 as f32, y: 10 as f32 }, rotation, Color::BLACK);
        self.draw_circle_v(line.start, 10.0, Color::BLACK);
        self.draw_circle_v(line.end, 10.0, Color::BLACK);
        self.draw_circle_v(line.start, 5.0, Color::ORANGE);
        self.draw_circle_v(line.end, 5.0, Color::ORANGE);
    }

    fn drawci(&mut self, circle: &Circle) {
        self.draw_ring(
            Vector2 {x:circle.center.x,y:circle.center.y},
            30.0,
            50.0,
            0.0,
            360.0,
            0,
            Color::BLACK);
        self.draw_circle_v(circle.start, 5.0, Color::ORANGE);
        self.draw_circle_v(circle.end, 5.0, Color::ORANGE);
    }

}

impl<T> Draw for T where T: RaylibDraw {}