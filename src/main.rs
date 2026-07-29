use std::env;
use std::io::{self, Read};

use statusline::parse;
use statusline::render;

fn main() {
    let use_classic = env::args().any(|arg| {
        arg == "--classic" || arg == "--no-nerdfont" || arg == "--compatibility"
    });

    let mut stdin = String::new();
    io::stdin().read_to_string(&mut stdin).unwrap();

    let input = parse::parse_input(&stdin);
    println!("{}", render::render_line(&input, use_classic));
}
