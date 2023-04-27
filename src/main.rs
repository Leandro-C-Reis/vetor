#![allow(warnings)]

mod window;
mod icons;
mod figure;
mod imports;
mod maths;
mod util;
mod framerate;

use raylib::prelude::*;
use raylib::ffi::GuiLoadIcons;
use window::*;
use icons::*;
use std::ffi::CString;
use std::rc::Rc;

fn u864(s: &str) -> [u8;64] {
    let mut buff: [u8; 64] = [0u8; 64];
    buff[..s.len()].clone_from_slice(s.as_bytes());
    buff
}

fn main() {
    let screen_width = 1200;
    let screen_height = 700;
    let center_x = screen_width / 2;
    let center_y = screen_height / 2;
    let center = Vector2::new(center_x as f32,center_y as f32);

    let (mut handle, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("Vetor Studio")
        .build();
    
    let mut figure = imports::bin::import_from_raw("men.vec", center);
    let texture = handle.load_render_texture(&thread, handle.get_screen_width() as u32, handle.get_screen_height() as u32).ok().unwrap();
    let edit_tab = Tab::edit(figure, texture);
    let mut window = Window {
        tabs: vec![edit_tab.clone(), Tab::animate()],
        selected_tab: edit_tab,
    };

    // Load custom icons.
    unsafe { GuiLoadIcons(CString::new("./src/icons/iconset.rgi".to_owned()).unwrap().into_raw(), true) };

    handle.set_target_fps(60);
    while !handle.window_should_close() {
        // ==== Update ====
        window.update(&handle);
        // ===== END ======
        
        // ===== Draw =====
        let mut draw_handle = handle.begin_drawing(&thread);

        draw_handle.clear_background(Color::RAYWHITE);
        window.draw(&mut draw_handle, &thread);
        
        // ===== END ======
    }
}