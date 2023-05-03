#![allow(warnings)]

mod figure;
mod framerate;
mod icons;
mod imports;
mod maths;
mod util;
mod window;

use icons::*;
use raylib::ffi::GuiLoadIcons;
use raylib::prelude::*;
use std::ffi::CString;
use std::fs;
use std::ops::Index;
use std::rc::Rc;
use window::*;

fn u864(s: &str) -> [u8; 64] {
    let mut buff: [u8; 64] = [0u8; 64];
    buff[..s.len()].clone_from_slice(s.as_bytes());
    buff
}

fn main() {
    let screen_width = 1200;
    let screen_height = 700;
    let center_x = screen_width / 2;
    let center_y = screen_height / 2;
    let center = Vector2::new(center_x as f32, center_y as f32);

    let (mut handle, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("Vetor Studio")
        .build();

    let mut window = Window::new(&mut handle, &thread);

    // Load custom icons.
    unsafe {
        GuiLoadIcons(
            CString::new("./src/icons/iconset.rgi".to_owned())
                .unwrap()
                .into_raw(),
            true,
        )
    };

    let styles: Vec<_> = fs::read_dir("./src/styles")
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.to_str().unwrap().ends_with(".rgs"))
        .collect();

    println!("RGI Styles {:?}", styles);
    let mut selected_style = "./src/styles/cyber.rgs";
    handle.gui_load_style(Some(cstr!(selected_style).as_c_str()));

    handle.set_target_fps(60);
    while !handle.window_should_close() {
        // ==== Update ====

        /// Change current style
        if handle.is_key_pressed(KeyboardKey::KEY_F9) && styles.len() > 0 {
            let index = styles.iter().position(|p| p.ends_with(selected_style));

            if index.is_some() && index.unwrap() + 1 == styles.len() {
                selected_style = "default";
            } else if selected_style == "default" {
                selected_style = styles[0].to_str().unwrap();
            } else {
                selected_style = styles[index.unwrap() + 1].to_str().unwrap();
            };

            println!("Loading style: {:?}", selected_style);

            if selected_style == "default" {
                handle.gui_load_style_default();
            } else {
                handle.gui_load_style(Some(cstr!(selected_style).as_c_str()));
            }
        }

        window.update(&mut handle, &thread);
        // ===== END ======

        // ===== Draw =====
        let mut draw_handle = handle.begin_drawing(&thread);

        draw_handle.clear_background(Color::RAYWHITE);
        window.draw(&mut draw_handle, &thread);

        // ===== END ======
    }
}
