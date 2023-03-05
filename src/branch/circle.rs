use raylib::{prelude::{Vector2, MouseButton}, check_collision_point_circle, RaylibHandle};
use crate::{branch::point::Point, Bone, BoneType};

use super::vector::{vector2_add, vector2_rotate};

#[derive(Clone, Debug)]
pub struct Circle {
    pub start: Vector2,
    pub end: Vector2,
    pub p1: Point,
    pub p2: Point,
    pub center: Vector2,
    pub pressed_start: bool,
    pub pressed_end: bool,
    pub radius: f32,
    // pub width: f32,
    pub angle: f32,
    pub parent: usize,   
}

impl Circle {
    pub fn update(&mut self, handle: &RaylibHandle, line_tree: &Vec<Bone>, mut point_pressed: &bool) -> Circle {
        let mouse_pos = handle.get_mouse_position();
        
        if check_collision_point_circle(mouse_pos, self.end, 5.0) && !point_pressed {
            self.pressed_end = true;
            point_pressed = &true;
        }

        if check_collision_point_circle(mouse_pos, self.start, 5.0) && !point_pressed {
            self.pressed_start = true;
            point_pressed = &true;
        }
        
        if handle.is_mouse_button_down(MouseButton::MOUSE_LEFT_BUTTON) {
            let distance = self.start.distance_to(self.end);

            if self.parent >= 0 {
                let bone = &line_tree[self.parent as usize];

                if bone.typo == BoneType::LINE {
                    let parent = bone.line.as_ref().unwrap();
                    let end = self.end;
                    let start = self.start;
                    let angle = parent.angle + end.angle_to(start);
    
                    self.start = parent.end;
                    self.end = vector2_add(vector2_rotate(distance, angle), self.start);
                    self.center = vector2_add(vector2_rotate(self.radius, start.angle_to(end)), end)
                } else {
                    let parent = bone.circle.as_ref().unwrap();
                    let end = self.end;
                    let start = self.start;
                    let angle = parent.angle + end.angle_to(start);
    
                    self.start = parent.end;
                    self.end = vector2_add(vector2_rotate(distance, angle), self.start);
                }
            }

            if self.pressed_start && self.parent < 0 {
                let angle = self.end.angle_to(self.start);
                self.start = mouse_pos;
                self.end = vector2_add(vector2_rotate(distance, angle), self.start);
            }

            if self.pressed_end {
                let angle = mouse_pos.angle_to(self.start);
                self.angle = angle - self.end.angle_to(self.start);
                self.end = vector2_add(vector2_rotate(distance, angle), self.start);
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