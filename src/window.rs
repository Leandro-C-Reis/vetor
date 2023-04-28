use std::{rc::Rc, ffi::CString, cell::RefCell, fs, path::Path};
use raylib::{prelude::*, ffi::{CheckCollisionPointRec}};
use crate::{icons::VetorIcons, figure::{Figure, edge::{Edge, EdgeFormat, EdgeDrawMode}}, log, maths::vector2_rotate};
use crate::maths::*;

struct Button {
    pub activated: bool,
    pub icon: Option<CString>,
    pub start: Vector2,
    pub len: f32,
}

impl Button {
    pub fn new(start: Vector2) -> Button {
        Button {
            activated: false,
            len: 30.0,
            icon: None,
            start
        }
    }

    pub fn set_icon(&mut self, handle: &mut RaylibDrawHandle, icon: VetorIcons) {
        self.icon = Some(
            CString::new(handle.gui_icon_text(icon.into(), None)).unwrap()
        );
    }

    pub fn dynamic_new(row: i32, col: i32, start: Vector2, width: f32) -> Button {
        let gap = width * 0.05;
        let icon_width = (width / 2.0) - gap;

        let factorx = (col as f32 * icon_width) + if col > 0 {gap * 1.5} else {gap / 2.0};
        let factory = (row as f32 * icon_width) + gap * (row + 1) as f32;

        Button {
            activated: false,
            len: icon_width,
            icon: None,
            start: Vector2::new(start.x + factorx, start.y + factory)
        }
    }
}

pub enum Tab {
    Edit {
        figure: Figure,
        framebuffer: RenderTexture2D,
        sidebar_width: f32,
        start: Vector2,
        btn_pressed: bool,
        circle: Button,
        insert: Button,
        hexagon: Button,
        copy: Button,
        toggle_type: Button,
        delete: Button,
        divide: Button,
        circle_fill: Button,
        root: Button,
        format: Button,
    },
    Animation {
        btn_pressed: bool,
        play: bool,
        repeat: bool,
        add_frame: bool,
    }
}

impl Tab {
    pub fn animate() -> Rc<RefCell<Tab>> {
        Rc::new(
            RefCell::new(
                Tab::Animation {
                    add_frame: false,
                    btn_pressed: false,
                    play: false,
                    repeat: false,
                }
            )
        )
    }

    pub fn edit(figure: Figure, texture: RenderTexture2D) -> Rc<RefCell<Tab>> {
        let sidebar_width = 80.0;
        let start = Vector2::new(0.0, 30.0);
        
        let circle = Button::dynamic_new(0, 0, start, sidebar_width);
        let insert = Button::dynamic_new(0, 1, start, sidebar_width);
        let hexagon = Button::dynamic_new(1, 0, start, sidebar_width);
        let copy = Button::dynamic_new(1, 1, start, sidebar_width);
        let toggle_type = Button::dynamic_new(2, 0, start, sidebar_width);
        let delete = Button::dynamic_new(2, 1, start, sidebar_width);
        let divide = Button::dynamic_new(3, 0, start, sidebar_width);
        let circle_fill = Button::dynamic_new(3, 1, start, sidebar_width);
        let root = Button::dynamic_new(4, 0, start, sidebar_width);
        let format = Button::dynamic_new(4, 1, start, sidebar_width);

        Rc::new(
            RefCell::new(
                Tab::Edit {
                    btn_pressed: false,
                    circle,
                    insert,
                    hexagon,
                    copy,
                    toggle_type,
                    delete,
                    divide,
                    circle_fill,
                    root,
                    format,
                    start,
                    sidebar_width,
                    figure,
                    framebuffer: texture
                }
            )
        )
    }
}

pub struct Window {
    pub tabs: Vec<Rc<RefCell<Tab>>>,
    pub selected_tab: Rc<RefCell<Tab>>
}

impl Window {
    pub fn update(&mut self, handle: &RaylibHandle) {
        if handle.is_mouse_button_pressed(MouseButton::MOUSE_LEFT_BUTTON) {
            let mouse_pos = handle.get_mouse_position();
            let collision = unsafe { CheckCollisionPointRec(mouse_pos.into(), rrect(0, 0, handle.get_screen_width(), 30).into()) };

            if collision {
                let width = 100;
                let height = 30;
    
                for (i, tab) in self.tabs.iter().enumerate() {
                    let tab_collision = unsafe { CheckCollisionPointRec(mouse_pos.into(), rrect(i as i32 * width, 0, width, height).into()) };

                    if tab_collision {
                        self.selected_tab = tab.clone();
                        break;
                    }
                }
            }
        }

        let mut tab = (*self.selected_tab).borrow_mut();

        match &mut *tab {
            Tab::Edit { 
                figure, 
                btn_pressed, 
                toggle_type, 
                divide,
                insert,
                delete,
                copy,
                format,
                circle,
                circle_fill,
                ..
            } => {
                if insert.activated || circle.activated {
                    match figure.tmp_edge {
                        Some(mut edge) => {
                            edge.end = handle.get_mouse_position();
                            edge.width = edge.start.distance_to(edge.end);
                            edge.fixed_angle = edge.end.angle_to(edge.start);
                            edge.format = if circle.activated { EdgeFormat::CIRCLE } else { EdgeFormat::LINE };
                            
                            if handle.is_mouse_button_pressed(MouseButton::MOUSE_LEFT_BUTTON) {
                                figure.insert(edge);
                                figure.tmp_edge = None;
                                figure.selected = None;
                                figure.pressed = false;
                                figure.presset_root = false;
                                figure.draw_option.point = true;
                                figure.should_update = true;

                                insert.activated = false;
                                circle.activated = false;
                                *btn_pressed = false;
                            } else {
                                figure.tmp_edge = Some(edge);
                            }
                        },
                        None => {
                            if figure.pressed {
                                let index = figure.selected.unwrap();
                                let pressed = *figure.get(index);

                                let end = if figure.presset_root {pressed.start} else {pressed.end};
                                let parent = if figure.presset_root {pressed.parent} else {index as isize};

                                figure.tmp_edge = Some(
                                    Edge::new(end, end, parent, 1)
                                );
                                
                                figure.draw_option.point = false;
                                figure.presset_root = false;
                                figure.should_update = false;

                                for index in figure.get_children(pressed.parent) {
                                    let child =  figure.get_mut(index);
                                    child.pressed_start = false;
                                    child.pressed_end = false;
                                }
                            }
                        }
                    }
                }

                if copy.activated {
                    match figure.tmp_edge {
                        Some(mut edge) => {
                            // Edge will move with mouse before insert
                            edge.start = handle.get_mouse_position();
                            edge.end = vector2_rotate(edge.width, edge.fixed_angle).add(edge.start);
                            edge.update_angle();
                            edge.moved_angle = 0.0;
                            edge.pressed_start = false;
                            edge.pressed_end = false;
                            
                            if !figure.should_update && handle.is_mouse_button_up(MouseButton::MOUSE_LEFT_BUTTON) {
                                figure.should_update = true;
                            }

                            if figure.pressed {
                                let index = figure.selected.unwrap();

                                edge.parent = index as isize;
                                figure.insert(edge);
                                figure.tmp_edge = None;
                                figure.selected = None;
                                figure.pressed = false;
                                figure.presset_root = false;
                                figure.draw_option.point = true;
                                figure.should_update = true;
                                figure.clear_edge_and_children(index);

                                copy.activated = false;
                                *btn_pressed = false;
                            } else {
                                figure.tmp_edge = Some(edge);
                            }
                        },
                        None => {
                            if figure.pressed {
                                let index = figure.selected.unwrap();
                                
                                figure.copy_tmp(index);
                                figure.selected = None;
                                figure.pressed = false;
                                figure.presset_root = false;
                                figure.should_update = false;
                                figure.clear_edge_and_children(index);
                            }
                        }
                    }
                }

                if figure.pressed {
                    if toggle_type.activated {
                        match figure.selected {
                            Some(index) => figure.toggle_type(index),
                            _ => ()
                        }
                        
                        toggle_type.activated = false;
                        *btn_pressed = false;
                    }
                    
                    if divide.activated {
                        match figure.selected {
                            Some(index) => figure.divide(index),
                            _ => ()
                        }
                        
                        divide.activated = false;
                        *btn_pressed = false;
                    }

                    if delete.activated {
                        match figure.selected {
                            Some(index) => figure.delete(index),
                            _ => ()
                        }

                        delete.activated = false;
                        *btn_pressed = false;
                    }

                    if format.activated {
                        match figure.selected {
                            Some(index) => {
                                let edge = figure.get_mut(index);
                                
                                if edge.format == EdgeFormat::LINE {
                                    edge.draw_mode = if edge.draw_mode == EdgeDrawMode::DEFAULT {EdgeDrawMode::LINE_BORDER_FLAT} else {EdgeDrawMode::DEFAULT};
                                }
                            },
                            _ => ()
                        }

                        format.activated = false;
                        *btn_pressed = false;
                    }
                    
                    if circle_fill.activated {
                        match figure.selected {
                            Some(index) => {
                                let edge = figure.get_mut(index);
                                
                                if edge.format == EdgeFormat::CIRCLE {
                                    edge.draw_mode = if edge.draw_mode == EdgeDrawMode::DEFAULT {
                                        EdgeDrawMode::CIRCLE_CLEAN
                                    } else if edge.draw_mode == EdgeDrawMode::CIRCLE_CLEAN {
                                        EdgeDrawMode::CIRCLE_FULL
                                    } else {
                                        EdgeDrawMode::DEFAULT
                                    }
                                }
                            },
                            _ => ()
                        }

                        circle_fill.activated = false;
                        *btn_pressed = false;
                    }
                }

                figure.update(handle);

                if !figure.pressed && handle.is_mouse_button_pressed(MouseButton::MOUSE_LEFT_BUTTON) {
                    figure.should_update = false;
                }
                
                if !figure.pressed && handle.is_mouse_button_up(MouseButton::MOUSE_LEFT_BUTTON) {
                    figure.should_update = true;
                }
            },
            _ => ()
        }
    }

    pub fn draw(&self, handle: &mut RaylibDrawHandle, thread: &RaylibThread) {
        // Draw current selected tab
        match &mut *(*self.selected_tab).borrow_mut() {
            Tab::Edit { 
                figure,
                framebuffer,
                toggle_type, 
                divide,
                insert,
                circle,
                circle_fill,
                copy,
                delete,
                format,
                hexagon,
                root,
                start,
                sidebar_width,
                btn_pressed,
            } => {
                let height = handle.get_screen_height() - start.y as i32;
                // ===== Drawing figure =====
                {
                    let mut draw_texture = handle.begin_texture_mode(thread, framebuffer);
                    figure.draw(&mut draw_texture);
                }
                handle.draw_texture_rec(
                    framebuffer.texture(),
                    rrect(0, 0, framebuffer.texture.width, -framebuffer.texture.height),
                    Vector2::new(0.0, 0.0),
                    Color::RAYWHITE.fade(1.0)
                );

                /// Take screenshoot when press 'F'
                /// TODO: This must be moved latter
                if handle.is_key_pressed(KeyboardKey::KEY_F) {
                    let dir = "./tests/generated";
                    let mut filename = format!("{}/screenshot_{}.png", dir, "0");

                    if Path::new(dir).is_dir() {
                        let count = fs::read_dir(dir)
                            .unwrap()
                            .map(|f| f.unwrap().path())
                            .filter(|p| p.to_str().unwrap().starts_with(&format!("{}/{}", dir, "screenshot_")))
                            .count();
                        filename = format!("{}/screenshot_{}.png", dir, count);
                    } else {
                        fs::create_dir_all(dir).unwrap();
                    }

                    let mut image = framebuffer.texture().get_texture_data().unwrap();
                    image.flip_vertical();
                    image.export_image(filename.as_str());
                }
                // ===== END Drawing figure =====
                // ===== Drawing sidebar edit menu =====

                handle.draw_rectangle(start.x as i32, start.y as i32, *sidebar_width as i32, height, Color::DARKGRAY);

                if circle.icon.is_none() {circle.set_icon(handle, VetorIcons::ICON_CIRCLE)}
                if insert.icon.is_none() {insert.set_icon(handle, VetorIcons::ICON_LINE)}
                if hexagon.icon.is_none() {hexagon.set_icon(handle, VetorIcons::ICON_HEXAGON)}
                if copy.icon.is_none() {copy.set_icon(handle, VetorIcons::ICON_COPY)}
                if toggle_type.icon.is_none() {toggle_type.set_icon(handle, VetorIcons::ICON_CIRCLE_LINED)}
                if delete.icon.is_none() {delete.set_icon(handle, VetorIcons::ICON_CROSS_BOLD)}
                if divide.icon.is_none() {divide.set_icon(handle, VetorIcons::ICON_DIVIDE)}
                if circle_fill.icon.is_none() {circle_fill.set_icon(handle, VetorIcons::ICON_UNDEFINED)}
                if root.icon.is_none() {root.set_icon(handle, VetorIcons::ICON_ROOT)}
                if format.icon.is_none() {format.set_icon(handle, VetorIcons::ICON_VERTEX_FORMAT)}

                fn draw_rec(handle: &mut RaylibDrawHandle,rec: Rectangle) {
                    handle.draw_rectangle(rec.x as i32, rec.y as i32, rec.width as i32, rec.height as i32, Color::new(91, 178, 217, 120));
                }

                for btn in [circle, insert, hexagon, copy, toggle_type, delete, divide, circle_fill, root, format] {
                    handle.gui_set_style(GuiControl::BUTTON, GuiControlProperty::TEXT_ALIGNMENT as i32, GuiTextAlignment::GUI_TEXT_ALIGN_CENTER as i32);

                    let btn_press = handle.gui_button(
                        rrect(btn.start.x, btn.start.y, btn.len, btn.len), Some(&btn.icon.clone().unwrap())
                    );

                    let must_toggle = btn_press && *btn_pressed && btn.activated;

                    if btn_press && !*btn_pressed {
                        *btn_pressed = true;
                        btn.activated = true;
                    }
                    
                    if must_toggle {
                        btn.activated = false;
                        *btn_pressed = false
                    }

                    if btn.activated {
                        draw_rec(handle, rrect(btn.start.x, btn.start.y, btn.len, btn.len));
                    }
                }
                // ===== END Drawing sidebar edit menu =====
            },
            Tab::Animation { .. } => ()
        }

        // Draw tab menu
        handle.draw_rectangle(0, 0, handle.get_screen_width(), 30, Color::LIGHTGRAY);
        for (i, tab) in self.tabs.iter().enumerate() {
            // Remove border and Align label text to center
            handle.gui_set_style(GuiControl::BUTTON, GuiControlProperty::BORDER_WIDTH as i32, 0);
            handle.gui_set_style(GuiControl::LABEL,GuiControlProperty::TEXT_ALIGNMENT as i32, GuiTextAlignment::GUI_TEXT_ALIGN_CENTER as i32);

            let text = match &*tab.borrow() {
                Tab::Edit { .. } => "Editar",
                Tab::Animation { .. } => "Animar"
            };
            let icon = CString::new(handle.gui_icon_text(VetorIcons::ICON_CROSS.into(), None)).unwrap();
            let width = 100.0;
            let x = width * i as f32;
            
            // Check if tab is the current selected.
            if tab.as_ptr() == self.selected_tab.as_ptr() {
                // Color = #5bb2d9
                handle.draw_rectangle(x as i32, 0, width as i32, 30, Color::new(91, 178, 217, 255));
            }

            handle.draw_rectangle_lines(x as i32, 0, width as i32, 30, Color::GRAY);
            handle.gui_label_button(rrect(x, 0.0, width, 30.0), Some(&CString::new(text).unwrap()));
            handle.gui_button(rrect(x + 75.0, 5.0, 20.0, 20.0), Some(&icon));
        }
    }
}
