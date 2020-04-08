#[derive(Copy, Clone)]
struct Layer {
    fg: u8,
    bg: u8,
}

const OPEN_BRACE: char = '{';
const CLOSE_BRACE: char = '}';

fn main() {
    let template = std::env::args().nth(1).expect("no template");
    print!("{}", generate(&template));
}

fn generate(template: &str) -> String {
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
                layers.push(read_meta(&mut chars));
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

    buffer
}

fn read_meta(chars: &mut std::str::Chars) -> Layer {
    let mut meta = String::new();

    for c in chars {
        if c == ':' {
            break;
        }
        meta.push(c);
    }

    let meta: Vec<u8> = meta.split(',').map(|x| x.parse::<u8>().unwrap()).collect();

    Layer {
        fg: meta[0],
        bg: meta[1],
    }
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
