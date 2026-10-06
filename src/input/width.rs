/// Returns the number of terminal columns occupied by `text`.
pub fn display_width(text: &str) -> usize {
    let mut width = 0;
    let mut chars = text.chars().peekable();

    while let Some(character) = chars.next() {
        if character == '\x1b' && chars.peek() == Some(&'[') {
            chars.next();
            for code in chars.by_ref() {
                if ('@'..='~').contains(&code) {
                    break;
                }
            }
            continue;
        }

        width += char_width(character);
    }

    width
}

fn char_width(character: char) -> usize {
    let codepoint = character as u32;

    if character.is_control() || is_combining_mark(codepoint) {
        return 0;
    }

    if is_wide(codepoint) { 2 } else { 1 }
}

// These ranges cover combining marks commonly found in terminal input.
fn is_combining_mark(codepoint: u32) -> bool {
    matches!(
        codepoint,
        0x0300..=0x036f
            | 0x0483..=0x0489
            | 0x0591..=0x05bd
            | 0x05bf
            | 0x05c1..=0x05c2
            | 0x05c4..=0x05c5
            | 0x0610..=0x061a
            | 0x064b..=0x065f
            | 0x0670
            | 0x06d6..=0x06dc
            | 0x06df..=0x06e4
            | 0x06e7..=0x06e8
            | 0x06ea..=0x06ed
            | 0x1ab0..=0x1aff
            | 0x1dc0..=0x1dff
            | 0x20d0..=0x20ff
            | 0xfe20..=0xfe2f
    )
}

// East Asian wide characters and emoji occupy two terminal columns in the
// terminals qsh targets. This intentionally keeps the implementation local
// so qsh can build without fetching an additional crate.
fn is_wide(codepoint: u32) -> bool {
    matches!(
        codepoint,
        0x1100..=0x115f
            | 0x2329..=0x232a
            | 0x2e80..=0xa4cf
            | 0xac00..=0xd7a3
            | 0xf900..=0xfaff
            | 0xfe10..=0xfe19
            | 0xfe30..=0xfe6f
            | 0xff00..=0xff60
            | 0xffe0..=0xffe6
            | 0x1f300..=0x1faff
            | 0x20000..=0x3fffd
    )
}

/// Returns the terminal column of an editor cursor.
///
/// The editor stores the cursor as a character index, while the terminal
/// needs a column measured in display cells. This is important for non-ASCII
/// text because a UTF-8 byte is not a terminal column and some characters are
/// wider than one cell.
pub fn cursor_column(prompt: &str, text: &str, cursor_position: usize) -> usize {
    let text_before_cursor: String = text.chars().take(cursor_position).collect();

    display_width(prompt) + display_width(&text_before_cursor)
}

#[cfg(test)]
mod tests {
    use super::{cursor_column, display_width};

    #[test]
    fn counts_unicode_display_width_instead_of_bytes() {
        assert_eq!(display_width("Привет"), 6);
        assert_eq!(display_width("界"), 2);
    }

    #[test]
    fn calculates_cursor_column_from_character_position() {
        assert_eq!(cursor_column("> ", "Привет", 3), 5);
        assert_eq!(cursor_column("界", "ab", 1), 3);
    }

    #[test]
    fn ignores_text_after_cursor() {
        assert_eq!(cursor_column("", "abc界", 3), 3);
    }

    #[test]
    fn handles_combining_marks_without_extra_columns() {
        assert_eq!(display_width("е\u{301}"), 1);
    }
}
