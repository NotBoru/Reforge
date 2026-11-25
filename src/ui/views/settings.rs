use crate::app::{App, Message};
use crate::core::winapi::format_bytes;
use crate::i18n::{tr, Lang};
use crate::ui::components::{
    h_divider, inset_group, page_header, primary_button, secondary_button, section_heading,
    segmented_control, status_dot,
};
use crate::ui::icons;
use crate::ui::motion;
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
            Some(icons::refresh(13.0, p.text_secondary).into()),
            Some(Message::RefreshSystemInfo),
        ),
    );

    // 1. System Overview inset group
    let disk_used = app
        .sys_info
        .disk_total_bytes
        .saturating_sub(app.sys_info.disk_free_bytes);
    let disk_ratio = if app.sys_info.disk_total_bytes > 0 {
        (disk_used as f32 / app.sys_info.disk_total_bytes as f32).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let sys_specs_row: Element<'a, Message> = container(
        row![
            spec_col(
                p,
                "Operating System",
                format!("{} ({})", app.sys_info.os_name, app.sys_info.os_build)
            ),
            spec_col(p, "Host", app.sys_info.hostname.clone()),
            spec_col(p, "Processor", format!("{} threads", app.sys_info.cpu_threads)),
            spec_col(p, "Memory", format!("{:.1} GB", app.sys_info.ram_total_gb)),
        ]
        .spacing(16),
    )
    .padding([13, 16])
    .into();

    let disk_row: Element<'a, Message> = container(
        column![
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
            ],
            progress_bar(0.0..=1.0, disk_ratio)
                .girth(Length::Fixed(5.0))
                .style(theme::progress_style(p, p.accent)),
        ]
        .spacing(7),
    )
    .padding([13, 16])
    .into();

    let winget_row: Element<'a, Message> = container(
        row![
            column![
                text("Windows Package Manager (WinGet)")
                    .font(FONT_MEDIUM)
                    .size(13)
                    .color(p.text_primary),
                if app.sys_info.winget_available {
                    status_dot(
                        p.success,
                        Some(&app.sys_info.winget_version),
                        p,
                    )
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
                    Some(motion::spinner(app.motion.spinner_phase, 13.0, p.accent, p.hairline))
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
    .padding([12, 16])
    .into();

    let system_section = column![
        section_heading(p, s.system_snapshot, None),
        inset_group(p, vec![sys_specs_row, disk_row, winget_row]),
    ]
    .spacing(2);

    // 2. Appearance inset group
    let appearance_section = column![
        section_heading(p, s.appearance_section, None),
        inset_group(
            p,
            vec![
                container(
                    row![
                        text(s.theme_label)
                            .font(FONT_MEDIUM)
                            .size(13)
                            .color(p.text_primary)
                            .width(Length::Fill),
                        segmented_control(
                            p,
                            vec![
                                (
                                    s.theme_system,
                                    app.config.theme == ThemeMode::System,
                                    Message::SetTheme(ThemeMode::System),
                                ),
                                (
                                    s.theme_dark,
                                    app.config.theme == ThemeMode::Dark,
                                    Message::SetTheme(ThemeMode::Dark),
                                ),
                                (
                                    s.theme_light,
                                    app.config.theme == ThemeMode::Light,
                                    Message::SetTheme(ThemeMode::Light),
                                ),
                            ],
                        ),
                    ]
                    .align_y(Vertical::Center),
                )
                .padding([11, 16])
                .into(),
                container(
                    row![
                        text(s.language_label)
                            .font(FONT_MEDIUM)
                            .size(13)
                            .color(p.text_primary)
                            .width(Length::Fill),
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
                    ]
                    .align_y(Vertical::Center),
                )
                .padding([11, 16])
                .into(),
            ],
        ),
    ]
    .spacing(2);

    // 3. Behavior inset group
    let behavior_section = column![
        section_heading(p, s.behavior_section, None),
        inset_group(
            p,
            vec![
                container(
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
                                .width(Length::Fixed(24.0))
                                .align_x(iced::alignment::Horizontal::Center),
                            button(text("+").font(FONT_SEMIBOLD).size(13))
                                .padding([4, 10])
                                .style(theme::btn_secondary(p))
                                .on_press(Message::SetMaxConcurrency(
                                    (app.config.max_concurrent_installs + 1).min(6)
                                )),
                        ]
                        .spacing(6)
                        .align_y(Vertical::Center),
                    ]
                    .align_y(Vertical::Center),
                )
                .padding([11, 16])
                .into(),
                container(
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
                            .size(20)
                            .style(theme::toggler_style(p))
                            .on_toggle(Message::SetAutoCleanup),
                    ]
                    .align_y(Vertical::Center),
                )
                .padding([11, 16])
                .into(),
                container(
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
                            .size(20)
                            .style(theme::toggler_style(p))
                            .on_toggle(Message::SetPreferQuietUninstall),
                    ]
                    .align_y(Vertical::Center),
                )
                .padding([11, 16])
                .into(),
            ],
        ),
    ]
    .spacing(2);

    // 4. Portable Profile & About
    let portable_section = column![
        section_heading(p, s.portable_section, None),
        inset_group(
            p,
            vec![
                container(
                    row![
                        primary_button(
                            p,
                            s.export_profile.into(),
                            Some(icons::save(13.0, Color::WHITE).into()),
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
                .padding([12, 16])
                .into(),
                container(
                    row![
                        text("Reforge v1.0.0  ·  Native Rust  ·  Static CRT  ·  MIT License")
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
                .padding([10, 16])
                .into(),
            ],
        ),
    ]
    .spacing(2);

    let body = column![
        system_section,
        appearance_section,
        behavior_section,
        portable_section
    ]
    .spacing(16)
    .padding(Padding {
        top: 14.0 + app.motion.tab_offset_y(),
        right: 24.0,
        bottom: 24.0,
        left: 24.0,
    });

    let scroll = scrollable(body)
        .height(Length::Fill)
        .style(theme::scrollable_style(p));

    column![header, h_divider(p), scroll]
        .height(Length::Fill)
        .into()
}

fn spec_col<'a>(p: Palette, label: &'a str, val: String) -> Element<'a, Message> {
    column![
        text(label).font(FONT_REGULAR).size(11).color(p.text_muted),
        text(val)
            .font(FONT_SEMIBOLD)
            .size(12.5)
            .color(p.text_primary),
    ]
    .spacing(2)
    .width(Length::Fill)
    .into()
}
