use std::borrow::Cow;
use std::fmt::Display;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub primary: Color,
    pub secondary: Color,

    pub enable_tab_navigation: bool,
    pub enable_vim_navigation: bool,
    pub enable_arrow_navigation: bool,

    pub border_width: f32,
    pub border_color: Color,

    pub padding: f32,
    pub spacing: f32,

    pub button_dim: f32,

    pub buttons: Vec<ConfigButton>,
}

#[derive(Debug, Deserialize)]
#[serde(try_from = "ConfigColor")]
pub struct Color(slint::Color);

impl Color {
    pub fn take(self) -> slint::Color {
        self.0
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ConfigColor {
    Hex(String),
    Rgba {
        red: u8,
        green: u8,
        blue: u8,
        alpha: Option<u8>,
    },
}

fn hex_to_num(c: u8) -> Option<u8> {
    if c.is_ascii_digit() {
        return Some(c - b'0');
    }

    if (b'a'..=b'f').contains(&c) {
        return Some(c - b'a' + 10);
    }

    if (b'A'..=b'F').contains(&c) {
        return Some(c - b'A' + 10);
    }

    None
}

struct ColorParseError {
    value: String,
    variant: ColorParseErrorVariant,
}

#[derive(Debug)]
enum ColorParseErrorVariant {
    MissingHex,
    NonAsciiCharacters,
    InvalidColorLength,
    InvalidHexadecimal,
}

impl Display for ColorParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { value, variant } = self;

        write!(f, "Invalid color '{value}'. Reason: ")?;

        use ColorParseErrorVariant as V;
        f.write_str(match variant {
            V::MissingHex => "missing '#' character at the start",
            V::NonAsciiCharacters => "color string contains non-ascii character",
            V::InvalidColorLength => {
                "hexadecimal string is invalid length. It can either be 3 or 6 characters long"
            }
            V::InvalidHexadecimal => "invalid hexadecimal digit",
        })
    }
}

impl ColorParseError {
    fn new(value: &str, variant: ColorParseErrorVariant) -> Self {
        Self {
            value: value.to_string(),
            variant,
        }
    }
}

impl TryFrom<ConfigColor> for Color {
    type Error = ColorParseError;

    fn try_from(value: ConfigColor) -> Result<Self, Self::Error> {
        match value {
            ConfigColor::Hex(hex) => {
                use ColorParseErrorVariant as E;

                let s = hex.trim();

                let Some(s) = s.strip_prefix('#') else {
                    return Err(ColorParseError::new(&hex, E::MissingHex));
                };

                if !s.is_ascii() {
                    return Err(ColorParseError::new(&hex, E::NonAsciiCharacters));
                }

                if s.len() != 3 && s.len() != 6 {
                    return Err(ColorParseError::new(&hex, E::InvalidColorLength));
                }

                if s.len() == 3 {
                    let red = s.as_bytes()[0];
                    let green = s.as_bytes()[1];
                    let blue = s.as_bytes()[2];

                    let red = hex_to_num(red)
                        .ok_or_else(|| ColorParseError::new(&hex, E::InvalidHexadecimal))?;
                    let green = hex_to_num(green)
                        .ok_or_else(|| ColorParseError::new(&hex, E::InvalidHexadecimal))?;
                    let blue = hex_to_num(blue)
                        .ok_or_else(|| ColorParseError::new(&hex, E::InvalidHexadecimal))?;

                    let red = red | (red << 4);
                    let green = green | (green << 4);
                    let blue = blue | (blue << 4);

                    return Ok(Color(slint::Color::from_rgb_u8(red, green, blue)));
                }

                let r0 = s.as_bytes()[0];
                let r1 = s.as_bytes()[1];
                let g0 = s.as_bytes()[2];
                let g1 = s.as_bytes()[3];
                let b0 = s.as_bytes()[4];
                let b1 = s.as_bytes()[5];

                let r0 = hex_to_num(r0)
                    .ok_or_else(|| ColorParseError::new(&hex, E::InvalidHexadecimal))?;
                let r1 = hex_to_num(r1)
                    .ok_or_else(|| ColorParseError::new(&hex, E::InvalidHexadecimal))?;
                let g0 = hex_to_num(g0)
                    .ok_or_else(|| ColorParseError::new(&hex, E::InvalidHexadecimal))?;
                let g1 = hex_to_num(g1)
                    .ok_or_else(|| ColorParseError::new(&hex, E::InvalidHexadecimal))?;
                let b0 = hex_to_num(b0)
                    .ok_or_else(|| ColorParseError::new(&hex, E::InvalidHexadecimal))?;
                let b1 = hex_to_num(b1)
                    .ok_or_else(|| ColorParseError::new(&hex, E::InvalidHexadecimal))?;

                let red = (r0 << 4) | r1;
                let green = (g0 << 4) | g1;
                let blue = (b0 << 4) | b1;

                Ok(Color(slint::Color::from_rgb_u8(red, green, blue)))
            }
            ConfigColor::Rgba {
                red,
                green,
                blue,
                alpha,
            } => Ok(Color(slint::Color::from_argb_u8(
                alpha.unwrap_or(255),
                red,
                green,
                blue,
            ))),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ConfigButton {
    pub icon: Icon,
    pub command: Vec<String>,
    pub key: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(try_from = "ConfigIcon")]
pub struct Icon {
    pub content: Cow<'static, str>,
}

impl TryFrom<ConfigIcon> for Icon {
    type Error = ::std::io::Error;

    fn try_from(value: ConfigIcon) -> Result<Self, Self::Error> {
        match value {
            ConfigIcon::BuiltIn(builtin) => {
                let content = match builtin {
                    BuiltInIcon::PowerOff => include_str!("../ui/icons/poweroff.svg"),
                    BuiltInIcon::Reboot => include_str!("../ui/icons/reboot.svg"),
                    BuiltInIcon::Lock => include_str!("../ui/icons/lock.svg"),
                    BuiltInIcon::Logout => include_str!("../ui/icons/logout.svg"),
                };

                Ok(Icon {
                    content: Cow::Borrowed(content),
                })
            }
            ConfigIcon::CustomIcon(custom) => {
                let content = ::std::fs::read_to_string(custom)?;

                Ok(Icon {
                    content: Cow::Owned(content),
                })
            }
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ConfigIcon {
    BuiltIn(BuiltInIcon),
    CustomIcon(PathBuf),
}

#[derive(Debug, Deserialize)]
pub enum BuiltInIcon {
    #[serde(rename = "poweroff")]
    PowerOff,
    #[serde(rename = "reboot")]
    Reboot,
    #[serde(rename = "lock")]
    Lock,
    #[serde(rename = "logout")]
    Logout,
}
