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

#[allow(dead_code)]
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

/// Centralized Reforge Design System Palette (Apple macOS / Linear surface-layer ladder).
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
    bg_canvas: rgb(14, 15, 19),           // #0E0F13 — rich obsidian neutral
    bg_sidebar: rgb(19, 20, 25),          // #131419 — macOS sidebar elevation
    bg_group: rgb(23, 25, 31),            // #17191F — macOS inset grouped container
    bg_elevated: rgb(29, 31, 39),         // #1D1F27 — modals, active segments
    bg_hover: rgba(255, 255, 255, 0.042), // whisper-subtle row hover
    bg_active: rgba(255, 255, 255, 0.075),
    bg_selected: rgba(10, 132, 255, 0.11),
    bg_input: rgb(17, 18, 23),            // #111217
    bg_segmented: rgb(17, 18, 24),
    scrim: rgba(0, 0, 0, 0.56),
    hairline: rgba(255, 255, 255, 0.065), // inner row divider
    border: rgba(255, 255, 255, 0.095),   // crisp outer surface border
    text_primary: rgb(245, 246, 249),     // #F5F6F9
    text_secondary: rgb(155, 161, 174),   // #9BA1AE
    text_muted: rgb(104, 110, 124),       // #686E7C
    accent: rgb(10, 132, 255),            // #0A84FF — Apple System Blue (Dark)
    accent_hover: rgb(46, 152, 255),
    accent_soft: rgba(10, 132, 255, 0.15),
    success: rgb(48, 209, 88),            // #30D158 — Apple System Green
    success_soft: rgba(48, 209, 88, 0.13),
    warning: rgb(255, 159, 10),           // #FF9F0A — Apple System Orange
    warning_soft: rgba(255, 159, 10, 0.14),
    danger: rgb(255, 69, 58),             // #FF453A — Apple System Red
    danger_soft: rgba(255, 69, 58, 0.14),
};

pub const LIGHT_PALETTE: Palette = Palette {
    is_dark: false,
    bg_canvas: rgb(245, 245, 247),        // #F5F5F7 — Apple macOS neutral canvas
    bg_sidebar: rgb(236, 237, 240),       // #ECEDF0 — cool macOS sidebar
    bg_group: rgb(255, 255, 255),         // #FFFFFF — crisp inset grouped white
    bg_elevated: rgb(255, 255, 255),
    bg_hover: rgba(0, 0, 0, 0.032),
    bg_active: rgba(0, 0, 0, 0.06),
    bg_selected: rgba(0, 122, 255, 0.08),
    bg_input: rgb(235, 236, 240),
    bg_segmented: rgb(229, 230, 235),
    scrim: rgba(15, 20, 30, 0.34),
    hairline: rgba(0, 0, 0, 0.068),
    border: rgba(0, 0, 0, 0.095),
    text_primary: rgb(24, 24, 27),        // #18181B
    text_secondary: rgb(94, 99, 110),     // #5E636E
    text_muted: rgb(142, 147, 158),       // #8E939E
    accent: rgb(0, 122, 255),             // #007AFF — Apple System Blue
    accent_hover: rgb(0, 98, 204),
    accent_soft: rgba(0, 122, 255, 0.10),
    success: rgb(40, 180, 68),
    success_soft: rgba(40, 180, 68, 0.10),
    warning: rgb(217, 119, 6),
    warning_soft: rgba(217, 119, 6, 0.11),
    danger: rgb(229, 57, 53),
    danger_soft: rgba(229, 57, 53, 0.10),
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

// ── SURFACE CONTAINERS ───────────────────────────────────────────────────────

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

pub fn border_divider_style(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(p.border)),
        ..Default::default()
    }
}

/// macOS System Settings inset grouped surface (holds multiple hairline-separated rows).
pub fn inset_group_style(p: Palette) -> impl Fn(&Theme) -> container::Style {
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
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.035),
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
            radius: Radius::from(14.0),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, if p.is_dark { 0.55 } else { 0.18 }),
            offset: Vector::new(0.0, 16.0),
            blur_radius: 40.0,
        },
        ..Default::default()
    }
}

pub fn monogram_tile_style(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(if p.is_dark {
            rgba(255, 255, 255, 0.06)
        } else {
            rgba(0, 0, 0, 0.05)
        })),
        text_color: Some(p.text_secondary),
        border: Border {
            color: p.hairline,
            width: 1.0,
            radius: Radius::from(7.0),
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
        background: Some(Background::Color(p.bg_hover)),
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
        text_color: Some(Color::WHITE),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: Radius::from(5.0),
        },
        ..Default::default()
    }
}

pub fn toast_style(p: Palette, accent: Color) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(p.bg_elevated)),
        text_color: Some(p.text_primary),
        border: Border {
            color: p.border,
            width: 1.0,
            radius: Radius::from(10.0),
        },
        shadow: Shadow {
            color: Color::from_rgba(
                accent.r * 0.0,
                accent.g * 0.0,
                accent.b * 0.0,
                if p.is_dark { 0.45 } else { 0.14 },
            ),
            offset: Vector::new(0.0, 8.0),
            blur_radius: 22.0,
        },
        ..Default::default()
    }
}

pub fn checkbox_tile_style(p: Palette, checked: bool) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(if checked { p.accent } else { Color::TRANSPARENT })),
        border: Border {
            color: if checked { p.accent } else { p.text_muted },
            width: 1.3,
            radius: Radius::from(4.5),
        },
        ..Default::default()
    }
}

// ── BUTTON VARIANTS ──────────────────────────────────────────────────────────

pub fn btn_primary(p: Palette) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let bg = match status {
            button::Status::Hovered => p.accent_hover,
            button::Status::Pressed => p.accent,
            button::Status::Disabled => {
                if p.is_dark {
                    rgba(255, 255, 255, 0.08)
                } else {
                    rgba(0, 0, 0, 0.08)
                }
            }
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
                radius: Radius::from(7.0),
            },
            ..Default::default()
        }
    }
}

pub fn btn_secondary(p: Palette) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let bg = match status {
            button::Status::Hovered => p.bg_hover,
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
            button::Status::Hovered => Some(Background::Color(p.bg_hover)),
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
            button::Status::Hovered => (p.danger, Color::WHITE),
            button::Status::Pressed => (p.danger, Color::WHITE),
            button::Status::Disabled => (p.bg_hover, p.text_muted),
            _ => (p.danger, Color::WHITE),
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
        let (bg, fg, border_c) = match status {
            button::Status::Hovered | button::Status::Pressed => (p.danger, Color::WHITE, p.danger),
            button::Status::Disabled => (Color::TRANSPARENT, p.text_muted, p.hairline),
            _ => (p.danger_soft, p.danger, Color::TRANSPARENT),
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: fg,
            border: Border {
                color: border_c,
                width: 1.0,
                radius: Radius::from(7.0),
            },
            ..Default::default()
        }
    }
}

/// Clean macOS sidebar navigation row (selected uses quiet tinted pill, never heavy borders).
pub fn btn_nav(p: Palette, active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let bg = if active {
            Some(Background::Color(if p.is_dark {
                rgba(255, 255, 255, 0.085)
            } else {
                rgba(0, 0, 0, 0.065)
            }))
        } else {
            match status {
                button::Status::Hovered => Some(Background::Color(p.bg_hover)),
                button::Status::Pressed => Some(Background::Color(p.bg_active)),
                _ => None,
            }
        };
        button::Style {
            background: bg,
            text_color: if active {
                p.text_primary
            } else if matches!(status, button::Status::Hovered) {
                p.text_primary
            } else {
                p.text_secondary
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

/// macOS Segmented Control item inside a `segmented_track_style` container.
pub fn btn_segment(p: Palette, active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let (bg, fg, shadow) = if active {
            (
                Some(Background::Color(p.bg_elevated)),
                p.text_primary,
                if p.is_dark {
                    Shadow::default()
                } else {
                    Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.08),
                        offset: Vector::new(0.0, 1.0),
                        blur_radius: 2.0,
                    }
                },
            )
        } else {
            match status {
                button::Status::Hovered => (Some(Background::Color(p.bg_hover)), p.text_primary, Shadow::default()),
                _ => (None, p.text_secondary, Shadow::default()),
            }
        };
        button::Style {
            background: bg,
            text_color: fg,
            border: Border {
                color: if active && p.is_dark { p.border } else { Color::TRANSPARENT },
                width: if active && p.is_dark { 1.0 } else { 0.0 },
                radius: Radius::from(6.0),
            },
            shadow,
            ..Default::default()
        }
    }
}

/// Hairline table/list row inside an inset group — transparent by default, crisp subtle hover/select.
pub fn btn_list_row(p: Palette, selected: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let bg = if selected {
            Some(Background::Color(p.bg_selected))
        } else {
            match status {
                button::Status::Hovered => Some(Background::Color(p.bg_hover)),
                button::Status::Pressed => Some(Background::Color(p.bg_active)),
                _ => None,
            }
        };
        button::Style {
            background: bg,
            text_color: p.text_primary,
            border: Border::default(),
            ..Default::default()
        }
    }
}

// ── INPUTS, CHECKBOXES, TOGGLERS, PROGRESS, SCROLLBAR ────────────────────────

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
            background: Background::Color(if is_checked { p.accent } else { Color::TRANSPARENT }),
            icon_color: Color::WHITE,
            border: Border {
                color: if is_checked { p.accent } else { p.text_muted },
                width: 1.3,
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
            background: Background::Color(if is_toggled {
                p.accent
            } else if p.is_dark {
                rgba(255, 255, 255, 0.16)
            } else {
                rgba(0, 0, 0, 0.16)
            }),
            background_border_width: 0.0,
            background_border_color: Color::TRANSPARENT,
            foreground: Background::Color(Color::WHITE),
            foreground_border_width: 0.0,
            foreground_border_color: Color::TRANSPARENT,
            text_color: Some(p.text_primary),
            border_radius: Some(Radius::from(999.0)),
            padding_ratio: 0.13,
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
