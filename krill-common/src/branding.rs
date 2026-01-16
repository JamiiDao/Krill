use bitcode::{Decode, Encode};

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Encode, Decode)]
pub enum ColorSchemePreference {
    #[default]
    Dark,
    Light,
    PitchBlack,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Encode, Decode)]
pub struct DynamicColorScheme {
    preference: ColorSchemePreference,
    glassmorphism: bool,
}

impl DynamicColorScheme {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn preference(&self) -> ColorSchemePreference {
        self.preference
    }

    pub fn is_glassmorphism_enabled(&self) -> bool {
        self.glassmorphism
    }

    pub fn enable_glassmorphism(&mut self) -> &mut Self {
        self.glassmorphism = true;

        self
    }

    pub fn disable_glassmorphism(&mut self) -> &mut Self {
        self.glassmorphism = false;

        self
    }

    pub fn set_dark_mode(&mut self) -> &mut Self {
        self.preference = ColorSchemePreference::Dark;

        self
    }

    pub fn set_light_mode(&mut self) -> &mut Self {
        self.preference = ColorSchemePreference::Light;

        self
    }
}

impl Default for DynamicColorScheme {
    fn default() -> Self {
        Self {
            preference: ColorSchemePreference::default(),
            glassmorphism: true,
        }
    }
}

/// Colors are defined by CSS `color:var(--user-color)` variables globally
#[derive(Debug, PartialEq, Eq, Encode, Decode, Clone)]
pub struct ColorScheme {
    primary_color: String,
    secondary_color: String,
    accent_color: String,
    background_dark: String,
    background_light: String,
    font_heading: String,
    font_subheading: String,
    font_normal: String,
}

impl ColorScheme {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_brand_colors(&mut self, brand_colors: BrandColors<'_>) -> &mut Self {
        self.primary_color = brand_colors.primary.to_string();
        self.secondary_color = brand_colors.secondary.to_string();
        self.accent_color = brand_colors.accent.to_string();
        self.background_dark = brand_colors.background_dark.to_string();
        self.background_light = brand_colors.background_light.to_string();

        self
    }

    pub fn primary_color(&self) -> &str {
        self.primary_color.as_str()
    }

    pub fn secondary_color(&self) -> &str {
        self.secondary_color.as_str()
    }

    pub fn accent_color(&self) -> &str {
        self.accent_color.as_str()
    }

    pub fn background_dark(&self) -> &str {
        self.background_dark.as_str()
    }

    fn pitch_black() -> &'static str {
        "#000000"
    }

    pub fn background_light(&self) -> &str {
        self.background_light.as_str()
    }

    pub fn page_background_color(&self, color_scheme: &DynamicColorScheme) -> &str {
        match color_scheme.preference {
            ColorSchemePreference::Dark => self.background_dark.as_str(),
            ColorSchemePreference::PitchBlack => Self::pitch_black(),
            _ => self.background_dark.as_str(),
        }
    }
}

impl Default for ColorScheme {
    fn default() -> Self {
        let colors = BrandColors::default();

        Self {
            primary_color: colors.primary.to_string(),
            secondary_color: colors.secondary.to_string(),
            accent_color: colors.accent.to_string(),
            background_dark: colors.background_dark.to_string(),
            background_light: colors.background_light.to_string(),
            font_heading: colors.font_heading.to_string(),
            font_subheading: colors.font_subheading.to_string(),
            font_normal: colors.font_normal.to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct BrandColors<'a> {
    primary: &'a str,
    secondary: &'a str,
    accent: &'a str,
    background_dark: &'a str,
    background_light: &'a str,
    font_heading: &'a str,
    font_subheading: &'a str,
    font_normal: &'a str,
}

impl<'a> BrandColors<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_primary(&mut self, hex_color: &'a str) -> &mut Self {
        self.primary = hex_color;

        self
    }

    pub fn set_secondary(&mut self, hex_color: &'a str) -> &mut Self {
        self.secondary = hex_color;

        self
    }

    pub fn set_accent(&mut self, hex_color: &'a str) -> &mut Self {
        self.accent = hex_color;

        self
    }

    pub fn set_background_dark(&mut self, hex_color: &'a str) -> &mut Self {
        self.background_dark = hex_color;

        self
    }

    pub fn primary(&self) -> &str {
        self.primary
    }

    pub fn secondary(&self) -> &str {
        self.secondary
    }

    pub fn accent(&self) -> &str {
        self.accent
    }

    pub fn background_dark(&self) -> &str {
        self.background_dark
    }

    pub fn background_light(&self) -> &str {
        self.background_light
    }

    pub fn font_heading(&self) -> &str {
        self.font_heading
    }

    pub fn font_subheading(&self) -> &str {
        self.font_subheading
    }

    pub fn font_normal(&self) -> &str {
        self.font_normal
    }
}

impl<'a> Default for BrandColors<'a> {
    fn default() -> Self {
        Self {
            primary: "#FF6600",
            secondary: "#0B0414",
            accent: "#FFFFFF",
            background_dark: ColorScheme::pitch_black(),
            background_light: "#eaeaea",
            font_heading: "#FF6600",
            font_subheading: "#FFFFFF",
            font_normal: "#FFFFFF",
        }
    }
}
