use crate::app::{App, Message};
use crate::core::winapi::format_bytes;
use crate::i18n::{tr, Lang};
use crate::ui::components::{h_divider, page_header, primary_button, secondary_button, tag_pill};
use crate::ui::icons;
use crate::ui::theme::{self, Palette, ThemeMode, FONT_MEDIUM, FONT_REGULAR, FONT_SEMIBOLD};
use iced::alignment::Vertical;
use iced::widget::{button, column, container, progress_bar, row, scrollable, text, toggler, Space};
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
            Some(icons::refresh(14.0, p.text_secondary).into()),
            Some(Message::RefreshSystemInfo),
        ),
    );

    // 1. System Snapshot card
    let disk_used = app
        .sys_info
        .disk_total_bytes
        .saturating_sub(app.sys_info.disk_free_bytes);
    let disk_ratio = if app.sys_info.disk_total_bytes > 0 {
        (disk_used as f32 / app.sys_info.disk_total_bytes as f32).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let winget_badge = if app.sys_info.winget_available {
        tag_pill(
            format!("{} ({})", s.winget_ready, app.sys_info.winget_version),
            p.success_soft,
            p.success,
        )
    } else {
        tag_pill(s.winget_missing, p.warning_soft, p.warning)
    };

    let sys_card = container(
        column![
            row![
                icons::cpu(15.0, p.accent),
                text(s.system_snapshot)
                    .font(FONT_SEMIBOLD)
                    .size(14)
                    .color(p.text_primary),
                Space::new().width(Length::Fill),
                winget_badge,
            ]
            .spacing(8)
            .align_y(Vertical::Center),
            h_divider(p),
            row![
                info_col(
                    p,
                    "OS",
                    format!("{} ({})", app.sys_info.os_name, app.sys_info.os_build)
                ),
                info_col(p, "Host", app.sys_info.hostname.clone()),
                info_col(p, "CPU", format!("{} threads", app.sys_info.cpu_threads)),
                info_col(p, "RAM", format!("{:.1} GB", app.sys_info.ram_total_gb)),
            ]
            .spacing(16),
            column![
                row![
                    text("System Drive (C:)")
                        .font(FONT_MEDIUM)
                        .size(12)
                        .color(p.text_secondary),
                    Space::new().width(Length::Fill),
                    text(format!(
                        "{} free of {}",
                        format_bytes(app.sys_info.disk_free_bytes),
                        format_bytes(app.sys_info.disk_total_bytes)
                    ))
                    .font(FONT_MEDIUM)
                    .size(12)
                    .color(p.text_primary),
                ],
                progress_bar(0.0..=1.0, disk_ratio)
                    .girth(Length::Fixed(6.0))
                    .style(theme::progress_style(p, p.accent)),
            ]
            .spacing(6),
            row![
                secondary_button(
                    p,
                    if app.winget_bootstrapping {
                        s.working.into()
                    } else {
                        s.install_winget.into()
                    },
                    Some(icons::download(14.0, p.accent).into()),
                    if app.winget_bootstrapping {
                        None
                    } else {
                        Some(Message::BootstrapWinget)
                    },
                ),
                Space::new().width(Length::Fill),
            ],
        ]
        .spacing(12),
    )
    .width(Length::Fill)
    .padding([16, 18])
    .style(theme::card_style(p));

    // 2. Appearance & Language card
    let theme_seg = |mode: ThemeMode, label: &'a str| {
        let active = app.config.theme == mode;
        button(
            text(label)
                .font(if active { FONT_SEMIBOLD } else { FONT_MEDIUM })
                .size(12),
        )
        .padding([6, 12])
        .style(theme::btn_pill(p, active))
        .on_press(Message::SetTheme(mode))
    };

    let lang_seg = |lang: Lang, label: &'a str| {
        let active = app.config.lang == lang;
        button(
            text(label)
                .font(if active { FONT_SEMIBOLD } else { FONT_MEDIUM })
                .size(12),
        )
        .padding([6, 12])
        .style(theme::btn_pill(p, active))
        .on_press(Message::SetLanguage(lang))
    };

    let appearance_card = container(
        column![
            text(s.appearance_section)
                .font(FONT_SEMIBOLD)
                .size(14)
                .color(p.text_primary),
            h_divider(p),
            row![
                text(s.theme_label)
                    .font(FONT_MEDIUM)
                    .size(13)
                    .color(p.text_primary)
                    .width(Length::Fill),
                row![
                    theme_seg(ThemeMode::System, s.theme_system),
                    theme_seg(ThemeMode::Dark, s.theme_dark),
                    theme_seg(ThemeMode::Light, s.theme_light),
                ]
                .spacing(6),
            ]
            .align_y(Vertical::Center),
            row![
                text(s.language_label)
                    .font(FONT_MEDIUM)
                    .size(13)
                    .color(p.text_primary)
                    .width(Length::Fill),
                row![
                    lang_seg(Lang::En, "English"),
                    lang_seg(Lang::PtBr, "Português (BR)"),
                ]
                .spacing(6),
            ]
            .align_y(Vertical::Center),
        ]
        .spacing(12),
    )
    .width(Length::Fill)
    .padding([16, 18])
    .style(theme::card_style(p));

    // 3. Performance & Behavior card
    let behavior_card = container(
        column![
            text(s.behavior_section)
                .font(FONT_SEMIBOLD)
                .size(14)
                .color(p.text_primary),
            h_divider(p),
            row![
                column![
                    text(s.concurrency_label)
                        .font(FONT_MEDIUM)
                        .size(13)
                        .color(p.text_primary),
                    text(s.concurrency_desc)
                        .font(FONT_REGULAR)
                        .size(12)
                        .color(p.text_secondary),
                ]
                .spacing(2)
                .width(Length::Fill),
                row![
                    button(text("-").font(FONT_SEMIBOLD).size(14))
                        .padding([4, 11])
                        .style(theme::btn_secondary(p))
                        .on_press(Message::SetMaxConcurrency(
                            app.config.max_concurrent_installs.saturating_sub(1).max(1)
                        )),
                    container(
                        text(app.config.max_concurrent_installs.to_string())
                            .font(FONT_SEMIBOLD)
                            .size(13)
                            .color(p.text_primary)
                    )
                    .padding([5, 14])
                    .style(theme::well_style(p)),
                    button(text("+").font(FONT_SEMIBOLD).size(14))
                        .padding([4, 11])
                        .style(theme::btn_secondary(p))
                        .on_press(Message::SetMaxConcurrency(
                            (app.config.max_concurrent_installs + 1).min(8)
                        )),
                ]
                .spacing(6)
                .align_y(Vertical::Center),
            ]
            .align_y(Vertical::Center),
            row![
                column![
                    text(s.auto_cleanup_label)
                        .font(FONT_MEDIUM)
                        .size(13)
                        .color(p.text_primary),
                    text(s.auto_cleanup_desc)
                        .font(FONT_REGULAR)
                        .size(12)
                        .color(p.text_secondary),
                ]
                .spacing(2)
                .width(Length::Fill),
                toggler(app.config.auto_cleanup_downloads)
                    .size(21)
                    .style(theme::toggler_style(p))
                    .on_toggle(Message::SetAutoCleanup),
            ]
            .align_y(Vertical::Center),
            row![
                column![
                    text(s.quiet_uninstall_label)
                        .font(FONT_MEDIUM)
                        .size(13)
                        .color(p.text_primary),
                    text(s.quiet_uninstall_desc)
                        .font(FONT_REGULAR)
                        .size(12)
                        .color(p.text_secondary),
                ]
                .spacing(2)
                .width(Length::Fill),
                toggler(app.config.prefer_quiet_uninstall)
                    .size(21)
                    .style(theme::toggler_style(p))
                    .on_toggle(Message::SetPreferQuietUninstall),
            ]
            .align_y(Vertical::Center),
        ]
        .spacing(12),
    )
    .width(Length::Fill)
    .padding([16, 18])
    .style(theme::card_style(p));

    // 4. Portable Profile & Safety
    let portable_card = container(
        column![
            text(s.portable_section)
                .font(FONT_SEMIBOLD)
                .size(14)
                .color(p.text_primary),
            h_divider(p),
            row![
                primary_button(
                    p,
                    s.export_profile.into(),
                    Some(icons::save(14.0, Color::WHITE).into()),
                    Some(Message::ExportProfile),
                ),
                secondary_button(
                    p,
                    s.import_profile.into(),
                    Some(icons::upload(14.0, p.text_secondary).into()),
                    Some(Message::ImportProfile),
                ),
                secondary_button(
                    p,
                    s.create_restore_point.into(),
                    Some(icons::shield_check(14.0, p.accent).into()),
                    Some(Message::CreateRestorePoint),
                ),
            ]
            .spacing(8),
        ]
        .spacing(12),
    )
    .width(Length::Fill)
    .padding([16, 18])
    .style(theme::card_style(p));

    // 5. About card
    let about_card = container(
        row![
            container(icons::brand_mark(20.0, Color::WHITE))
                .padding(8)
                .style(theme::badge_style(p.accent, Color::WHITE, Color::TRANSPARENT)),
            column![
                text("Reforge v1.0.0 — Portable Windows Post-Format Toolkit")
                    .font(FONT_SEMIBOLD)
                    .size(13)
                    .color(p.text_primary),
                text("100% Native Rust  ·  Static CRT  ·  Zero External Runtime Dependencies  ·  MIT License")
                    .font(FONT_REGULAR)
                    .size(12)
                    .color(p.text_secondary),
            ]
            .spacing(2)
            .width(Length::Fill),
            secondary_button(
                p,
                "GitHub".into(),
                Some(icons::globe(13.0, p.text_secondary).into()),
                Some(Message::OpenUrl("https://github.com/reforge-win/reforge".into())),
            ),
        ]
        .spacing(12)
        .align_y(Vertical::Center),
    )
    .width(Length::Fill)
    .padding([14, 18])
    .style(theme::card_style(p));

    let body = column![
        sys_card,
        appearance_card,
        behavior_card,
        portable_card,
        about_card
    ]
    .spacing(14)
    .padding(Padding {
        top: 16.0,
        right: 26.0,
        bottom: 24.0,
        left: 26.0,
    });

    let scroll = scrollable(body)
        .height(Length::Fill)
        .style(theme::scrollable_style(p));

    column![header, h_divider(p), scroll]
        .height(Length::Fill)
        .into()
}

fn info_col<'a>(p: Palette, label: &'a str, value: String) -> Element<'a, Message> {
    column![
        text(label)
            .font(FONT_MEDIUM)
            .size(11)
            .color(p.text_muted),
        text(value)
            .font(FONT_SEMIBOLD)
            .size(13)
            .color(p.text_primary),
    ]
    .spacing(2)
    .width(Length::Fill)
    .into()
}
