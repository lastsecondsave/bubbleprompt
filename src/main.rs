use std::str::Chars;

#[derive(Copy, Clone)]
struct Layer {
    fg: u8,
    bg: u8,
}

const OPEN_BRACE: char = '{';
const CLOSE_BRACE: char = '}';

fn main() {
    let template = match std::env::args().nth(1) {
        Some(template) => template,
        None => {
            eprintln!("No template provided");
            return;
        }
    };

    match generate(&template) {
        Ok(result) => println!("{}", result),
        Err(e) => eprintln!("{}", e),
    }
}

fn generate(template: &str) -> Result<String, String> {
    let mut buffer = String::new();

    let mut layers: Vec<Layer> = Vec::new();
    let mut curr_layer: Option<Layer> = None;
    let mut last_brace: Option<char> = None;

    let mut chars = template.chars();

    while let Some(c) = chars.next() {
        match last_brace {
            Some(last_brace) if last_brace != c => {
                let next_layer = layers.last();
                buffer.push_str(&gen_transition(curr_layer.as_ref(), next_layer, last_brace));
                curr_layer = next_layer.cloned();
            }
            _ => (),
        }

        match c {
            OPEN_BRACE => {
                layers.push(read_meta(&mut chars)?);
                last_brace = Some(OPEN_BRACE);
            }
            CLOSE_BRACE => {
                layers.pop();
                last_brace = Some(CLOSE_BRACE);
            }
            _ => {
                buffer.push(c);
                last_brace = None;
            }
        }
    }

    if let Some(last_brace) = last_brace {
        buffer.push_str(&gen_transition(
            curr_layer.as_ref(),
            layers.last(),
            last_brace,
        ));
    }

    Ok(buffer)
}

fn read_meta(chars: &mut Chars) -> Result<Layer, String> {
    let mut buffer = String::new();

    let meta: Vec<&str> = {
        for c in chars.take_while(|c| *c != ':') {
            buffer.push(c);
        }
        buffer.split(',').collect()
    };

    if meta.len() != 2 {
        return Err("Both fg and bg should be specified".to_string());
    }

    let fg: u8 = match meta[0].parse::<u8>() {
        Ok(fg) => fg,
        Err(e) => return Err(format!("Invalid fg: {}", e.to_string())),
    };

    let bg: u8 = match meta[1].parse::<u8>() {
        Ok(bg) => bg,
        Err(e) => return Err(format!("Invalid bg: {}", e.to_string())),
    };

    Ok(Layer { fg, bg })
}

fn gen_transition(curr_layer: Option<&Layer>, next_layer: Option<&Layer>, brace: char) -> String {
    let mut buffer = String::new();

    if brace == OPEN_BRACE {
        if let Some(next_layer) = next_layer {
            esc_change_fg(next_layer.bg, &mut buffer);
            buffer.push('');
            esc_change_fg(next_layer.fg, &mut buffer);
            esc_change_bg(next_layer.bg, &mut buffer);
        }
    } else if brace == CLOSE_BRACE {
        if let Some(next_layer) = next_layer {
            esc_change_bg(next_layer.bg, &mut buffer);
        } else {
            esc_reset_color(&mut buffer);
        }

        if let Some(curr_layer) = curr_layer {
            esc_change_fg(curr_layer.bg, &mut buffer);
            buffer.push('');
        }

        if let Some(next_layer) = next_layer {
            esc_change_fg(next_layer.fg, &mut buffer);
        } else {
            esc_reset_color(&mut buffer);
        }
    }

    buffer
}

fn esc_change_fg(color: u8, buffer: &mut String) {
    buffer.push_str(&format!("\x1b[38;5;{}m", color));
}

fn esc_change_bg(color: u8, buffer: &mut String) {
    buffer.push_str(&format!("\x1b[48;5;{}m", color));
}

fn esc_reset_color(buffer: &mut String) {
    buffer.push_str("\x1b[0m");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_layer() {
        assert_eq!(
            generate("{0,1:xxx}"),
            Ok("\x1b[38;5;1m\x1b[38;5;0m\x1b[48;5;1mxxx\x1b[0m\x1b[38;5;1m\x1b[0m".to_string())
        );
    }

    #[test]
    fn sequential_layers() {
        assert_eq!(
            generate("{0,1:xxx} {100,200:yyy}"),
            Ok("\x1b[38;5;1m\x1b[38;5;0m\x1b[48;5;1mxxx\x1b[0m\x1b[38;5;1m\x1b[0m \x1b[38;5;200m\x1b[38;5;100m\x1b[48;5;200myyy\x1b[0m\x1b[38;5;200m\x1b[0m".to_string())
        );
    }

    #[test]
    fn overlap_left() {
        assert_eq!(
            generate("{0,1:xxx {100,200:yyy}}"),
            Ok("\x1b[38;5;1m\x1b[38;5;0m\x1b[48;5;1mxxx \x1b[38;5;200m\x1b[38;5;100m\x1b[48;5;200myyy\x1b[0m\x1b[38;5;200m\x1b[0m".to_string())
        );
    }

    #[test]
    fn overlap_right() {
        assert_eq!(
            generate("{0,1:{100,200:yyy} xxx}"),
            Ok("\x1b[38;5;200m\x1b[38;5;100m\x1b[48;5;200myyy\x1b[48;5;1m\x1b[38;5;200m\x1b[38;5;0m xxx\x1b[0m\x1b[38;5;1m\x1b[0m".to_string())
        );
    }

    #[test]
    fn bad_fg() {
        assert_eq!(
            generate("{999,1:xxx}"),
            Err("Invalid fg: number too large to fit in target type".to_string())
        );
    }

    #[test]
    fn bad_bg() {
        assert_eq!(
            generate("{1,-9:xxx}"),
            Err("Invalid bg: invalid digit found in string".to_string())
        );
    }

    #[test]
    fn incomplete_meta() {
        assert_eq!(
            generate("{1:xxx}"),
            Err("Both fg and bg should be specified".to_string())
        );
    }
}
