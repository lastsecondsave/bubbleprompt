use std::str::Chars;

#[derive(Copy, Clone)]
enum Escape {
    Foreground(u8),
    Background(u8),
    Reset,
}

#[derive(Copy, Clone)]
struct Style {
    fg: u8,
    bg: u8,
}

#[derive(Copy, Clone)]
pub enum Shell {
    None,
    Zsh,
    Bash,
}

const OPEN_BRACE: char = '{';
const CLOSE_BRACE: char = '}';

pub fn generate(template: &str, shell: Shell) -> Result<String, String> {
    let mut buffer = String::new();

    let mut styles: Vec<Style> = Vec::new();
    let mut active_style: Option<Style> = None;
    let mut last_brace: Option<char> = None;

    let mut chars = template.chars();

    while let Some(next) = chars.next() {
        match last_brace {
            Some(brace) if brace != next => {
                push_brace(
                    &mut buffer,
                    brace,
                    active_style.as_ref(),
                    styles.last(),
                    shell,
                );
                active_style = styles.last().copied();
            }
            _ => (),
        }

        last_brace = match next {
            OPEN_BRACE => {
                styles.push(parse_style(&mut chars)?);
                Some(OPEN_BRACE)
            }
            CLOSE_BRACE => {
                styles.pop();
                Some(CLOSE_BRACE)
            }
            _ => {
                buffer.push(next);
                None
            }
        };
    }

    if !styles.is_empty() {
        return Err("Error: unbalanced braces.".to_string());
    }

    if let Some(last_brace) = last_brace {
        push_brace(
            &mut buffer,
            last_brace,
            active_style.as_ref(),
            styles.last(),
            shell,
        );
    }

    Ok(buffer)
}

fn parse_style(chars: &mut Chars) -> Result<Style, String> {
    let mut buffer = String::new();

    let meta: Vec<&str> = {
        for c in chars.take_while(|c| *c != ':') {
            buffer.push(c);
        }
        buffer.split(',').collect()
    };

    if meta.len() != 2 {
        return Err("Error: invalid style, should be 'fg,bg'.".to_string());
    }

    let fg: u8 = match meta[0].trim().parse::<u8>() {
        Ok(fg) => fg,
        Err(e) => return Err(format!("Error: invalid fg, {}.", e.to_string())),
    };

    let bg: u8 = match meta[1].trim().parse::<u8>() {
        Ok(bg) => bg,
        Err(e) => return Err(format!("Error: invalid bg, {}.", e.to_string())),
    };

    Ok(Style { fg, bg })
}

fn push_brace(
    buffer: &mut String,
    brace: char,
    style: Option<&Style>,
    next_style: Option<&Style>,
    shell: Shell,
) {
    if brace == OPEN_BRACE {
        if let Some(next_style) = next_style {
            push_escape_code(buffer, Escape::Foreground(next_style.bg), shell);
            buffer.push('');
            push_escape_code(buffer, Escape::Foreground(next_style.fg), shell);
            push_escape_code(buffer, Escape::Background(next_style.bg), shell);
        }
    } else if brace == CLOSE_BRACE {
        let escape = match next_style {
            Some(next_style) => Escape::Background(next_style.bg),
            None => Escape::Reset,
        };

        push_escape_code(buffer, escape, shell);

        if let Some(style) = style {
            push_escape_code(buffer, Escape::Foreground(style.bg), shell);
            buffer.push('');
        }

        let escape = match next_style {
            Some(next_style) => Escape::Foreground(next_style.fg),
            None => Escape::Reset,
        };

        push_escape_code(buffer, escape, shell);
    }
}

fn push_escape_code(buffer: &mut String, escape: Escape, shell: Shell) {
    match shell {
        Shell::Zsh => buffer.push_str("%{"),
        Shell::Bash => buffer.push_str("\\["),
        _ => (),
    }

    buffer.push_str("\x1b[");

    match escape {
        Escape::Foreground(color) => {
            buffer.push_str("38;5;");
            buffer.push_str(&color.to_string())
        }
        Escape::Background(color) => {
            buffer.push_str("48;5;");
            buffer.push_str(&color.to_string())
        }
        Escape::Reset => buffer.push('0'),
    };

    buffer.push('m');

    match shell {
        Shell::Zsh => buffer.push_str("%}"),
        Shell::Bash => buffer.push_str("\\]"),
        _ => (),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_section() {
        assert_eq!(
            generate("{0,1:xxx}", Shell::None),
            Ok("\x1b[38;5;1m\x1b[38;5;0m\x1b[48;5;1mxxx\x1b[0m\x1b[38;5;1m\x1b[0m".to_string())
        );
    }

    #[test]
    fn one_section_zsh() {
        assert_eq!(
            generate("{0,1:xxx}", Shell::Zsh),
            Ok("%{\x1b[38;5;1m%}%{\x1b[38;5;0m%}%{\x1b[48;5;1m%}xxx%{\x1b[0m%}%{\x1b[38;5;1m%}%{\x1b[0m%}".to_string())
        );
    }

    #[test]
    fn one_section_bash() {
        assert_eq!(
            generate("{0,1:xxx}", Shell::Bash),
            Ok("\\[\x1b[38;5;1m\\]\\[\x1b[38;5;0m\\]\\[\x1b[48;5;1m\\]xxx\\[\x1b[0m\\]\\[\x1b[38;5;1m\\]\\[\x1b[0m\\]".to_string())
        );
    }

    #[test]
    fn sequential_sections() {
        assert_eq!(
            generate("{0, 1:xxx} {100,200:yyy}", Shell::None),
            Ok("\x1b[38;5;1m\x1b[38;5;0m\x1b[48;5;1mxxx\x1b[0m\x1b[38;5;1m\x1b[0m \x1b[38;5;200m\x1b[38;5;100m\x1b[48;5;200myyy\x1b[0m\x1b[38;5;200m\x1b[0m".to_string())
        );
    }

    #[test]
    fn overlap_left() {
        assert_eq!(
            generate("{0,1:xxx {100,200:yyy}}", Shell::None),
            Ok("\x1b[38;5;1m\x1b[38;5;0m\x1b[48;5;1mxxx \x1b[38;5;200m\x1b[38;5;100m\x1b[48;5;200myyy\x1b[0m\x1b[38;5;200m\x1b[0m".to_string())
        );
    }

    #[test]
    fn overlap_right() {
        assert_eq!(
            generate("{0,1 :{100,200:yyy} xxx}", Shell::None),
            Ok("\x1b[38;5;200m\x1b[38;5;100m\x1b[48;5;200myyy\x1b[48;5;1m\x1b[38;5;200m\x1b[38;5;0m xxx\x1b[0m\x1b[38;5;1m\x1b[0m".to_string())
        );
    }

    #[test]
    fn bad_fg() {
        assert_eq!(
            generate("{999,1:xxx}", Shell::None),
            Err("Error: invalid fg, number too large to fit in target type.".to_string())
        );
    }

    #[test]
    fn bad_bg() {
        assert_eq!(
            generate("{1,-9:xxx}", Shell::None),
            Err("Error: invalid bg, invalid digit found in string.".to_string())
        );
    }

    #[test]
    fn incomplete_meta() {
        assert_eq!(
            generate("{1:xxx}", Shell::None),
            Err("Error: invalid style, should be 'fg,bg'.".to_string())
        );
    }
}
