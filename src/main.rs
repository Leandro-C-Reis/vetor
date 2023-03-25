#![allow(warnings)]

mod figure;
mod imports;
mod maths;

use raylib::prelude::*;

fn main() {
    let screen_width = 1200;
    let screen_height = 700;
    let center_x = screen_width / 2;
    let center_y = screen_height / 2;
    let center = Vector2::new(center_x as f32,center_y as f32);

    let (mut handle, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("Vetor")
        .build();
    
    let mut point_pressed = false;
    let mut point_pressed_root = false;

    let mut figure = imports::bin::import_from_raw("men.vec", center);

    handle.set_target_fps(60);
    while !handle.window_should_close() {
        // ==== Update ====
        figure.update(&handle, &mut point_pressed, &mut point_pressed_root);
        // ================
        
        // ===== Draw =====
        let mut window = handle.begin_drawing(&thread);

        window.clear_background(Color::RAYWHITE);

        figure.draw(&mut window);
        // ================
    }
}