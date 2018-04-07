extern crate rterm;

fn main() {
    let mut cb = rterm::Terminal::new();

    cb.clear();

    loop {
        cb.put_string(5, 3, "Hello, ");
        cb.put_string_with_attrs(12, 3, "World!", rterm::COLOR_GREEN, rterm::COLOR_DEFAULT);
        cb.flush();

        if let Some(ev) = cb.get_event() {
            match ev {
                rterm::Event::Key(key, _, _) => 
                	if key == rterm::KEY_ESC {
                		break;
                	},
                _ => {}
            }
        }
    }

    cb.stop();
}
