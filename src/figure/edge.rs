use raylib::prelude::*;

use crate::maths::*;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum EdgeFormat {
    LINE = 1,
    CIRCLE = 2
}

impl From<isize> for EdgeFormat {
    fn from(value: isize) -> Self {
        match value {
            1 => EdgeFormat::LINE,
            2 => EdgeFormat::CIRCLE,
            _ => EdgeFormat::LINE
        }
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct EdgeDrawOption {
    pub point: bool,
}

impl EdgeDrawOption {
    pub fn new() -> EdgeDrawOption {
        EdgeDrawOption {
            point: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Edge {
    pub start: Vector2,
    pub end: Vector2,
    pub pressed_start: bool,
    pub pressed_end: bool,
    pub width: f32,
    pub moved: bool,
    pub fixed_angle: f32,
    pub moved_angle: f32,
    pub parent: isize,
    pub format: EdgeFormat,
}

impl Edge {
    pub fn new(start: Vector2, end: Vector2, parent: isize, typ: isize) -> Edge {
        Edge {
            start,
            end,
            parent,
            format: EdgeFormat::from(typ),
            pressed_start: false,
            pressed_end: false,
            moved: false,
            moved_angle: 0.0,
            fixed_angle: end.angle_to(start),
            width: start.distance_to(end),
        }
    }

    /// Calculate real end position rotating on fixed angle
    /// with current width and then sum with start vector.
    pub fn get_real_end(&self) -> Vector2 {
        vector2_rotate(self.width, self.fixed_angle).add(self.start)
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
            && self.parent == -1 && (!*point_pressed || *pressed_root)
        {
            self.pressed_start = true;
            *point_pressed = true;
            *pressed_root = true;
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
                self.end = vector2_rotate(self.width, angle).add(self.start);
            }

            if self.pressed_start && self.parent < 0 {
                let angle = self.end.angle_to(self.start);
                self.start = mouse_pos;
                self.end = vector2_rotate(self.width, angle).add(self.start);
            }

            if self.pressed_end {
                let angle = mouse_pos.angle_to(self.start);
                self.moved = true;
                self.end = vector2_rotate(self.width, angle).add(self.start);
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

    pub fn draw(&self, draw_handle: &mut RaylibTextureMode<RaylibDrawHandle>, option: EdgeDrawOption) {
        match self.format {
            EdgeFormat::LINE => {
                let radian = self.start.angle_to(self.end);
                let rotation = radian * 180.0 / PI as f32;
                let distance = self.start.distance_to(self.end);
        
                let rect = Rectangle {
                    x: self.start.x as f32,
                    y: self.start.y as f32,
                    width: distance,
                    height: 20.0,
                };
        
                draw_handle.draw_rectangle_pro(rect, Vector2 { x:0 as f32, y: 10 as f32 }, rotation, Color::BLACK);
                draw_handle.draw_circle_v(self.start, 10.0, Color::BLACK);
                draw_handle.draw_circle_v(self.end, 10.0, Color::BLACK);
            },
            EdgeFormat::CIRCLE => {
                let radius = self.width / 2.0;
                let center = vector2_rotate(radius, self.start.angle_to(self.end)).add(self.end);

                let thickness = 20.0;

                draw_handle.draw_ring(
                    Vector2 { x: center.x, y: center.y},
                    radius - (thickness / 2.0),
                    radius + (thickness / 2.0),
                    0.0,
                    360.0,
                    0,
                    Color::BLACK
                );
            }
        }
    }

    /// Draw edge points
    pub fn draw_points(&self, draw_handle: &mut RaylibTextureMode<RaylibDrawHandle>) {
        let mut root_point_color = Color::RED;
        
        if self.parent == -1 {
            root_point_color = Color::ORANGE;
        }

        draw_handle.draw_circle_v(self.start, 5.0, root_point_color);
        draw_handle.draw_circle_v(self.end, 5.0, Color::RED);
    }
}