use crate::core::winapi::is_system_dark_mode;
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

pub const FONT_BOLD: Font = Font {
    family: Family::Name("Inter"),
    weight: Weight::Bold,
    stretch: Stretch::Normal,
    style: FontStyle::Normal,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    System,
    Dark,
    Light,
}

impl ThemeMode {
    pub fn is_dark(self) -> bool {
        match self {
            ThemeMode::Dark => true,
            ThemeMode::Light => false,
            ThemeMode::System => is_system_dark_mode(),
        }
    }

    pub fn cycle(self) -> Self {
        match self {
            ThemeMode::System => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::Light,
            ThemeMode::Light => ThemeMode::System,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub is_dark: bool,
    pub bg_canvas: Color,
    pub bg_sidebar: Color,
    pub bg_surface: Color,
    pub bg_surface_hover: Color,
    pub bg_surface_active: Color,
    pub bg_input: Color,
    pub border: Color,
    pub border_subtle: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub accent: Color,
    pub accent_hover: Color,
    pub accent_soft: Color,
    pub success: Color,
    pub success_soft: Color,
    pub warning: Color,
    pub warning_soft: Color,
    pub danger: Color,
    pub danger_soft: Color,
}

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

const fn rgba(r: u8, g: u8, b: u8, a: f32) -> Color {
    Color::from_rgba(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a)
}

pub const DARK_PALETTE: Palette = Palette {
    is_dark: true,
    bg_canvas: rgb(11, 12, 15),           // #0B0C0F
    bg_sidebar: rgb(16, 17, 22),          // #101116
    bg_surface: rgb(21, 23, 30),          // #15171E
    bg_surface_hover: rgb(27, 30, 39),    // #1B1E27
    bg_surface_active: rgb(32, 36, 48),   // #202430
    bg_input: rgb(14, 15, 20),            // #0E0F14
    border: rgb(34, 37, 48),              // #222530
    border_subtle: rgb(26, 28, 36),       // #1A1C24
    text_primary: rgb(243, 244, 248),     // #F3F4F8
    text_secondary: rgb(156, 163, 178),   // #9CA3B2
    text_muted: rgb(100, 107, 122),       // #646B7A
    accent: rgb(47, 129, 247),            // #2F81F7
    accent_hover: rgb(68, 147, 248),      // #4493F8
    accent_soft: rgba(47, 129, 247, 0.14),
    success: rgb(52, 199, 89),            // #34C759
    success_soft: rgba(52, 199, 89, 0.13),
    warning: rgb(245, 165, 36),           // #F5A524
    warning_soft: rgba(245, 165, 36, 0.13),
    danger: rgb(240, 68, 56),             // #F04438
    danger_soft: rgba(240, 68, 56, 0.13),
};

pub const LIGHT_PALETTE: Palette = Palette {
    is_dark: false,
    bg_canvas: rgb(246, 247, 249),        // #F6F7F9
    bg_sidebar: rgb(239, 240, 244),       // #EFF0F4
    bg_surface: rgb(255, 255, 255),       // #FFFFFF
    bg_surface_hover: rgb(249, 250, 252), // #F9FAFC
    bg_surface_active: rgb(237, 242, 252),
    bg_input: rgb(242, 244, 248),         // #F2F4F8
    border: rgb(224, 227, 235),           // #E0E3EB
    border_subtle: rgb(234, 237, 243),    // #EAEDF3
    text_primary: rgb(17, 19, 24),        // #111318
    text_secondary: rgb(87, 96, 113),     // #576071
    text_muted: rgb(136, 145, 163),       // #8891A3
    accent: rgb(0, 102, 255),             // #0066FF
    accent_hover: rgb(0, 82, 204),        // #0052CC
    accent_soft: rgba(0, 102, 255, 0.09),
    success: rgb(22, 163, 74),            // #16A34A
    success_soft: rgba(22, 163, 74, 0.10),
    warning: rgb(217, 119, 6),            // #D97706
    warning_soft: rgba(217, 119, 6, 0.11),
    danger: rgb(220, 38, 38),             // #DC2626
    danger_soft: rgba(220, 38, 38, 0.10),
};

pub fn palette(mode: ThemeMode) -> Palette {
    if mode.is_dark() {
        DARK_PALETTE
    } else {
        LIGHT_PALETTE
    }
}

pub fn iced_theme(mode: ThemeMode) -> Theme {
    let p = palette(mode);
    Theme::custom(
        if p.is_dark { "Reforge Dark" } else { "Reforge Light" }.to_string(),
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

// ── CONTAINER STYLES ─────────────────────────────────────────────────────────

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
        border: Border {
            color: p.border,
            width: 0.0,
            radius: Radius::from(0.0),
        },
        ..Default::default()
    }
}

pub fn divider_style(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(p.border)),
        ..Default::default()
    }
}

pub fn card_style(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(p.bg_surface)),
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
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.04),
                offset: Vector::new(0.0, 1.0),
                blur_radius: 3.0,
            }
        },
        ..Default::default()
    }
}

pub fn well_style(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(p.bg_input)),
        text_color: Some(p.text_primary),
        border: Border {
            color: p.border_subtle,
            width: 1.0,
            radius: Radius::from(9.0),
        },
        ..Default::default()
    }
}

pub fn badge_style(bg: Color, fg: Color, border_color: Color) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(bg)),
        text_color: Some(fg),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: Radius::from(6.0),
        },
        ..Default::default()
    }
}

pub fn pill_badge_style(bg: Color, fg: Color) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(bg)),
        text_color: Some(fg),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: Radius::from(999.0),
        },
        ..Default::default()
    }
}

pub fn toast_style(p: Palette, accent: Color) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(p.bg_surface)),
        text_color: Some(p.text_primary),
        border: Border {
            color: accent,
            width: 1.0,
            radius: Radius::from(10.0),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, if p.is_dark { 0.35 } else { 0.12 }),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 14.0,
        },
        ..Default::default()
    }
}

// ── BUTTON STYLES ────────────────────────────────────────────────────────────

pub fn btn_primary(p: Palette) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let bg = match status {
            button::Status::Hovered => p.accent_hover,
            button::Status::Pressed => p.accent,
            button::Status::Disabled => rgba(128, 136, 152, 0.25),
            _ => p.accent,
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: if matches!(status, button::Status::Disabled) {
                p.text_muted
            } else {
                Color::WHITE
            },
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: Radius::from(8.0),
            },
            ..Default::default()
        }
    }
}

pub fn btn_secondary(p: Palette) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let bg = match status {
            button::Status::Hovered => p.bg_surface_hover,
            button::Status::Pressed => p.bg_surface_active,
            button::Status::Disabled => p.bg_input,
            _ => p.bg_surface,
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
                radius: Radius::from(8.0),
            },
            ..Default::default()
        }
    }
}

pub fn btn_ghost(p: Palette) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let bg = match status {
            button::Status::Hovered => Some(Background::Color(p.bg_surface_hover)),
            button::Status::Pressed => Some(Background::Color(p.bg_surface_active)),
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
                radius: Radius::from(7.0),
            },
            ..Default::default()
        }
    }
}

pub fn btn_danger(p: Palette) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let (bg, fg, border_c) = match status {
            button::Status::Hovered => (p.danger, Color::WHITE, p.danger),
            button::Status::Pressed => (p.danger, Color::WHITE, p.danger),
            button::Status::Disabled => (p.bg_input, p.text_muted, p.border_subtle),
            _ => (p.danger_soft, p.danger, rgba(240, 68, 56, 0.30)),
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: fg,
            border: Border {
                color: border_c,
                width: 1.0,
                radius: Radius::from(8.0),
            },
            ..Default::default()
        }
    }
}

pub fn btn_nav(p: Palette, active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let bg = if active {
            Some(Background::Color(p.accent_soft))
        } else {
            match status {
                button::Status::Hovered => Some(Background::Color(p.bg_surface_hover)),
                button::Status::Pressed => Some(Background::Color(p.bg_surface_active)),
                _ => None,
            }
        };
        button::Style {
            background: bg,
            text_color: if active {
                p.accent
            } else if matches!(status, button::Status::Hovered) {
                p.text_primary
            } else {
                p.text_secondary
            },
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: Radius::from(8.0),
            },
            ..Default::default()
        }
    }
}

pub fn btn_pill(p: Palette, active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let (bg, fg, border_c) = if active {
            (p.accent_soft, p.accent, p.accent)
        } else {
            match status {
                button::Status::Hovered => (p.bg_surface_hover, p.text_primary, p.border),
                _ => (p.bg_surface, p.text_secondary, p.border),
            }
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: fg,
            border: Border {
                color: border_c,
                width: 1.0,
                radius: Radius::from(999.0),
            },
            ..Default::default()
        }
    }
}

pub fn btn_card_selectable(
    p: Palette,
    selected: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let (bg, border_c) = if selected {
            (
                if p.is_dark {
                    rgba(47, 129, 247, 0.11)
                } else {
                    rgba(0, 102, 255, 0.06)
                },
                p.accent,
            )
        } else {
            match status {
                button::Status::Hovered => (p.bg_surface_hover, p.text_muted),
                _ => (p.bg_surface, p.border),
            }
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: p.text_primary,
            border: Border {
                color: border_c,
                width: 1.0,
                radius: Radius::from(10.0),
            },
            shadow: if p.is_dark || selected {
                Shadow::default()
            } else {
                Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.03),
                    offset: Vector::new(0.0, 1.0),
                    blur_radius: 2.0,
                }
            },
            ..Default::default()
        }
    }
}

// ── INPUT, CHECKBOX, TOGGLER, PROGRESS, SCROLLBAR ────────────────────────────

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
            background: Background::Color(if is_checked { p.accent } else { p.bg_input }),
            icon_color: Color::WHITE,
            border: Border {
                color: if is_checked { p.accent } else { p.border },
                width: 1.2,
                radius: Radius::from(5.0),
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
            foreground: Background::Color(Color::WHITE),
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
        background: Background::Color(p.border_subtle),
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
                background: Background::Color(p.bg_surface),
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
