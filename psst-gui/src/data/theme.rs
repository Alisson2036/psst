use std::{
    fmt, fs,
    io::BufReader,
    path::Path,
};

use druid::{Color, Data};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use super::Config;

/// RGBA color packed as `0xRRGGBBAA` for exact equality checks in Druid `Data`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ThemeColor(u32);

impl ThemeColor {
    pub const fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(u32::from_be_bytes([r, g, b, a]))
    }

    pub fn to_druid_color(self) -> Color {
        Color::from_rgba32_u32(self.0)
    }

    /// Formats the color as `#rrggbb` when opaque, or `#rrggbbaa` when translucent.
    pub fn to_hex_string(self) -> String {
        let [r, g, b, a] = self.0.to_be_bytes();
        if a == u8::MAX {
            format!("#{r:02x}{g:02x}{b:02x}")
        } else {
            format!("#{r:02x}{g:02x}{b:02x}{a:02x}")
        }
    }

    /// Parses `#RGB`, `#RGBA`, `#RRGGBB`, or `#RRGGBBAA` (leading `#` optional).
    pub fn parse_hex(input: &str) -> Result<Self, String> {
        let hex = input.trim().strip_prefix('#').unwrap_or(input.trim());
        if !hex.is_ascii() {
            return Err(format!("invalid hex color '{input}': non-ASCII input"));
        }

        let parse_byte = |slice: &str| {
            u8::from_str_radix(slice, 16)
                .map_err(|_| format!("invalid hex color '{input}'"))
        };
        let expand_nibble = |slice: &str| {
            u8::from_str_radix(slice, 16)
                .map(|n| (n << 4) | n)
                .map_err(|_| format!("invalid hex color '{input}'"))
        };

        let (r, g, b, a) = match hex.len() {
            3 => (
                expand_nibble(&hex[0..1])?,
                expand_nibble(&hex[1..2])?,
                expand_nibble(&hex[2..3])?,
                0xff,
            ),
            4 => (
                expand_nibble(&hex[0..1])?,
                expand_nibble(&hex[1..2])?,
                expand_nibble(&hex[2..3])?,
                expand_nibble(&hex[3..4])?,
            ),
            6 => (
                parse_byte(&hex[0..2])?,
                parse_byte(&hex[2..4])?,
                parse_byte(&hex[4..6])?,
                0xff,
            ),
            8 => (
                parse_byte(&hex[0..2])?,
                parse_byte(&hex[2..4])?,
                parse_byte(&hex[4..6])?,
                parse_byte(&hex[6..8])?,
            ),
            _ => {
                return Err(format!(
                    "invalid hex color '{input}': expected 3, 4, 6, or 8 hex digits"
                ));
            }
        };

        Ok(Self::from_rgba8(r, g, b, a))
    }
}

impl From<ThemeColor> for Color {
    fn from(color: ThemeColor) -> Self {
        color.to_druid_color()
    }
}

impl Data for ThemeColor {
    fn same(&self, other: &Self) -> bool {
        self == other
    }
}

impl fmt::Display for ThemeColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex_string())
    }
}

impl Serialize for ThemeColor {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_hex_string())
    }
}

impl<'de> Deserialize<'de> for ThemeColor {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::parse_hex(&raw).map_err(de::Error::custom)
    }
}

/// Optional color overrides loaded from `theme.json` (or `PSST_THEME`).
///
/// Any omitted field falls back to the active base `Light` or `Dark` theme.
/// Palette fields (`grey_*`, `blue_*`, etc.) are applied first so derived
/// semantic colors inherit them automatically unless overridden explicitly.
#[derive(Clone, Debug, Default, PartialEq, Eq, Data, Serialize, Deserialize)]
#[serde(default)]
pub struct CustomTheme {
    // Base palette overrides
    pub grey_000: Option<ThemeColor>,
    pub grey_100: Option<ThemeColor>,
    pub grey_200: Option<ThemeColor>,
    pub grey_300: Option<ThemeColor>,
    pub grey_400: Option<ThemeColor>,
    pub grey_500: Option<ThemeColor>,
    pub grey_600: Option<ThemeColor>,
    pub grey_700: Option<ThemeColor>,
    pub blue_100: Option<ThemeColor>,
    pub blue_200: Option<ThemeColor>,
    pub red: Option<ThemeColor>,
    #[serde(alias = "link_hot")]
    pub link_hot_color: Option<ThemeColor>,
    #[serde(alias = "link_active")]
    pub link_active_color: Option<ThemeColor>,
    #[serde(alias = "link_cold")]
    pub link_cold_color: Option<ThemeColor>,

    // Semantic token overrides
    #[serde(alias = "window_background")]
    pub window_background_color: Option<ThemeColor>,
    pub text_color: Option<ThemeColor>,
    pub icon_color: Option<ThemeColor>,
    pub placeholder_color: Option<ThemeColor>,
    pub primary_light: Option<ThemeColor>,
    pub primary_dark: Option<ThemeColor>,
    pub background_light: Option<ThemeColor>,
    pub background_dark: Option<ThemeColor>,
    pub foreground_light: Option<ThemeColor>,
    pub foreground_dark: Option<ThemeColor>,
    pub button_light: Option<ThemeColor>,
    pub button_dark: Option<ThemeColor>,
    pub border_light: Option<ThemeColor>,
    pub border_dark: Option<ThemeColor>,
    #[serde(alias = "selection_background")]
    pub selected_text_background_color: Option<ThemeColor>,
    pub selection_text_color: Option<ThemeColor>,
    pub cursor_color: Option<ThemeColor>,
    pub scrollbar_color: Option<ThemeColor>,
    pub scrollbar_border_color: Option<ThemeColor>,
    pub menu_button_bg_active: Option<ThemeColor>,
    pub menu_button_bg_inactive: Option<ThemeColor>,
    pub menu_button_fg_active: Option<ThemeColor>,
    pub menu_button_fg_inactive: Option<ThemeColor>,
}

impl CustomTheme {
    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        let file = fs::File::open(path)
            .map_err(|err| format!("failed to open theme file {:?}: {err}", path))?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader)
            .map_err(|err| format!("failed to parse theme file {:?}: {err}", path))
    }

    /// Loads custom theme overrides from `Config::theme_path()`, falling back
    /// to empty overrides if the file does not exist or cannot be parsed.
    pub fn load_default() -> Self {
        let Some(path) = Config::theme_path() else {
            return Self::default();
        };
        if !path.exists() {
            log::info!("custom theme file not found at {:?}, using base theme", path);
            return Self::default();
        }
        match Self::load_from_path(&path) {
            Ok(theme) => {
                log::info!("loaded custom theme: {:?}", path);
                theme
            }
            Err(err) => {
                log::warn!("{err}");
                Self::default()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn parses_hex_color_formats() {
        assert_eq!(
            ThemeColor::parse_hex("#1e1e2e").unwrap(),
            ThemeColor::from_rgba8(0x1e, 0x1e, 0x2e, 0xff)
        );
        assert_eq!(
            ThemeColor::parse_hex("AABBCC80").unwrap(),
            ThemeColor::from_rgba8(0xaa, 0xbb, 0xcc, 0x80)
        );
        assert_eq!(
            ThemeColor::parse_hex(" #abc ").unwrap(),
            ThemeColor::from_rgba8(0xaa, 0xbb, 0xcc, 0xff)
        );
        assert_eq!(
            ThemeColor::parse_hex("#1234").unwrap(),
            ThemeColor::from_rgba8(0x11, 0x22, 0x33, 0x44)
        );
        assert!(ThemeColor::parse_hex("#12345").is_err());
        assert!(ThemeColor::parse_hex("#zzzzzz").is_err());
        assert!(ThemeColor::parse_hex("").is_err());
    }

    #[test]
    fn serializes_and_deserializes_partial_theme_json() {
        let json = r##"{
            "name": "Catppuccin Mocha",
            "grey_700": "#1e1e2e",
            "blue_200": "#89b4fa",
            "window_background": "#11111b",
            "link_hot": "#ffffff10"
        }"##;
        let parsed: CustomTheme = serde_json::from_str(json).unwrap();
        assert_eq!(
            parsed.grey_700,
            Some(ThemeColor::from_rgba8(0x1e, 0x1e, 0x2e, 0xff))
        );
        assert_eq!(
            parsed.blue_200,
            Some(ThemeColor::from_rgba8(0x89, 0xb4, 0xfa, 0xff))
        );
        assert_eq!(
            parsed.window_background_color,
            Some(ThemeColor::from_rgba8(0x11, 0x11, 0x1b, 0xff))
        );
        assert_eq!(
            parsed.link_hot_color,
            Some(ThemeColor::from_rgba8(0xff, 0xff, 0xff, 0x10))
        );
        assert_eq!(parsed.grey_000, None);

        let serialized = serde_json::to_string(&parsed).unwrap();
        let roundtrip: CustomTheme = serde_json::from_str(&serialized).unwrap();
        assert_eq!(parsed, roundtrip);
    }

    #[test]
    fn loads_theme_file_from_disk_and_handles_errors() {
        let dir = std::env::temp_dir().join(format!(
            "psst-theme-test-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("theme.json");

        assert!(CustomTheme::load_from_path(&path).is_err());

        {
            let mut file = fs::File::create(&path).unwrap();
            writeln!(file, r##"{{"grey_600": "#181825"}}"##).unwrap();
        }
        let loaded = CustomTheme::load_from_path(&path).unwrap();
        assert_eq!(
            loaded.grey_600,
            Some(ThemeColor::from_rgba8(0x18, 0x18, 0x25, 0xff))
        );

        {
            let mut file = fs::File::create(&path).unwrap();
            writeln!(file, r##"{{"grey_600": "not-a-color"}}"##).unwrap();
        }
        assert!(CustomTheme::load_from_path(&path).is_err());

        let _ = fs::remove_dir_all(&dir);
    }
}
