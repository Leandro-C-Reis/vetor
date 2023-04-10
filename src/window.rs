use std::{rc::Rc, ffi::CString, cell::RefCell, borrow::BorrowMut};

use raylib::{prelude::*, ffi::CheckCollisionPointRec};

use crate::{icons::VetorIcons, figure::Figure};

pub enum Tab {
    Edit {
        figure: Figure,
        framebuffer: RenderTexture2D,
        btn_pressed: bool,
        circle: bool,
        clone: bool,
        delete: bool,
        change_root: bool,
        toggle_type: bool,
        divide: bool,
        insert: bool,
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
        Rc::new(
            RefCell::new(
                Tab::Edit {
                    btn_pressed: false,
                    circle: false,
                    clone: false,
                    delete: false,
                    change_root: false,
                    toggle_type: false,
                    divide: false,
                    insert: false,
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
            Tab::Edit { figure, btn_pressed, toggle_type, divide, insert, .. } => {
                figure.update(handle);

                *btn_pressed = figure.pressed;

                if *btn_pressed {
                    if *toggle_type {
                        match figure.selected {
                            Some(index) => figure.toggle_type(index),
                            _ => ()
                        }
                        
                        *toggle_type = false;
                    }
                    
                    if *divide {
                        match figure.selected {
                            Some(index) => figure.divide(index),
                            _ => ()
                        }
                        
                        *divide = false;
                    }

                    if *insert {
                        match figure.selected {
                            Some(index) => figure.insert(index),
                            _ => ()
                        }

                        if figure.tmp_edge.is_none() {
                            *insert = false;
                        }
                    }
                }
            },
            _ => ()
        }
    }

    pub fn draw(&self, handle: &mut RaylibDrawHandle, thread: &RaylibThread) {
        let width = 80;
        let height = handle.get_screen_height() - 30;
        let start = Vector2::new(0.0 , 30.0);

        // Draw current selected tab
        match &mut *(*self.selected_tab).borrow_mut() {
            Tab::Edit { 
                figure,
                framebuffer,
                toggle_type, divide,
                insert,
                ..
            } => {
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
                // ===== END Drawing figure =====
                // ===== Drawing sidebar edit menu =====
                handle.draw_rectangle(start.x as i32, start.y as i32, width, height, Color::DARKGRAY);
        
                let mut icons = vec![];
                icons.push(
                    ("circle", CString::new(handle.gui_icon_text(VetorIcons::ICON_CIRCLE.into(), None)).unwrap())
                );
                icons.push(
                    ("line", CString::new(handle.gui_icon_text(VetorIcons::ICON_LINE.into(), None)).unwrap())
                );
                icons.push(
                    ("hexagon", CString::new(handle.gui_icon_text(VetorIcons::ICON_HEXAGON.into(), None)).unwrap())
                );
                icons.push(
                    ("copy", CString::new(handle.gui_icon_text(VetorIcons::ICON_COPY.into(), None)).unwrap())
                );
                icons.push(
                    ("toggle", CString::new(handle.gui_icon_text(VetorIcons::ICON_CIRCLE_LINED.into(), None)).unwrap())
                );
                icons.push(
                    ("delete", CString::new(handle.gui_icon_text(VetorIcons::ICON_CROSS_BOLD.into(), None)).unwrap())
                );
                icons.push(
                    ("divide", CString::new(handle.gui_icon_text(VetorIcons::ICON_DIVIDE.into(), None)).unwrap())
                );
                icons.push(
                    ("undefined", CString::new(handle.gui_icon_text(VetorIcons::ICON_UNDEFINED.into(), None)).unwrap())
                );
                icons.push(
                    ("root", CString::new(handle.gui_icon_text(VetorIcons::ICON_ROOT.into(), None)).unwrap())
                );
                icons.push(
                    ("format", CString::new(handle.gui_icon_text(VetorIcons::ICON_VERTEX_FORMAT.into(), None)).unwrap())
                );

                let mut col = 0;
                let mut row = 0;
                let gap = width as f32 * 0.05;
                let icon_width = (width / 2) as f32 - gap;
                
                fn draw_rec(handle: &mut RaylibDrawHandle,rec: Rectangle) {
                    handle.draw_rectangle(rec.x as i32, rec.y as i32, rec.width as i32, rec.height as i32, Color::new(91, 178, 217, 120));
                }

                for (i, (t, icon)) in icons.iter().enumerate() {
                    let factorx = (col as f32 * icon_width) + if col > 0 {gap * 1.5} else {gap / 2.0};
                    let factory = (row as f32 * icon_width) + gap * (row + 1) as f32;

                    if col > 0 { col=0; row+=1 } else { col+=1 };

                    handle.gui_set_style(GuiControl::BUTTON, GuiControlProperty::TEXT_ALIGNMENT as i32, GuiTextAlignment::GUI_TEXT_ALIGN_CENTER as i32);
                    let btn_press = handle.gui_button(rrect(start.x + factorx, start.y + factory, icon_width, icon_width), Some(&icon));

                    match *t {
                        "toggle" => {
                            if btn_press {
                                *toggle_type = !*toggle_type
                            }

                            if *toggle_type {
                                draw_rec(handle, rrect(start.x + factorx, start.y + factory, icon_width, icon_width));
                            }
                        },
                        "divide" => {
                            if btn_press {
                                *divide = !*divide;
                            }
                            
                            if *divide {
                                draw_rec(handle, rrect(start.x + factorx, start.y + factory, icon_width, icon_width));
                            }
                        },
                        "line" => {
                            if btn_press {
                                *insert = !*insert;
                            }

                            if *insert {
                                draw_rec(handle, rrect(start.x + factorx, start.y + factory, icon_width, icon_width));
                            }
                        }
                        _ => ()
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
