extern crate rterm;

/*
 * Creates empty 2x2 buffer
 */
#[test]
fn create() {
    let w = 2i32;
    let h = 2i32;
    let cb = rterm::CellBuf::new(w, h);

    assert_eq!(cb.dirty, false);
    assert_eq!(cb.cells.len(), (w * h) as usize);
    let cl = rterm::Cell{
        ch: ' ',
        bg: rterm::COLOR_DEFAULT,
        fg: rterm::COLOR_DEFAULT,
    };

    assert_eq!(cb.width, w);
    assert_eq!(cb.height, h);

    let v = vec![cl; 4];
    assert_eq!(v, cb.cells);
}

/*
 * Modifies 2x2 buffer and then clears it
 */
#[test]
fn clear() {
    let w = 2i32;
    let h = 2i32;
    let mut cb = rterm::CellBuf::new(w, h);

    let cl = rterm::Cell{
        ch: ' ',
        bg: rterm::COLOR_DEFAULT,
        fg: rterm::COLOR_DEFAULT,
    };
    let mut v = vec![cl; 4];
    v[1].ch = 'a';
    cb.cells[1].ch = 'a';
    assert_eq!(v, cb.cells);

    cb.clear();

    let cl = rterm::Cell{
        ch: ' ',
        bg: rterm::COLOR_DEFAULT,
        fg: rterm::COLOR_DEFAULT,
    };
    let v_empty = vec![cl; 4];
    assert_eq!(v_empty, cb.cells);
    assert!(cb.dirty);
}

/* Creates a bit modified 2x2 buffer (+ - means modified, 0 - is default value)
 *     0 0
 *     + +
 * Then it resizes it to 3x3. Old values must be preserved:
 *     0 0 0
 *     + + 0
 *     0 0 0
 * Then modifies and resizes it to 2x2
 *    0 +
 *    + +
 */
#[test]
fn resize() {
    let w = 2i32;
    let h = 2i32;
    let mut cb = rterm::CellBuf::new(w, h);

    let cl = rterm::Cell{
        ch: ' ',
        bg: rterm::COLOR_DEFAULT,
        fg: rterm::COLOR_DEFAULT,
    };
    let mut v = vec![cl.clone(); 4];
    v[3].ch = 'a';
    cb.cells[3].ch = 'a';
    v[2].fg = rterm::COLOR_RED;
    cb.cells[2].fg = rterm::COLOR_RED;
    assert_eq!(v, cb.cells);

    cb.resize(3, 3);

    let mut v_resized = vec![cl; 9];
    v_resized[4].ch = 'a';
    v_resized[3].fg = rterm::COLOR_RED;
    assert_eq!(v_resized, cb.cells);
    assert!(cb.dirty);

    v[1].ch = '+';
    cb.cells[1].ch = '+';

    cb.resize(w, h);
    assert_eq!(cb.cells.len(), (w * h) as usize);
    assert_eq!(v, cb.cells);
    assert!(cb.dirty);
}

/* Changes some values of buffer and gets the result */
#[test]
fn put_and_get() {
    let w = 3i32;
    let h = 3i32;
    let mut cb = rterm::CellBuf::new(w, h);

    let cl = rterm::Cell{
        ch: ' ',
        bg: rterm::COLOR_DEFAULT,
        fg: rterm::COLOR_DEFAULT,
    };
    let mut v = vec![cl.clone(); 9];

    let new_c = rterm::Cell{ ch: 'z', fg: rterm::COLOR_BLUE, bg: rterm::COLOR_MAGENTA};
    let idx = (1 + 2 * w) as usize;
    v[idx] = new_c.clone();
    cb.set_cell(1, 2, new_c.clone());
    let g_c = cb.get_cell(1, 2);
    assert_eq!(g_c.unwrap(), new_c);
    assert_eq!(cb.cells, v);

    let res = cb.set_cell(-1, 3, new_c);
    assert!(!res);
    assert_eq!(cb.cells, v);

    let mut no_val = cb.get_cell(-1, 0);
    assert!(no_val.is_none());
    no_val = cb.get_cell(1, -2);
    assert!(no_val.is_none());
    no_val = cb.get_cell(1, 22);
    assert!(no_val.is_none());
    no_val = cb.get_cell(7, 2);
    assert!(no_val.is_none());
    assert!(cb.dirty);

    cb.dirty = false;
    cb.set_cell(1, 2, rterm::Cell{ch: '-', fg: rterm::COLOR_RED, bg: rterm::COLOR_GREEN});
    assert!(cb.dirty);
    cb.dirty = false;
    cb.set_cell(1, 2, rterm::Cell{ch: '=', fg: rterm::COLOR_RED, bg: rterm::COLOR_GREEN});
    assert!(cb.dirty);
    cb.dirty = false;
    cb.set_cell(1, 2, rterm::Cell{ch: '=', fg: rterm::COLOR_BLUE, bg: rterm::COLOR_GREEN});
    assert!(cb.dirty);
    cb.dirty = false;
    cb.set_cell(1, 2, rterm::Cell{ch: '=', fg: rterm::COLOR_BLUE, bg: rterm::COLOR_WHITE});
    assert!(cb.dirty);
    cb.dirty = false;
    cb.set_cell(1, 2, rterm::Cell{ch: '=', fg: rterm::COLOR_BLUE, bg: rterm::COLOR_WHITE});
    assert!(!cb.dirty);
}
