use crate::core::winapi::is_system_dark_mode;
use crate::ui::motion::lerp_color;
use iced::border::Radius;
use iced::font::{Family, Stretch, Style as FontStyle, Weight};
use iced::widget::{button, checkbox, container, progress_bar, scrollable, text_input, toggler};
use iced::{Background, Border, Color, Font, Shadow, Theme, Vector};
use serde::{Deserialize, Serialize};

pub const INTER_REGULAR_BYTES: &[u8] = include_bytes!("../../assets/fonts/Inter-Regular.ttf");
pub const INTER_MEDIUM_BYTES: &[u8] = include_bytes!("../../assets/fonts/Inter-Medium.ttf");
pub const INTER_SEMIBOLD_BYTES: &[u8] = include_bytes!("../../assets/fonts/Inter-SemiBold.ttf");
pub const INTER_BOLD_BYTES: &[u8] = include_bytes!("../../assets/fonts/Inter-Bold.ttf");

pub const FONT_REGULAR: Font = Font {
    family: Family::Name("Inter"),
    weight: Weight::Normal,
    stretch: Stretch::Normal,
    style: FontStyle::Normal,
};

pub const FONT_MEDIUM: Font = Font {
    family: Family::Name("Inter"),
    weight: Weight::Medium,
    stretch: Stretch::Normal,
    style: FontStyle::Normal,
};

pub const FONT_SEMIBOLD: Font = Font {
    family: Family::Name("Inter"),
    weight: Weight::Semibold,
    stretch: Stretch::Normal,
    style: FontStyle::Normal,
};

#[allow(dead_code)]
pub const FONT_BOLD: Font = Font {
    family: Family::Name("Inter"),
    weight: Weight::Bold,
    stretch: Stretch::Normal,
    style: FontStyle::Normal,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    Dark,
    GeminiDark,
    Light,
    System,
}

impl ThemeMode {
    pub fn is_dark(self) -> bool {
        match self {
            ThemeMode::Dark | ThemeMode::GeminiDark => true,
            ThemeMode::Light => false,
            ThemeMode::System => is_system_dark_mode(),
        }
    }

    pub fn cycle(self) -> Self {
        match self {
            ThemeMode::Dark => ThemeMode::GeminiDark,
            ThemeMode::GeminiDark => ThemeMode::Light,
            ThemeMode::Light => ThemeMode::System,
            ThemeMode::System => ThemeMode::Dark,
        }
    }

    pub fn titlebar_rgb(self) -> (u8, u8, u8) {
        match self {
            ThemeMode::Dark => (16, 16, 16),          // Vesper #101010
            ThemeMode::GeminiDark => (19, 19, 20),    // Gemini LegacyDark #131314
            ThemeMode::Light => (243, 243, 241),      // Vesper Light #F3F3F1
            ThemeMode::System => {
                if is_system_dark_mode() {
                    (16, 16, 16)
                } else {
                    (243, 243, 241)
                }
            }
        }
    }
}

/// Vesper-inspired Reforge Design System Palette.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub is_dark: bool,
    pub bg_canvas: Color,
    pub bg_sidebar: Color,
    pub bg_group: Color,
    pub bg_elevated: Color,
    pub bg_hover: Color,
    pub bg_active: Color,
    pub bg_selected: Color,
    pub bg_selected_border: Color,
    pub bg_input: Color,
    pub bg_segmented: Color,
    pub scrim: Color,
    pub hairline: Color,
    pub border: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub accent: Color,
    pub accent_hover: Color,
    pub accent_fg: Color,
    pub accent_soft: Color,
    pub success: Color,
    pub success_soft: Color,
    pub warning: Color,
    pub warning_soft: Color,
    pub danger: Color,
    pub danger_solid: Color,
    pub danger_soft: Color,
}

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

const fn rgba(r: u8, g: u8, b: u8, a: f32) -> Color {
    Color::from_rgba(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a)
}

/// Iconic Vesper Dark — deep matte `#101010` obsidian, warm peach `#FFC799`, soft mint `#99FFE4`.
pub const VESPER_DARK: Palette = Palette {
    is_dark: true,
    bg_canvas: rgb(16, 16, 16),              // #101010
    bg_sidebar: rgb(20, 20, 20),             // #141414
    bg_group: rgb(23, 23, 23),               // #171717
    bg_elevated: rgb(28, 28, 28),            // #1C1C1C
    bg_hover: rgb(27, 27, 27),               // #1B1B1B
    bg_active: rgb(34, 34, 34),              // #222222
    bg_selected: rgb(30, 26, 23),            // warm subtle Vesper tint
    bg_selected_border: rgba(255, 199, 153, 0.28),
    bg_input: rgb(21, 21, 21),               // #151515
    bg_segmented: rgb(20, 20, 20),
    scrim: rgba(0, 0, 0, 0.62),
    hairline: rgb(29, 29, 29),               // #1D1D1D
    border: rgb(37, 37, 37),                 // #252525
    text_primary: rgb(245, 245, 245),        // #F5F5F5
    text_secondary: rgb(160, 160, 160),      // #A0A0A0
    text_muted: rgb(102, 102, 102),          // #666666
    accent: rgb(255, 199, 153),              // #FFC799 — Vesper Peach
    accent_hover: rgb(255, 214, 179),        // #FFD6B3
    accent_fg: rgb(16, 16, 16),              // crisp matte ink on peach
    accent_soft: rgba(255, 199, 153, 0.12),
    success: rgb(153, 255, 228),             // #99FFE4 — Vesper Mint
    success_soft: rgba(153, 255, 228, 0.10),
    warning: rgb(255, 199, 153),             // #FFC799
    warning_soft: rgba(255, 199, 153, 0.12),
    danger: rgb(255, 128, 128),              // #FF8080 — Vesper Coral Red
    danger_solid: rgb(217, 78, 78),          // #D94E4E
    danger_soft: rgba(255, 128, 128, 0.12),
};

/// Vesper Light — warm alabaster paper, deep charcoal ink, refined terracotta-amber accent.
pub const VESPER_LIGHT: Palette = Palette {
    is_dark: false,
    bg_canvas: rgb(250, 250, 249),           // #FAFAF9
    bg_sidebar: rgb(243, 243, 241),          // #F3F3F1
    bg_group: rgb(255, 255, 255),            // #FFFFFF
    bg_elevated: rgb(255, 255, 255),
    bg_hover: rgb(241, 240, 237),            // #F1F0ED
    bg_active: rgb(233, 232, 228),
    bg_selected: rgb(253, 246, 239),         // soft warm paper selection
    bg_selected_border: rgba(196, 104, 43, 0.30),
    bg_input: rgb(242, 241, 238),
    bg_segmented: rgb(236, 235, 232),
    scrim: rgba(16, 16, 16, 0.36),
    hairline: rgb(235, 234, 231),
    border: rgb(224, 223, 219),
    text_primary: rgb(20, 20, 20),           // #141414
    text_secondary: rgb(98, 98, 96),
    text_muted: rgb(146, 146, 142),
    accent: rgb(196, 104, 43),               // #C4682B — Warm Vesper Terracotta
    accent_hover: rgb(173, 88, 32),
    accent_fg: Color::WHITE,
    accent_soft: rgba(196, 104, 43, 0.10),
    success: rgb(16, 152, 108),
    success_soft: rgba(16, 152, 108, 0.10),
    warning: rgb(196, 114, 24),
    warning_soft: rgba(196, 114, 24, 0.10),
    danger: rgb(212, 62, 62),
    danger_solid: rgb(206, 56, 56),
    danger_soft: rgba(212, 62, 62, 0.10),
};

/// Classic Google Gemini `LegacyDark` — deep graphite `#131314`, `#1E1F20` surfaces, `#A8C7FA` periwinkle blue accent.
pub const GEMINI_LEGACY_DARK: Palette = Palette {
    is_dark: true,
    bg_canvas: rgb(19, 19, 20),              // #131314
    bg_sidebar: rgb(30, 31, 32),             // #1E1F20
    bg_group: rgb(30, 31, 32),               // #1E1F20
    bg_elevated: rgb(40, 42, 44),            // #282A2C
    bg_hover: rgb(40, 42, 44),               // #282A2C
    bg_active: rgb(51, 53, 55),              // #333537
    bg_selected: rgb(27, 39, 54),            // subtle Gemini blue-tinted selection
    bg_selected_border: rgba(168, 199, 250, 0.34),
    bg_input: rgb(30, 31, 32),               // #1E1F20
    bg_segmented: rgb(30, 31, 32),
    scrim: rgba(0, 0, 0, 0.62),
    hairline: rgb(42, 43, 46),               // #2A2B2E
    border: rgb(55, 57, 59),                 // #37393B
    text_primary: rgb(227, 227, 227),        // #E3E3E3
    text_secondary: rgb(196, 199, 197),      // #C4C7C5
    text_muted: rgb(142, 145, 143),          // #8E918F
    accent: rgb(168, 199, 250),              // #A8C7FA — Gemini Periwinkle Blue
    accent_hover: rgb(194, 217, 255),        // #C2D9FF
    accent_fg: rgb(6, 46, 111),              // #062E6F — Material 3 onPrimary
    accent_soft: rgba(168, 199, 250, 0.14),
    success: rgb(129, 201, 149),             // #81C995
    success_soft: rgba(129, 201, 149, 0.12),
    warning: rgb(253, 214, 99),              // #FDD663
    warning_soft: rgba(253, 214, 99, 0.12),
    danger: rgb(242, 139, 130),              // #F28B82
    danger_solid: rgb(217, 48, 37),          // #D93025
    danger_soft: rgba(242, 139, 130, 0.12),
};

pub fn palette(mode: ThemeMode) -> Palette {
    match mode {
        ThemeMode::Dark => VESPER_DARK,
        ThemeMode::GeminiDark => GEMINI_LEGACY_DARK,
        ThemeMode::Light => VESPER_LIGHT,
        ThemeMode::System => {
            if is_system_dark_mode() {
                VESPER_DARK
            } else {
                VESPER_LIGHT
            }
        }
    }
}

pub fn iced_theme(mode: ThemeMode) -> Theme {
    let p = palette(mode);
    let label = match mode {
        ThemeMode::Dark => "Vesper Dark",
        ThemeMode::GeminiDark => "Gemini LegacyDark",
        ThemeMode::Light => "Vesper Light",
        ThemeMode::System => {
            if p.is_dark {
                "Vesper Dark"
            } else {
                "Vesper Light"
            }
        }
    };
    Theme::custom(
        label.to_string(),
        iced::theme::Palette {
            background: p.bg_canvas,
            text: p.text_primary,
            primary: p.accent,
            success: p.success,
            warning: p.warning,
            danger: p.danger,
        },
    )
}

/// Interactive visual theme swatch card used in Settings.
pub fn theme_card_style(
    p: Palette,
    active: bool,
    hover_t: f32,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let t = match status {
            button::Status::Pressed => 1.0,
            _ => hover_t.clamp(0.0, 1.0),
        };
        let bg = if active {
            lerp_color(p.bg_selected, p.bg_active, t * 0.25)
        } else {
            lerp_color(p.bg_group, p.bg_hover, t)
        };
        let border_col = if active {
            p.accent
        } else {
            lerp_color(p.hairline, p.border, t)
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: p.text_primary,
            border: Border {
                color: border_col,
                width: if active { 1.5 } else { 1.0 },
                radius: Radius::from(10.0),
            },
            ..Default::default()
        }
    }
}

pub fn swatch_box_style(bg: Color, border_col: Color, radius: f32) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(bg)),
        border: Border {
            color: border_col,
            width: 1.0,
            radius: Radius::from(radius),
        },
        ..Default::default()
    }
}

pub fn canvas_style(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(p.bg_canvas)),
        text_color: Some(p.text_primary),
        ..Default::default()
    }
}

pub fn sidebar_style(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(p.bg_sidebar)),
        text_color: Some(p.text_primary),
        ..Default::default()
    }
}

pub fn hairline_style(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(p.hairline)),
        ..Default::default()
    }
}

/// Subtle panel surface (used for Settings blocks, Cleaner summary bar, and Modal sub-lists).
pub fn surface_panel_style(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(p.bg_group)),
        text_color: Some(p.text_primary),
        border: Border {
            color: p.border,
            width: 1.0,
            radius: Radius::from(10.0),
        },
        shadow: if p.is_dark {
            Shadow::default()
        } else {
            Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.03),
                offset: Vector::new(0.0, 1.0),
                blur_radius: 3.0,
            }
        },
        ..Default::default()
    }
}

/// Full-width borderless rounded row with smooth interpolated hover + zero-bleed selection.
pub fn animated_row_style(
    p: Palette,
    selected: bool,
    hover_t: f32,
) -> impl Fn(&Theme) -> container::Style {
    move |_| {
        let base_bg = if selected {
            p.bg_selected
        } else {
            Color::TRANSPARENT
        };
        let target_bg = if selected {
            lerp_color(p.bg_selected, p.bg_hover, 0.45)
        } else {
            p.bg_hover
        };
        let bg = lerp_color(base_bg, target_bg, hover_t);
        let border_color = if selected {
            p.bg_selected_border
        } else {
            Color::TRANSPARENT
        };
        container::Style {
            background: if bg.a > 0.001 {
                Some(Background::Color(bg))
            } else {
                None
            },
            text_color: Some(p.text_primary),
            border: Border {
                color: border_color,
                width: if selected { 1.0 } else { 0.0 },
                radius: Radius::from(8.0),
            },
            ..Default::default()
        }
    }
}

pub fn well_style(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(p.bg_input)),
        text_color: Some(p.text_primary),
        border: Border {
            color: p.border,
            width: 1.0,
            radius: Radius::from(8.0),
        },
        ..Default::default()
    }
}

pub fn segmented_track_style(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(p.bg_segmented)),
        border: Border {
            color: p.border,
            width: 1.0,
            radius: Radius::from(8.0),
        },
        ..Default::default()
    }
}

pub fn modal_scrim_style(p: Palette, alpha: f32) -> impl Fn(&Theme) -> container::Style {
    move |_| {
        let mut c = p.scrim;
        c.a *= alpha.clamp(0.0, 1.0);
        container::Style {
            background: Some(Background::Color(c)),
            ..Default::default()
        }
    }
}

pub fn modal_sheet_style(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(p.bg_elevated)),
        text_color: Some(p.text_primary),
        border: Border {
            color: p.border,
            width: 1.0,
            radius: Radius::from(12.0),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, if p.is_dark { 0.65 } else { 0.16 }),
            offset: Vector::new(0.0, 16.0),
            blur_radius: 40.0,
        },
        ..Default::default()
    }
}

pub fn monogram_tile_style(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(p.bg_elevated)),
        text_color: Some(p.text_secondary),
        border: Border {
            color: p.border,
            width: 1.0,
            radius: Radius::from(6.5),
        },
        ..Default::default()
    }
}

pub fn dot_style(color: Color) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(color)),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: Radius::from(999.0),
        },
        ..Default::default()
    }
}

pub fn subtle_count_style(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(p.bg_elevated)),
        text_color: Some(p.text_muted),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: Radius::from(5.0),
        },
        ..Default::default()
    }
}

pub fn accent_count_style(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(p.accent)),
        text_color: Some(p.accent_fg),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: Radius::from(5.0),
        },
        ..Default::default()
    }
}

pub fn toast_style(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(p.bg_elevated)),
        text_color: Some(p.text_primary),
        border: Border {
            color: p.border,
            width: 1.0,
            radius: Radius::from(9.0),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, if p.is_dark { 0.50 } else { 0.14 }),
            offset: Vector::new(0.0, 8.0),
            blur_radius: 22.0,
        },
        ..Default::default()
    }
}

pub fn checkbox_tile_style(p: Palette, checked: bool) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(if checked {
            p.accent
        } else {
            Color::TRANSPARENT
        })),
        border: Border {
            color: if checked { p.accent } else { p.text_muted },
            width: 1.25,
            radius: Radius::from(4.5),
        },
        ..Default::default()
    }
}

pub fn btn_transparent(p: Palette) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, _| button::Style {
        background: None,
        text_color: p.text_primary,
        border: Border::default(),
        ..Default::default()
    }
}

pub fn btn_primary(p: Palette) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let bg = match status {
            button::Status::Hovered => p.accent_hover,
            button::Status::Pressed => p.accent,
            button::Status::Disabled => p.bg_elevated,
            _ => p.accent,
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: if matches!(status, button::Status::Disabled) {
                p.text_muted
            } else {
                p.accent_fg
            },
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: Radius::from(7.0),
            },
            ..Default::default()
        }
    }
}

pub fn btn_secondary(p: Palette) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let bg = match status {
            button::Status::Hovered => p.bg_elevated,
            button::Status::Pressed => p.bg_active,
            button::Status::Disabled => Color::TRANSPARENT,
            _ => p.bg_group,
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: if matches!(status, button::Status::Disabled) {
                p.text_muted
            } else {
                p.text_primary
            },
            border: Border {
                color: p.border,
                width: 1.0,
                radius: Radius::from(7.0),
            },
            ..Default::default()
        }
    }
}

pub fn btn_ghost(p: Palette) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let bg = match status {
            button::Status::Hovered => Some(Background::Color(p.bg_elevated)),
            button::Status::Pressed => Some(Background::Color(p.bg_active)),
            _ => None,
        };
        button::Style {
            background: bg,
            text_color: match status {
                button::Status::Hovered | button::Status::Pressed => p.text_primary,
                button::Status::Disabled => p.text_muted,
                _ => p.text_secondary,
            },
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: Radius::from(6.0),
            },
            ..Default::default()
        }
    }
}

pub fn btn_destructive(p: Palette) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let (bg, fg) = match status {
            button::Status::Disabled => (p.bg_elevated, p.text_muted),
            button::Status::Hovered => (p.danger, p.accent_fg),
            _ => (p.danger_solid, Color::WHITE),
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: fg,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: Radius::from(7.0),
            },
            ..Default::default()
        }
    }
}

pub fn btn_destructive_subtle(p: Palette) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let (bg, fg) = match status {
            button::Status::Hovered | button::Status::Pressed => (p.danger_solid, Color::WHITE),
            button::Status::Disabled => (Color::TRANSPARENT, p.text_muted),
            _ => (p.danger_soft, p.danger),
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: fg,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: Radius::from(7.0),
            },
            ..Default::default()
        }
    }
}

pub fn btn_nav_animated(
    p: Palette,
    active: bool,
    hover_t: f32,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, _| {
        let bg = if active {
            Some(Background::Color(p.bg_elevated))
        } else if hover_t > 0.001 {
            Some(Background::Color(lerp_color(
                Color::TRANSPARENT,
                p.bg_hover,
                hover_t,
            )))
        } else {
            None
        };
        let fg = if active {
            p.text_primary
        } else {
            lerp_color(p.text_secondary, p.text_primary, hover_t)
        };
        button::Style {
            background: bg,
            text_color: fg,
            border: Border {
                color: if active { p.border } else { Color::TRANSPARENT },
                width: if active { 1.0 } else { 0.0 },
                radius: Radius::from(7.0),
            },
            ..Default::default()
        }
    }
}

pub fn btn_segment(p: Palette, active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let (bg, fg) = if active {
            (Some(Background::Color(p.bg_elevated)), p.text_primary)
        } else {
            match status {
                button::Status::Hovered => (Some(Background::Color(p.bg_hover)), p.text_primary),
                _ => (None, p.text_secondary),
            }
        };
        button::Style {
            background: bg,
            text_color: fg,
            border: Border {
                color: if active { p.border } else { Color::TRANSPARENT },
                width: if active { 1.0 } else { 0.0 },
                radius: Radius::from(6.0),
            },
            ..Default::default()
        }
    }
}

pub fn input_style(p: Palette) -> impl Fn(&Theme, text_input::Status) -> text_input::Style {
    move |_, status| {
        let border_c = match status {
            text_input::Status::Focused { .. } => p.accent,
            text_input::Status::Hovered => p.text_muted,
            _ => p.border,
        };
        text_input::Style {
            background: Background::Color(p.bg_input),
            border: Border {
                color: border_c,
                width: 1.0,
                radius: Radius::from(8.0),
            },
            icon: p.text_muted,
            placeholder: p.text_muted,
            value: p.text_primary,
            selection: p.accent_soft,
        }
    }
}

pub fn checkbox_style(p: Palette) -> impl Fn(&Theme, checkbox::Status) -> checkbox::Style {
    move |_, status| {
        let is_checked = matches!(
            status,
            checkbox::Status::Active { is_checked: true }
                | checkbox::Status::Hovered { is_checked: true }
                | checkbox::Status::Disabled { is_checked: true }
        );
        checkbox::Style {
            background: Background::Color(if is_checked {
                p.accent
            } else {
                Color::TRANSPARENT
            }),
            icon_color: p.accent_fg,
            border: Border {
                color: if is_checked { p.accent } else { p.text_muted },
                width: 1.25,
                radius: Radius::from(4.5),
            },
            text_color: Some(p.text_primary),
        }
    }
}

pub fn toggler_style(p: Palette) -> impl Fn(&Theme, toggler::Status) -> toggler::Style {
    move |_, status| {
        let is_toggled = matches!(
            status,
            toggler::Status::Active { is_toggled: true }
                | toggler::Status::Hovered { is_toggled: true }
                | toggler::Status::Disabled { is_toggled: true }
        );
        toggler::Style {
            background: Background::Color(if is_toggled { p.accent } else { p.border }),
            background_border_width: 0.0,
            background_border_color: Color::TRANSPARENT,
            foreground: Background::Color(if is_toggled { p.accent_fg } else { p.text_secondary }),
            foreground_border_width: 0.0,
            foreground_border_color: Color::TRANSPARENT,
            text_color: Some(p.text_primary),
            border_radius: Some(Radius::from(999.0)),
            padding_ratio: 0.14,
        }
    }
}

pub fn progress_style(p: Palette, color: Color) -> impl Fn(&Theme) -> progress_bar::Style {
    move |_| progress_bar::Style {
        background: Background::Color(p.hairline),
        bar: Background::Color(color),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: Radius::from(999.0),
        },
    }
}

pub fn scrollable_style(p: Palette) -> impl Fn(&Theme, scrollable::Status) -> scrollable::Style {
    move |_, status| {
        let scroller_color = match status {
            scrollable::Status::Hovered {
                is_vertical_scrollbar_hovered: true,
                ..
            }
            | scrollable::Status::Dragged { .. } => p.text_muted,
            _ => p.border,
        };
        let rail = scrollable::Rail {
            background: None,
            border: Border::default(),
            scroller: scrollable::Scroller {
                background: Background::Color(scroller_color),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: Radius::from(999.0),
                },
            },
        };
        scrollable::Style {
            container: container::Style::default(),
            vertical_rail: rail,
            horizontal_rail: rail,
            gap: None,
            auto_scroll: scrollable::AutoScroll {
                background: Background::Color(p.bg_elevated),
                border: Border {
                    color: p.border,
                    width: 1.0,
                    radius: Radius::from(999.0),
                },
                shadow: Shadow::default(),
                icon: p.text_secondary,
            },
        }
    }
}
