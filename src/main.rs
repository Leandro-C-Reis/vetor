
mod branch;
mod imports;

use branch::{line::*, vector::*, node::*, Draw, point::Point, circle::Circle};
use raylib::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum BoneType {
    LINE = 1 as isize,
    CIRCLE = 2
}

#[derive(Debug, Clone)]
pub struct Bone {
    pub typo: BoneType,
    pub circle: Option<Circle>,
    pub line: Option<LI>
}

fn main() {
    let screen_width = 1200;
    let screen_height = 700;
    let center_x = screen_width / 2;
    let center_y = screen_height / 2;

    let (mut handle, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("Pivot animator")
        .build();
    
    let mut point_pressed = false;

    let points = imports::bin::import_from_raw("men.vec");
    let mut line_tree: Vec<Bone> = vec![]; 

    for i in 0..points.len() {
        let point = points[i].clone();

        if point.index == 0 {
            continue;
        }

        let p1 = points[point.parent].clone();
        let start = Vector2 { x: center_x as f32 + p1.x, y: center_y as f32 + p1.y };
        let end = Vector2 { x: center_x as f32 + point.x, y: center_y as f32 + point.y };
        let parent = line_tree.iter()
            .position(|x| {
                if x.typo == BoneType::LINE {
                    x.line.as_ref().unwrap().p2.index == point.parent
                } else {
                    x.circle.as_ref().unwrap().p2.index == point.parent
                }
            })
            .map(|x| x as i32);
        
        if point.typo == BoneType::LINE as u8 {
            let line = LI {
                p1,
                p2: point,
                start,
                end,
                angle: start.angle_to(end),
                width: start.distance_to(end),
                pressed_start: false,
                pressed_end: false,
                parent: parent.unwrap_or_else(|| -1),
            };

            line_tree.push(Bone {
                typo: BoneType::LINE,
                line: Some(line),
                circle: None,
            });

            continue;
        }

        let radius = start.distance_to(end) / 2.0;

        line_tree.push(Bone {
            typo: BoneType::CIRCLE,
            line: None,
            circle: Some(Circle {
                p1,
                p2: point,
                start,
                end,
                angle: start.angle_to(end),
                pressed_start: false,
                pressed_end: false,
                parent: parent.unwrap() as usize,
                radius: start.distance_to(end) / 2.0,
                center: vector2_add(vector2_rotate(radius, start.angle_to(end)), end)
            }),
        });
    }

    handle.set_target_fps(60);
    while !handle.window_should_close() {
        // Update
        // ----------

        for i in 0..line_tree.len() {
            let mut bone: Bone = line_tree[i].clone();
            
            if bone.typo == BoneType::LINE {
                let mut line = bone.line.unwrap();
                line_tree[i].line = Some(line.update(&handle, &line_tree, &point_pressed));
            } else {
                let mut circle = bone.circle.unwrap();
                line_tree[i].circle = Some(circle.update(&handle, &line_tree, &point_pressed));
            }
        }
        
        // Draw
        let mut window = handle.begin_drawing(&thread);

        window.clear_background(Color::RAYWHITE);
        
        for bone in line_tree.iter() {
            if bone.typo == BoneType::LINE {
                window.drawli(&bone.line.as_ref().unwrap());
            } else if bone.typo == BoneType::CIRCLE {
                window.drawci(&bone.circle.as_ref().unwrap());
            }
        }
        // ----------
    }
}

fn oldline(center_x: i32, center_y: i32)  {
    let mut line_draging = false;
    let root = Line::boxed(
        Vector2 { x: center_x as f32, y: center_y as f32 },
        Vector2 { x: center_x as f32 + 100.0, y: center_y as f32 + 100.0 },
        None
    );

    let ptr = Box::into_raw(root);

    let bind = Line::boxed(
        Vector2::new(center_x as f32 + 100.0, center_y as f32 + 100.0),
        Vector2::new(center_x as f32 + 200.0, center_y as f32 + 200.0),
        Line::encapsulate(ptr)
    );
    
    let bind2 = Line::boxed(
        Vector2::new(center_x as f32 + 100.0 , center_y as f32 + 100.0),
        Vector2::new(center_x as f32 + 200.0, center_y as f32),
        Line::encapsulate(ptr)
    );

    let mut line_tree = Node::Children(vec![
        Node::Leaf(Line::encapsulate(ptr).unwrap()),
        Node::Children(vec![
            Node::Leaf(bind),
            Node::Leaf(bind2)
        ])
    ]);

    // line_tree = imports::bin::import_from_raw("men.vec");
    
    // let cam = Camera2D {
    //     target: Vector2 { x: center_x as f32, y: center_y as f32 },
    //     offset: Vector2 { x: center_x as f32, y: center_y as f32 },
    //     rotation: 0.0,
    //     zoom: 1.0,
    // };

    // handle.set_target_fps(60);
    // while !handle.window_should_close() {
    //     // Update
    //     // ----------
    //     let mouse_pos = handle.get_mouse_position();

    //     for line in line_tree.iter_mut() {
    //         line.angle = 0.0;
            
    //         if check_collision_point_circle(mouse_pos, line.start, 5.0) && !line_draging {
    //             line.pressed_start = true;
    //             line_draging = true;
    //         }
            
    //         if check_collision_point_circle(mouse_pos, line.end, 5.0) && !line_draging {
    //             line.pressed_end = true;
    //             line_draging = true;
    //         }
    
    //         if handle.is_mouse_button_down(MouseButton::MOUSE_LEFT_BUTTON) {
    //             match line.parent.as_mut() {
    //                 None => {},
    //                 Some(mut parent) => {
    //                     let end = line.end;
    //                     let start = line.start;
    //                     let angle = parent.angle + end.angle_to(start);

    //                     line.start = parent.end;
    //                     line.end = vector2_add(vector2_rotate(line.width, angle), line.start)
    //                 }
    //             }

    //             if line.pressed_start {
    //                 let angle = mouse_pos.angle_to(line.end);
    //                 line.start = vector2_add(vector2_rotate(line.width, angle), line.end);
    //             }
    //             if line.pressed_end {
    //                 let angle = mouse_pos.angle_to(line.start);
    //                 line.angle = angle - line.end.angle_to(line.start);
    //                 line.end = vector2_add(vector2_rotate(line.width, angle), line.start);
    //             }
    //         }
    
    //         if handle.is_mouse_button_up(MouseButton::MOUSE_LEFT_BUTTON) {
    //             line.pressed_end = false;
    //             line.pressed_start = false;
    //             line_draging = false;
    //         }
    //     }
        
    //     // Draw
    //     let mut window = handle.begin_drawing(&thread);

    //     window.clear_background(Color::RAYWHITE);
        
    //     // window.begin_mode2D(cam);

    //     for line in line_tree.iter() {
    //         window.drawln(line);
    //     }
        // ----------
    // }
}
