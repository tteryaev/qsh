#[derive(Clone)]
pub enum Color {
    // Use terminal default color
    Default,

    // Custom RGB color
    Rgb { r: u8, g: u8, b: u8 },
}

impl Color {
    pub fn from_option(value: &Option<String>) -> Self {
        match value {
            Some(hex) => Self::from_hex(hex).unwrap_or(Color::Default),

            None => Color::Default,
        }
    }

    pub fn from_hex(value: &str) -> Option<Self> {
        let value = value.strip_prefix('#')?;

        if value.len() != 6 {
            return None;
        }

        let r = u8::from_str_radix(&value[0..2], 16).ok()?;

        let g = u8::from_str_radix(&value[2..4], 16).ok()?;

        let b = u8::from_str_radix(&value[4..6], 16).ok()?;

        Some(Color::Rgb { r, g, b })
    }

    pub fn ansi(&self) -> String {
        match self {
            Color::Default => "\x1b[39m".to_string(),

            Color::Rgb { r, g, b } => {
                format!("\x1b[38;2;{};{};{}m", r, g, b)
            }
        }
    }
}
