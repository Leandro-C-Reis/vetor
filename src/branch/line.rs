use raylib::prelude::*;

use crate::{Bone, BoneType};

use super::{point::Point, vector::{vector2_add, vector2_rotate}};

#[derive(Debug, Clone)]
pub struct LI {
    pub start: Vector2,
    pub end: Vector2,
    pub p1: Point,
    pub p2: Point,
    pub pressed_start: bool,
    pub pressed_end: bool,
    pub width: f32,
    pub angle: f32,
    pub parent: i32,
}

impl LI {
    pub fn update(&mut self, handle: &RaylibHandle, line_tree: &Vec<Bone>, mut point_pressed: &bool) -> LI {
        let mouse_pos = handle.get_mouse_position();
        self.angle = 0.0;

        if check_collision_point_circle(mouse_pos, self.end, 5.0) && !point_pressed {
            self.pressed_end = true;
            point_pressed = &true;
        }

        if check_collision_point_circle(mouse_pos, self.start, 5.0) && (!point_pressed || self.parent < 0) {
            self.pressed_start = true;
            point_pressed = &true;
        }

        if handle.is_mouse_button_down(MouseButton::MOUSE_LEFT_BUTTON) {
            if self.parent >= 0 {
                let bone = &line_tree[self.parent as usize];

                if bone.typo == BoneType::LINE {
                    let parent = bone.line.as_ref().unwrap();
                    let end = self.end;
                    let start = self.start;
                    let angle = parent.angle + end.angle_to(start);
    
                    self.start = parent.end;
                    self.end = vector2_add(vector2_rotate(self.width, angle), self.start)
                } else {
                    let parent = bone.circle.as_ref().unwrap();
                    let end = self.end;
                    let start = self.start;
                    let angle = parent.angle + end.angle_to(start);
    
                    self.start = parent.end;
                    self.end = vector2_add(vector2_rotate(self.width, angle), self.start)
                }

            }

            if self.pressed_start && self.parent < 0 {
                let angle = self.end.angle_to(self.start);
                self.start = mouse_pos;
                self.end = vector2_add(vector2_rotate(self.width, angle), self.start);
            }

            if self.pressed_end {
                let angle = mouse_pos.angle_to(self.start);
                self.angle = angle - self.end.angle_to(self.start);
                self.end = vector2_add(vector2_rotate(self.width, angle), self.start);
            }
        }

        if handle.is_mouse_button_up(MouseButton::MOUSE_LEFT_BUTTON) {
            self.pressed_end = false;
            self.pressed_start = false;
            point_pressed = &false;
        }

        self.clone()
    }
}

#[derive(Clone, Debug, Default)]
pub struct Line {
    pub start: Vector2,
    pub end: Vector2,
    pub pressed_start: bool,
    pub pressed_end: bool,
    pub width: f32,
    pub angle: f32,
    pub parent: Option<Box<Line>>
}

impl Line {
    pub fn new(start: Vector2, end: Vector2, parent: Option<Box<Line>>) -> Line {
        Line {
            start,
            end,
            parent,
            width: start.distance_to(end),
            pressed_start: false,
            pressed_end: false,
            // angle: start.angle_to(end)
            angle: 0.0
        }
    }

    pub fn boxed(start: Vector2, end: Vector2, parent: Option<Box<Line>>) -> Box<Line> {
        Box::new(Line::new(start, end, parent))
    }

    pub fn encapsulate(ptr: *mut Line) -> Option<Box<Line>> {
        Some(unsafe { Box::from_raw(ptr) })
    }
}