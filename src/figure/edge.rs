use raylib::prelude::*;

pub enum EdgeType {
    LINE = 1,
    CIRCLE = 2
}

#[derive(Debug, Clone)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub typ: isize,
    pub parent: usize,
    pub index: usize,
    pub chidren: Vec<usize>
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub start: Vector2,
    pub end: Vector2,
    pub p1: Point,
    pub p2: Point,
    pub pressed_start: bool,
    pub pressed_end: bool,
    pub width: f32,
    pub moved: bool,
    pub fixed_angle: f32,
    pub moved_angle: f32,
    pub parent: isize,
    pub typ: isize,
}

impl Edge {
    pub fn new(p1: Point, p2: Point, parent: isize, typ: isize) -> Edge {
        let start = Vector2::new(p1.x, p1.y);
        let end = Vector2::new(p2.x, p2.y);
        Edge {
            p1,
            p2,
            start,
            end,
            parent,
            typ,
            pressed_start: false,
            pressed_end: false,
            moved: false,
            moved_angle: 0.0,
            fixed_angle: end.angle_to(start),
            width: start.distance_to(end),
        }
    }

    pub fn update(&mut self, handle: &RaylibHandle, line_tree: &Vec<Edge>, point_pressed: &mut bool, pressed_root: &mut bool) -> Edge {
        let mouse_pos = handle.get_mouse_position();
        self.moved = false;
        self.moved_angle = 0.0;

        if check_collision_point_circle(mouse_pos, self.end, 5.0) && !*point_pressed {
            self.pressed_end = true;
            *point_pressed = true;
        }

        // Check if point is collided and if root point is pressed
        if check_collision_point_circle(mouse_pos, self.start, 5.0) 
            && !*point_pressed 
            || self.parent == -1 
            && *pressed_root 
        {
            self.pressed_start = true;
            *point_pressed = true;

            if self.parent == -1 {
                *pressed_root = true;
            }
        }

        if handle.is_mouse_button_down(MouseButton::MOUSE_LEFT_BUTTON) {
            if self.parent >= 0 {
                let parent = &line_tree[self.parent as usize];

                let end = self.end;
                let start = self.start;
                let parent_angle = parent.end.angle_to(parent.start);
                
                // Get current static angle or rotate with parent.
                let angle = if parent.moved {
                    self.moved = true;
                    parent_angle - parent.fixed_angle + self.fixed_angle
                } else { 
                    end.angle_to(start)
                };

                self.start = parent.end;
                self.end = vector2_add(vector2_rotate(self.width, angle), self.start);
            }

            if self.pressed_start && self.parent < 0 {
                let angle = self.end.angle_to(self.start);
                self.start = mouse_pos;
                self.end = vector2_add(vector2_rotate(self.width, angle), self.start);
            }

            if self.pressed_end {
                let angle = mouse_pos.angle_to(self.start);
                self.moved = true;
                self.end = vector2_add(vector2_rotate(self.width, angle), self.start);
            }
        }

        // Clear pressed variables when mouse is not pressed anymore
        if handle.is_mouse_button_up(MouseButton::MOUSE_LEFT_BUTTON) {
            // Caculate a diference of the rotated angles.
            if self.pressed_end {
                let angle = self.fixed_angle;
                self.fixed_angle = self.end.angle_to(self.start);
                self.moved_angle = self.fixed_angle - angle;
            }

            // Recalculate rotated angles to children.
            if self.parent >= 0 {
                let parent = &line_tree[self.parent as usize];

                if parent.moved_angle != 0.0 {
                    let angle = self.fixed_angle;
                    self.fixed_angle = parent.moved_angle + self.fixed_angle;
                    self.moved_angle = self.fixed_angle - angle;
                }
            }

            self.pressed_end = false;
            self.pressed_start = false;
            *point_pressed = false;
            *pressed_root = false;
        }

        self.clone()
    }
}

pub fn vector2_rotate(length: f32, angle: f32)  -> Vector2 {
    let cs = (angle.cos() * 100.0).round() / 100.0;
    let sn = (angle.sin() * 100.0).round() / 100.0;

    // Multiply by -1 because coordinate rotation is reversed.
    let x = (-1.0 * length * cs).round();
    let y = (-1.0 * length * sn).round();
    
    Vector2::new(x, y)
}

pub fn vector2_subtract(v1: Vector2, v2: Vector2) -> Vector2 {
    Vector2::new(v1.x - v2.x, v1.y - v2.y)
}

pub fn vector2_add(v1: Vector2, v2: Vector2) -> Vector2 {
    Vector2::new(v1.x + v2.x, v1.y + v2.y)
}