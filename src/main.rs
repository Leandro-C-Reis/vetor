#![allow(warnings)]

mod figure;
mod imports;

use figure::{Draw, edge::*, create_figure};
use raylib::prelude::*;

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
    let mut point_pressed_root = false;

    let points = imports::bin::import_from_raw("men.vec");
    let mut line_tree = create_figure(points); 

    handle.set_target_fps(60);
    while !handle.window_should_close() {
        // Update
        // ----------

        for i in 0..line_tree.len() {
            let mut edge: Edge = line_tree[i].clone();
            line_tree[i] = edge.update(&handle, &line_tree, &mut point_pressed, &mut point_pressed_root);
        }
        
        // Draw
        let mut window = handle.begin_drawing(&thread);

        window.clear_background(Color::RAYWHITE);
        
        for edge in line_tree.iter() {
            if edge.typ == EdgeType::LINE as isize{
                window.drawli(&edge);
            } else if edge.typ == EdgeType::CIRCLE as isize {
                window.drawci(&edge);
            }
        }
        // ----------
    }
}