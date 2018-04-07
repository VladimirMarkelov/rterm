extern crate rterm;

fn draw_rect(cb: &mut rterm::Terminal, x: i32, y: i32, sz: i32, c: char, clr: rterm::Attribute) {
    cb.put_horizontal_line_with_attrs(x - sz, y - sz, sz * 2 + 1, c, clr, rterm::COLOR_BLACK);
    cb.put_horizontal_line_with_attrs(x - sz, y + sz, sz * 2 + 1, c, clr, rterm::COLOR_BLACK);
    cb.put_vertical_line_with_attrs(x - sz, y - sz + 1, sz * 2 - 1, c, clr, rterm::COLOR_BLACK);
    cb.put_vertical_line_with_attrs(x + sz, y - sz + 1, sz * 2 - 1, c, clr, rterm::COLOR_BLACK);
}

fn main() {
    let mut cb = rterm::Terminal::new();

    let mut x: i32 = 3;
    let mut y: i32 = 4;
    let mut sz: i32 = 1;
    let c: char = '*';
    let mut color = rterm::COLOR_WHITE;
    let mut dragged = false;

    let (mut mousex, mut mousey) = (-1i32, -1i32);

    let (cwidth, cheight) = cb.get_size();

    cb.clear();
    cb.set_foreground(color);
    if let Err(er) = cb.set_cursor_pos(0, 1) {
        panic!("Failed to set cursor position: {}", er);
    }

    loop {
        cb.put_string(0, 0, "Try dragging rectangle with mouse. ESC to exit DEMO");
        cb.put_string(0, 1, "Arrows - move, wheel/+/- resize, click - change color");
        draw_rect(&mut cb, x, y, sz, c, color);
        cb.flush();

        let (mut x1, mut y1, mut sz1, mut color1) = (x, y, sz, color);

        if let Some(ev) = cb.get_event() {
            match ev {
                rterm::Event::Key(key, ch, _) => {
                    match key {
                        rterm::KEY_ESC => break,
                        rterm::KEY_ARROW_LEFT => if x - sz > 0 {
                                                    x1 -= 1;
                                                },
                        rterm::KEY_ARROW_RIGHT => if x + 1 + sz < cwidth {
                                                    x1 += 1;
                                                },
                        rterm::KEY_ARROW_UP => if y - sz > 2 {
                                                    y1 -= 1;
                                                },
                        rterm::KEY_ARROW_DOWN => if y + 1 + sz < cheight {
                                                    y1 += 1;
                                                },
                        _ => match ch {
                                '+' | '=' => {
                                    if x - sz > 0 && x + sz + 1 < cwidth &&
                                        y - sz > 2 && y + sz + 1 < cheight {
                                        sz1 += 1;
                                    }
                                },
                                '-' | '_' => {
                                    if sz > 1 {
                                        sz1 -= 1;
                                    }
                                },
                                _ => {},
                            }
                    }
                },
                rterm::Event::Mouse(xx, yy, key, modif) => {
                    if key == rterm::MOUSE_RELEASE {
                        if !dragged {
                            color1 = color + 1;
                            if color1 > rterm::COLOR_WHITE {
                                color1 = rterm::COLOR_RED;
                            }
                        }
                        mousex = -1;
                        mousey = -1;
                        dragged = false;
                    }
                    if key == rterm::MOUSE_WHEEL_UP {
                        if x - sz > 0 && x + sz + 1 < cwidth &&
                            y - sz > 2 && y + sz + 1 < cheight {
                            sz1 += 1;
                        }
                    } else if key == rterm::MOUSE_WHEEL_DOWN {
                        if sz > 1 {
                            sz1 -= 1;
                        }
                    } else if key == rterm::MOUSE_LEFT {
                        if yy <= y + sz && yy >= y - sz && xx <= x + sz && xx >= x - sz {
                            mousex = xx;
                            mousey = yy;
                        }
                    }
                    if modif == rterm::MOD_MOTION {
                        if mousex >=0 && mousey >= 0 {
                            let dx = xx - mousex;
                            let dy = yy - mousey;

                            if dx != 0 || dy != 0 {
                                dragged = true;
                                if x + dx - sz >= 0 && x + dx + sz < cwidth &&
                                    y +dy - sz >= 2 && y + dy + sz < cheight {
                                    x1 = x + dx;
                                    y1 = y + dy;
                                }
                                mousex = xx;
                                mousey = yy;
                            }
                        }
                    }
                },
                _ => {}
            }
        }

        if sz1 != sz || x1 != x || y1 != y || color1 != color {
            draw_rect(&mut cb, x, y, sz, ' ', color);
            x = x1;
            y = y1;
            sz = sz1;
            color = color1;
        }
    }

    cb.stop();
}
