use std::process;

use promptgen::Shell;

fn main() {
    let template = match std::env::args().nth(1) {
        Some(template) => template,
        None => {
            eprintln!("No template provided");
            return;
        }
    };

    match promptgen::generate(&template, Shell::Any) {
        Ok(result) => {
            println!("{}", result);
        }
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}
