
use life_the_game::{
    draw::{self, App},
    proc,
};

use std::io::stdin;

fn main() -> Result<(), Box<dyn std::error::Error>>{
    let f = proc::Field::from_string(readlines());
    let a = App::new(f, 150000);
    draw::run(a)?;
    Ok(())
}

fn readlines() -> String {
    let mut s = String::new();
    for w in stdin().lines() {
        let w = w.unwrap();
        if !w.is_empty() {
            s.push_str(&w);
            s.push('\n')
        } else {
            break
        }
    }
    s
}
