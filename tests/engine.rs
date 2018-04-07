extern crate rterm;

/*
 * Creates empty 4x4 engine
 */
#[test]
fn create() {
    let cb = rterm::Terminal::new();
    let (w, h) = cb.get_size();

    assert_eq!(cb.dirty(), false);
    assert_eq!(cb.cells().len(), (w * h) as usize);
    let cl = rterm::Cell{
        ch: ' ',
        bg: rterm::COLOR_DEFAULT,
        fg: rterm::COLOR_DEFAULT,
    };

    let (ww, hh) = cb.get_size();
    assert_eq!(ww, w);
    assert_eq!(hh, h);

    let sz = (w * h) as usize;
    let v = vec![cl; sz];
    assert_eq!(v, cb.cells());
}

// Put char
#[test]
fn put_char() {
    let mut cb = rterm::Terminal::new();
    let (w, h) = cb.get_size();
    let sz = (w * h) as usize;

    let cl = rterm::Cell{
        ch: ' ',
        bg: rterm::COLOR_DEFAULT,
        fg: rterm::COLOR_DEFAULT,
    };

    let mut v = vec![cl; sz];
    cb.set_foreground(rterm::COLOR_BLUE);
    let mut r = cb.put_char(1, 1, '+');
    let idx = (w+1) as usize;
    v[idx].ch = '+';
    v[idx].fg = rterm::COLOR_BLUE;
    let c = cb.get_cell(1, 1).unwrap();
    assert_eq!(v[idx], c);
    assert!(r);
    r = cb.put_char(1, 0, '=');
    assert!(r);
    r = cb.put_char(w+1, 1, '*');
    assert!(!r);

    v[1].ch = '=';
    v[1].fg = rterm::COLOR_BLUE;

    assert!(cb.dirty());
    let cells = cb.cells();
    assert_eq!(cells.len(), (w * h) as usize);
    assert_eq!(v, cells.to_vec());
}

// Put string horizontally
#[test]
fn put_string_horizontal() {
    let mut cb = rterm::Terminal::new();
    let (w, h) = cb.get_size();

    let mut r = cb.put_string(-10, 3, "example");
    assert!(!r);
    r = cb.put_string(w+1, 3, "example");
    assert!(!r);
    r = cb.put_string(1, -1, "example");
    assert!(!r);
    r = cb.put_string(1, h+1, "example");
    assert!(!r);

    r = cb.put_string(0, 0, "example");
    assert!(r);
    r = cb.put_string(2, 1, "example");
    assert!(r);
    r = cb.put_string(-4, 2, "example");
    assert!(r);
    r = cb.put_string(3, 3, "example");
    assert!(r);

    assert!(cb.dirty());
}

// Put string vertically
#[test]
fn put_string_vertical() {
    let mut cb = rterm::Terminal::new();
    let (w, h) = cb.get_size();

    let mut r = cb.put_string_vertical(3, -10, "example");
    assert!(!r);
    r = cb.put_string_vertical(3, h+1, "example");
    assert!(!r);
    r = cb.put_string_vertical(-1, 1, "example");
    assert!(!r);
    r = cb.put_string_vertical(w+1, 1, "example");
    assert!(!r);

    r = cb.put_string_vertical(0, 0, "example");
    assert!(r);
    r = cb.put_string_vertical(1, 2, "example");
    assert!(r);
    r = cb.put_string_vertical(2, -4, "example");
    assert!(r);
    r = cb.put_string_vertical(3, 3, "example");
    assert!(r);

    assert!(cb.dirty());
}

// Put vertical line
#[test]
fn put_line_vertical() {
    let mut cb = rterm::Terminal::new();
    let (w, h) = cb.get_size();

    let mut r = cb.put_vertical_line(3, -10, 7, '-');
    assert!(!r);
    r = cb.put_vertical_line(3, h+1, 7, '-');
    assert!(!r);
    r = cb.put_vertical_line(-1, 1, 7, '-');
    assert!(!r);
    r = cb.put_vertical_line(w+1, 1, 7, '-');
    assert!(!r);

    r = cb.put_vertical_line(0, -1, 7, '-');
    assert!(r);
    r = cb.put_vertical_line(1, 2, 7, '+');
    assert!(r);
    r = cb.put_vertical_line(2, -4, 7, '*');
    assert!(r);
    r = cb.put_vertical_line(3, 3, 7, '=');
    assert!(r);
    r = cb.put_vertical_line(0, 1, 2, '%');
    assert!(r);

    assert!(cb.dirty());
}

// Put horizontal line
#[test]
fn put_line_horizontal() {
    let mut cb = rterm::Terminal::new();
    let (w, h) = cb.get_size();

    let mut r = cb.put_horizontal_line(w+1, 3, 7, '-');
    assert!(!r);
    r = cb.put_horizontal_line(w+1, 3, 7, '-');
    assert!(!r);
    r = cb.put_horizontal_line(1, -1, 7, '-');
    assert!(!r);
    r = cb.put_horizontal_line(1, h+1, 7, '-');
    assert!(!r);

    r = cb.put_horizontal_line(-1, 0, 7, '-');
    assert!(r);
    r = cb.put_horizontal_line(2, 1, 7, '+');
    assert!(r);
    r = cb.put_horizontal_line(-4, 2, 7, '*');
    assert!(r);
    r = cb.put_horizontal_line(3, 3, 7, '=');
    assert!(r);
    r = cb.put_horizontal_line(1, 0, 2, '%');
    assert!(r);

    assert!(cb.dirty());
}

#[test]
fn flush() {
    let mut cb = rterm::Terminal::new();

    cb.set_foreground(rterm::COLOR_BLUE | rterm::COLOR_MAGENTA);
    let r = cb.put_char(0, 0, '$');
    assert!(r);
    assert!(cb.dirty());

    cb.flush();
    assert!(!cb.dirty());
}
