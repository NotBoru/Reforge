use crate::app::{App, Message};
use crate::core::winapi::format_bytes;
use crate::i18n::{tr, Lang};
use crate::ui::components::{
    h_divider, hover_row, page_header, primary_button, secondary_button, segmented_control,
    status_dot,
};
use crate::ui::icons;
use crate::ui::motion;
use crate::ui::theme::{
    self, Palette, ThemeMode, FONT_MEDIUM, FONT_REGULAR, FONT_SEMIBOLD, GEMINI_LEGACY_DARK,
    VESPER_DARK, VESPER_LIGHT,
};
use iced::alignment::Vertical;
use iced::widget::{
    button, column, container, mouse_area, progress_bar, row, scrollable, text, toggler, Space,
};
use iced::{Color, Element, Length, Padding};

pub fn view<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    let s = tr(app.config.lang);

    let header = page_header(
        p,
        s.settings_title,
        s.settings_sub,
        secondary_button(
            p,
            s.refresh.into(),
            Some(icons::refresh(13.0, p.text_secondary).into()),
            Some(Message::RefreshSystemInfo),
        ),
    );

    // ── 1. APPEARANCE (Visual Theme Swatches + Language) ─────────────────────
    let theme_cards = row![
        theme_swatch_card(
            app,
            p,
            "sw_vesper_dark",
            ThemeMode::Dark,
            s.theme_dark,
            "Obsidian & Peach",
            VESPER_DARK.bg_canvas,
            VESPER_DARK.bg_sidebar,
            VESPER_DARK.accent,
            VESPER_DARK.success,
        ),
        theme_swatch_card(
            app,
            p,
            "sw_gemini_dark",
            ThemeMode::GeminiDark,
            s.theme_gemini,
            "LegacyDark Slate",
            GEMINI_LEGACY_DARK.bg_canvas,
            GEMINI_LEGACY_DARK.bg_sidebar,
            GEMINI_LEGACY_DARK.accent,
            GEMINI_LEGACY_DARK.success,
        ),
        theme_swatch_card(
            app,
            p,
            "sw_vesper_light",
            ThemeMode::Light,
            s.theme_light,
            "Warm Paper",
            VESPER_LIGHT.bg_canvas,
            VESPER_LIGHT.bg_sidebar,
            VESPER_LIGHT.accent,
            VESPER_LIGHT.success,
        ),
        theme_swatch_card(
            app,
            p,
            "sw_system",
            ThemeMode::System,
            s.theme_system,
            "Auto Windows",
            VESPER_DARK.bg_canvas,
            VESPER_LIGHT.bg_canvas,
            p.accent,
            p.success,
        ),
    ]
    .spacing(10)
    .width(Length::Fill);

    let lang_row = pref_row(
        app,
        p,
        "set_lang",
        s.language_label,
        if app.config.lang == Lang::PtBr {
            "Idioma da interface e descrições dos módulos"
        } else {
            "Interface language and module descriptions"
        },
        segmented_control(
            p,
            vec![
                (
                    "English",
                    app.config.lang == Lang::En,
                    Message::SetLanguage(Lang::En),
                ),
                (
                    "Português (BR)",
                    app.config.lang == Lang::PtBr,
                    Message::SetLanguage(Lang::PtBr),
                ),
            ],
        ),
    );

    let appearance_sec = column![
        section_label(p, s.appearance_section),
        theme_cards,
        lang_row,
    ]
    .spacing(12);

    // ── 2. BEHAVIOR (Borderless Smooth Hover Rows) ───────────────────────────
    let concurrency_ctrl = row![
        button(text("−").font(FONT_SEMIBOLD).size(13))
            .padding([4, 10])
            .style(theme::btn_secondary(p))
            .on_press(Message::SetMaxConcurrency(
                app.config.max_concurrent_installs.saturating_sub(1).max(1)
            )),
        text(app.config.max_concurrent_installs.to_string())
            .font(FONT_SEMIBOLD)
            .size(13)
            .color(p.text_primary)
            .width(Length::Fixed(26.0))
            .align_x(iced::alignment::Horizontal::Center),
        button(text("+").font(FONT_SEMIBOLD).size(13))
            .padding([4, 10])
            .style(theme::btn_secondary(p))
            .on_press(Message::SetMaxConcurrency(
                (app.config.max_concurrent_installs + 1).min(6)
            )),
    ]
    .spacing(6)
    .align_y(Vertical::Center);

    let behavior_sec = column![
        section_label(p, s.behavior_section),
        column![
            pref_row(
                app,
                p,
                "set_conc",
                s.concurrency_label,
                s.concurrency_desc,
                concurrency_ctrl.into(),
            ),
            pref_row(
                app,
                p,
                "set_autoclean",
                s.auto_cleanup_label,
                s.auto_cleanup_desc,
                toggler(app.config.auto_cleanup_downloads)
                    .size(20)
                    .style(theme::toggler_style(p))
                    .on_toggle(Message::SetAutoCleanup)
                    .into(),
            ),
            pref_row(
                app,
                p,
                "set_quiet",
                s.quiet_uninstall_label,
                s.quiet_uninstall_desc,
                toggler(app.config.prefer_quiet_uninstall)
                    .size(20)
                    .style(theme::toggler_style(p))
                    .on_toggle(Message::SetPreferQuietUninstall)
                    .into(),
            ),
        ]
        .spacing(3),
    ]
    .spacing(10);

    // ── 3. SYSTEM & WINGET (Minimalist Open Telemetry, Zero Stiff Cards) ─────
    let disk_used = app
        .sys_info
        .disk_total_bytes
        .saturating_sub(app.sys_info.disk_free_bytes);
    let disk_ratio = if app.sys_info.disk_total_bytes > 0 {
        (disk_used as f32 / app.sys_info.disk_total_bytes as f32).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let spec_strip = row![
        spec_cell(
            p,
            "Operating System",
            format!("{} ({})", app.sys_info.os_name, app.sys_info.os_build)
        ),
        spec_cell(p, "Host", app.sys_info.hostname.clone()),
        spec_cell(p, "Processor", format!("{} threads", app.sys_info.cpu_threads)),
        spec_cell(p, "Memory", format!("{:.1} GB RAM", app.sys_info.ram_total_gb)),
    ]
    .spacing(16)
    .width(Length::Fill);

    let storage_strip = column![
        row![
            text("System Drive (C:)")
                .font(FONT_MEDIUM)
                .size(12.5)
                .color(p.text_primary),
            Space::new().width(Length::Fill),
            text(format!(
                "{} free of {}",
                format_bytes(app.sys_info.disk_free_bytes),
                format_bytes(app.sys_info.disk_total_bytes)
            ))
            .font(FONT_REGULAR)
            .size(12)
            .color(p.text_secondary),
        ]
        .align_y(Vertical::Center),
        progress_bar(0.0..=1.0, disk_ratio)
            .girth(Length::Fixed(5.0))
            .style(theme::progress_style(p, p.accent)),
    ]
    .spacing(7);

    let winget_row = hover_row(
        p,
        app.motion.hover_level("set_winget"),
        false,
        Message::HoverEnter("set_winget".into()),
        Message::HoverExit("set_winget".into()),
        container(
            row![
                column![
                    text("Windows Package Manager (WinGet)")
                        .font(FONT_MEDIUM)
                        .size(13)
                        .color(p.text_primary),
                    if app.sys_info.winget_available {
                        status_dot(p.success, Some(&app.sys_info.winget_version), p)
                    } else {
                        status_dot(p.warning, Some(s.winget_missing), p)
                    },
                ]
                .spacing(3)
                .width(Length::Fill),
                secondary_button(
                    p,
                    if app.winget_bootstrapping {
                        s.working.into()
                    } else {
                        s.install_winget.into()
                    },
                    if app.winget_bootstrapping {
                        Some(motion::spinner(
                            app.motion.spinner_phase,
                            13.0,
                            p.accent,
                            p.hairline,
                        ))
                    } else {
                        Some(icons::download(13.0, p.text_secondary).into())
                    },
                    if app.winget_bootstrapping {
                        None
                    } else {
                        Some(Message::BootstrapWinget)
                    },
                ),
            ]
            .spacing(12)
            .align_y(Vertical::Center),
        )
        .padding([10, 12])
        .into(),
    );

    let system_sec = column![
        section_label(p, s.system_snapshot),
        container(column![spec_strip, storage_strip].spacing(14)).padding([2, 12]),
        winget_row,
    ]
    .spacing(12);

    // ── 4. PORTABLE PROFILE & ABOUT ──────────────────────────────────────────
    let profile_sec = column![
        section_label(p, s.portable_section),
        container(
            row![
                primary_button(
                    p,
                    s.export_profile.into(),
                    Some(icons::save(13.0, p.accent_fg).into()),
                    Some(Message::ExportProfile),
                ),
                secondary_button(
                    p,
                    s.import_profile.into(),
                    Some(icons::upload(13.0, p.text_secondary).into()),
                    Some(Message::ImportProfile),
                ),
                secondary_button(
                    p,
                    s.create_restore_point.into(),
                    Some(icons::shield_check(13.0, p.accent).into()),
                    Some(Message::CreateRestorePoint),
                ),
                Space::new().width(Length::Fill),
                secondary_button(
                    p,
                    "GitHub".into(),
                    Some(icons::globe(13.0, p.text_secondary).into()),
                    Some(Message::OpenUrl(
                        "https://github.com/reforge-win/reforge".into(),
                    )),
                ),
            ]
            .spacing(8)
            .align_y(Vertical::Center),
        )
        .padding([2, 12]),
        container(
            row![
                text("Reforge v1.2  ·  Native Rust  ·  Static CRT  ·  MIT License")
                    .font(FONT_REGULAR)
                    .size(11.5)
                    .color(p.text_muted),
                Space::new().width(Length::Fill),
                text(if app.motion.reduced_motion {
                    "Reduced Motion: On"
                } else {
                    "Motion: Adaptive"
                })
                .font(FONT_REGULAR)
                .size(11.5)
                .color(p.text_muted),
            ]
        )
        .padding([2, 12]),
    ]
    .spacing(12);

    let body = column![
        appearance_sec,
        h_divider(p),
        behavior_sec,
        h_divider(p),
        system_sec,
        h_divider(p),
        profile_sec,
    ]
    .spacing(22)
    .max_width(860.0)
    .padding(Padding {
        top: 18.0 + app.motion.tab_offset_y(),
        right: 28.0,
        bottom: 32.0,
        left: 28.0,
    });

    let scroll = scrollable(body)
        .height(Length::Fill)
        .style(theme::scrollable_style(p));

    column![header, h_divider(p), scroll]
        .height(Length::Fill)
        .into()
}

fn section_label<'a>(p: Palette, title: &'a str) -> Element<'a, Message> {
    container(
        text(title)
            .font(FONT_SEMIBOLD)
            .size(12)
            .color(p.text_secondary),
    )
    .padding([0, 12])
    .into()
}

fn pref_row<'a>(
    app: &'a App,
    p: Palette,
    key: &'static str,
    title: &'a str,
    desc: &'a str,
    control: Element<'a, Message>,
) -> Element<'a, Message> {
    hover_row(
        p,
        app.motion.hover_level(key),
        false,
        Message::HoverEnter(key.to_string()),
        Message::HoverExit(key.to_string()),
        container(
            row![
                column![
                    text(title)
                        .font(FONT_MEDIUM)
                        .size(13)
                        .color(p.text_primary),
                    text(desc)
                        .font(FONT_REGULAR)
                        .size(12)
                        .color(p.text_secondary),
                ]
                .spacing(2)
                .width(Length::Fill),
                control,
            ]
            .spacing(16)
            .align_y(Vertical::Center),
        )
        .padding([10, 12])
        .into(),
    )
}

#[allow(clippy::too_many_arguments)]
fn theme_swatch_card<'a>(
    app: &'a App,
    p: Palette,
    key: &'static str,
    mode: ThemeMode,
    title: &'a str,
    subtitle: &'a str,
    canvas_c: Color,
    sidebar_c: Color,
    accent_c: Color,
    dot_c: Color,
) -> Element<'a, Message> {
    let active = app.config.theme == mode;
    let ht = app.motion.hover_level(key);

    // Mini window preview swatch
    let preview = container(
        row![
            container(
                column![
                    container(Space::new())
                        .width(Length::Fixed(16.0))
                        .height(Length::Fixed(4.0))
                        .style(theme::swatch_box_style(accent_c, Color::TRANSPARENT, 2.0)),
                    container(Space::new())
                        .width(Length::Fixed(22.0))
                        .height(Length::Fixed(3.0))
                        .style(theme::swatch_box_style(
                            Color::from_rgba(0.6, 0.6, 0.6, 0.35),
                            Color::TRANSPARENT,
                            2.0
                        )),
                ]
                .spacing(4)
                .padding(6),
            )
            .width(Length::Fixed(36.0))
            .height(Length::Fill)
            .style(theme::swatch_box_style(sidebar_c, Color::TRANSPARENT, 5.0)),
            container(
                column![
                    row![
                        container(Space::new())
                            .width(Length::Fill)
                            .height(Length::Fixed(5.0))
                            .style(theme::swatch_box_style(
                                Color::from_rgba(0.6, 0.6, 0.6, 0.25),
                                Color::TRANSPARENT,
                                2.0
                            )),
                        container(Space::new())
                            .width(Length::Fixed(18.0))
                            .height(Length::Fixed(5.0))
                            .style(theme::swatch_box_style(accent_c, Color::TRANSPARENT, 2.5)),
                    ]
                    .spacing(6),
                    row![
                        container(Space::new())
                            .width(Length::Fixed(6.0))
                            .height(Length::Fixed(6.0))
                            .style(theme::dot_style(dot_c)),
                        container(Space::new())
                            .width(Length::Fill)
                            .height(Length::Fixed(4.0))
                            .style(theme::swatch_box_style(
                                Color::from_rgba(0.6, 0.6, 0.6, 0.18),
                                Color::TRANSPARENT,
                                2.0
                            )),
                    ]
                    .spacing(5)
                    .align_y(Vertical::Center),
                ]
                .spacing(8)
                .padding(8),
            )
            .width(Length::Fill)
            .height(Length::Fill),
        ],
    )
    .width(Length::Fill)
    .height(Length::Fixed(48.0))
    .style(theme::swatch_box_style(canvas_c, p.hairline, 6.0));

    let label_row = row![
        column![
            text(title)
                .font(if active { FONT_SEMIBOLD } else { FONT_MEDIUM })
                .size(12.5)
                .color(p.text_primary),
            text(subtitle)
                .font(FONT_REGULAR)
                .size(11)
                .color(p.text_muted),
        ]
        .spacing(1)
        .width(Length::Fill),
        if active {
            Element::from(icons::check(13.0, p.accent))
        } else {
            Space::new().width(Length::Fixed(13.0)).into()
        },
    ]
    .align_y(Vertical::Center);

    mouse_area(
        button(column![preview, label_row].spacing(8))
            .width(Length::Fill)
            .padding(9)
            .style(theme::theme_card_style(p, active, ht))
            .on_press(Message::SetTheme(mode)),
    )
    .on_enter(Message::HoverEnter(key.to_string()))
    .on_exit(Message::HoverExit(key.to_string()))
    .into()
}

fn spec_cell<'a>(p: Palette, label: &'a str, val: String) -> Element<'a, Message> {
    column![
        text(label).font(FONT_REGULAR).size(11).color(p.text_muted),
        text(val)
            .font(FONT_SEMIBOLD)
            .size(12.5)
            .color(p.text_primary),
    ]
    .spacing(3)
    .width(Length::Fill)
    .into()
}
