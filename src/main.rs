use std::process;

use clap::{App, Arg, ArgMatches};
use promptgen::Shell;

const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

fn parse_command_line<'a>() -> ArgMatches<'a> {
    App::new("Bubble prompt generator")
        .version(VERSION.unwrap_or("unknown"))
        .arg(
            Arg::with_name("template")
                .help("Prompt template to use")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("shell")
                .short("s")
                .long("shell")
                .value_name("NAME")
                .help("If specified will wrap escape codes into non-printing characters specific for a shell")
                .takes_value(true)
                .possible_values(&["zsh", "bash"])
                .case_insensitive(true),
        )
        .get_matches()
}

fn main() {
    let matches = parse_command_line();
    let template = matches.value_of("template").unwrap();

    let shell = matches
        .value_of("shell")
        .map(str::to_lowercase)
        .map(|shell| match shell.as_ref() {
            "zsh" => Shell::Zsh,
            "bash" => Shell::Bash,
            _ => Shell::Any,
        })
        .unwrap_or(Shell::Any);

    match promptgen::generate(&template, shell) {
        Ok(result) => {
            println!("{}", result);
        }
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}
