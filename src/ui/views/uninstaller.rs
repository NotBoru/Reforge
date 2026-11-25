use crate::app::{App, Message};
use crate::core::uninstaller::{InstalledApp, TraceConfidence, TraceKind};
use crate::core::winapi::format_bytes;
use crate::i18n::tr;
use crate::ui::components::{
    danger_button, empty_state, h_divider, page_header, search_bar, secondary_button, tag_pill,
};
use crate::ui::icons;
use crate::ui::theme::{self, Palette, FONT_MEDIUM, FONT_REGULAR, FONT_SEMIBOLD};
use iced::alignment::Vertical;
use iced::widget::{button, checkbox, column, container, row, scrollable, text, Space};
use iced::{Color, Element, Length, Padding};

pub fn view<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    let s = tr(app.config.lang);

    // If a Revo-style Deep Trace Scan review is open, show the dedicated clean trace panel!
    if let Some(target) = &app.trace_target_app {
        return view_trace_review(app, target, p);
    }

    let actions = row![
        checkbox(app.show_system_apps)
            .label(s.show_system_components)
            .size(16)
            .text_size(12)
            .style(theme::checkbox_style(p))
            .on_toggle(Message::ToggleSystemApps),
        secondary_button(
            p,
            s.refresh.into(),
            Some(icons::refresh(14.0, p.text_secondary).into()),
            Some(Message::RefreshInstalledApps),
        ),
    ]
    .spacing(12)
    .align_y(Vertical::Center);

    let header = page_header(p, s.uninstaller_title, s.uninstaller_sub, actions.into());

    let q = app.uninstaller_search.trim().to_lowercase();
    let filtered: Vec<&InstalledApp> = app
        .installed_apps
        .iter()
        .filter(|a| {
            q.is_empty()
                || a.name.to_lowercase().contains(&q)
                || a.publisher.to_lowercase().contains(&q)
        })
        .collect();

    let toolbar = container(
        row![
            search_bar(
                p,
                s.search_placeholder,
                &app.uninstaller_search,
                Message::UninstallerSearchChanged,
                Message::UninstallerSearchChanged(String::new()),
            ),
            tag_pill(
                format!("{} {}", filtered.len(), s.apps_found),
                p.bg_surface,
                p.text_secondary,
            ),
        ]
        .spacing(12)
        .align_y(Vertical::Center),
    )
    .padding(Padding {
        top: 0.0,
        right: 26.0,
        bottom: 14.0,
        left: 26.0,
    });

    let mut list = column![].spacing(6).padding(Padding {
        top: 14.0,
        right: 26.0,
        bottom: 24.0,
        left: 26.0,
    });

    if filtered.is_empty() {
        list = list.push(empty_state(
            p,
            icons::trash(22.0, p.text_muted).into(),
            s.empty_search,
            "",
        ));
    } else {
        for app_item in filtered {
            let busy = app.uninstalling_id.as_deref() == Some(app_item.id.as_str());
            let size_str = if app_item.estimated_size_bytes > 0 {
                format_bytes(app_item.estimated_size_bytes)
            } else {
                "—".into()
            };
            let meta = if !app_item.publisher.is_empty() && !app_item.version.is_empty() {
                format!("{}  ·  v{}", app_item.publisher, app_item.version)
            } else if !app_item.publisher.is_empty() {
                app_item.publisher.clone()
            } else if !app_item.version.is_empty() {
                format!("v{}", app_item.version)
            } else {
                app_item.reg_root.as_str().to_string()
            };

            let mut row_actions = row![].spacing(6).align_y(Vertical::Center);
            if !app_item.install_location.is_empty() {
                row_actions = row_actions.push(
                    button(icons::folder(14.0, p.text_secondary))
                        .width(Length::Fixed(30.0))
                        .padding([6, 8])
                        .style(theme::btn_ghost(p))
                        .on_press(Message::OpenPath(app_item.install_location.clone())),
                );
            } else {
                row_actions = row_actions.push(Space::new().width(Length::Fixed(30.0)));
            }
            row_actions = row_actions
                .push(
                    button(
                        row![
                            icons::search(13.0, p.accent),
                            text(s.deep_scan_btn)
                                .font(FONT_MEDIUM)
                                .size(12)
                                .color(p.text_primary),
                        ]
                        .spacing(5)
                        .align_y(Vertical::Center),
                    )
                    .padding([6, 10])
                    .style(theme::btn_secondary(p))
                    .on_press(Message::DeepScanApp(app_item.clone())),
                )
                .push(
                    button(
                        text(if busy { s.working } else { s.uninstall_btn })
                            .font(FONT_MEDIUM)
                            .size(12),
                    )
                    .padding([6, 11])
                    .style(theme::btn_danger(p))
                    .on_press_maybe(if busy {
                        None
                    } else {
                        Some(Message::UninstallApp(app_item.clone()))
                    }),
                )
                .push(
                    button(
                        text(s.force_btn)
                            .font(FONT_MEDIUM)
                            .size(11)
                            .color(p.text_muted),
                    )
                    .padding([6, 8])
                    .style(theme::btn_ghost(p))
                    .on_press(Message::ForceRemoveApp(app_item.clone())),
                );

            let card = container(
                row![
                    column![
                        text(&app_item.name)
                            .font(FONT_SEMIBOLD)
                            .size(13)
                            .color(p.text_primary),
                        text(meta)
                            .font(FONT_REGULAR)
                            .size(11)
                            .color(p.text_secondary),
                    ]
                    .spacing(2)
                    .width(Length::FillPortion(5)),
                    text(size_str)
                        .font(FONT_MEDIUM)
                        .size(12)
                        .color(p.text_secondary)
                        .width(Length::Fixed(76.0)),
                    text(&app_item.install_date)
                        .font(FONT_REGULAR)
                        .size(11)
                        .color(p.text_muted)
                        .width(Length::Fixed(82.0)),
                    row_actions,
                ]
                .spacing(12)
                .align_y(Vertical::Center),
            )
            .width(Length::Fill)
            .padding([10, 14])
            .style(theme::card_style(p));

            list = list.push(card);
        }
    }

    let scroll = scrollable(list)
        .height(Length::Fill)
        .style(theme::scrollable_style(p));

    column![header, toolbar, h_divider(p), scroll]
        .height(Length::Fill)
        .into()
}

fn view_trace_review<'a>(
    app: &'a App,
    target: &'a InstalledApp,
    p: Palette,
) -> Element<'a, Message> {
    let s = tr(app.config.lang);
    let selected_count = app.detected_traces.iter().filter(|t| t.selected).count();
    let selected_bytes: u64 = app
        .detected_traces
        .iter()
        .filter(|t| t.selected)
        .map(|t| t.size_bytes)
        .sum();

    let header_actions = row![
        secondary_button(
            p,
            s.back.into(),
            None,
            Some(Message::CloseTraceReview),
        ),
        danger_button(
            p,
            format!(
                "{} ({} · {})",
                s.clean_traces_btn,
                selected_count,
                format_bytes(selected_bytes)
            ),
            Some(icons::trash(14.0, p.danger).into()),
            if selected_count > 0 && !app.traces_scanning {
                Some(Message::DeleteSelectedTraces)
            } else {
                None
            },
        ),
    ]
    .spacing(8)
    .align_y(Vertical::Center);

    let sub = format!("{} — {}", target.name, target.publisher);
    let header = container(
        row![
            column![
                text(s.traces_title)
                    .font(FONT_SEMIBOLD)
                    .size(21)
                    .color(p.text_primary),
                text(sub)
                    .font(FONT_REGULAR)
                    .size(13)
                    .color(p.text_secondary),
            ]
            .spacing(3),
            Space::new().width(Length::Fill),
            header_actions,
        ]
        .align_y(Vertical::Center),
    )
    .padding(Padding {
        top: 22.0,
        right: 26.0,
        bottom: 16.0,
        left: 26.0,
    });

    let toolbar = container(
        row![
            secondary_button(
                p,
                s.select_high_only.into(),
                None,
                Some(Message::SelectHighConfidenceTraces),
            ),
            secondary_button(
                p,
                s.select_all.into(),
                None,
                Some(Message::ToggleAllTraces(true)),
            ),
            secondary_button(
                p,
                s.deselect_all.into(),
                None,
                Some(Message::ToggleAllTraces(false)),
            ),
            Space::new().width(Length::Fill),
            tag_pill(
                format!("{} items", app.detected_traces.len()),
                p.bg_surface,
                p.text_secondary,
            ),
        ]
        .spacing(8)
        .align_y(Vertical::Center),
    )
    .padding(Padding {
        top: 0.0,
        right: 26.0,
        bottom: 14.0,
        left: 26.0,
    });

    let mut list = column![].spacing(6).padding(Padding {
        top: 14.0,
        right: 26.0,
        bottom: 24.0,
        left: 26.0,
    });

    if app.traces_scanning {
        list = list.push(empty_state(
            p,
            icons::search(22.0, p.accent).into(),
            s.scanning,
            &target.name,
        ));
    } else if app.detected_traces.is_empty() {
        list = list.push(empty_state(
            p,
            icons::check(22.0, p.success).into(),
            s.traces_empty,
            &target.name,
        ));
    } else {
        for trace in &app.detected_traces {
            let (kind_icon, kind_label): (Element<'a, Message>, &str) = match trace.kind {
                TraceKind::Directory => (icons::folder(14.0, p.accent).into(), "Folder"),
                TraceKind::File => (icons::hard_drive(14.0, p.text_secondary).into(), "File"),
                TraceKind::RegistryKey => (icons::key(14.0, p.warning).into(), "Registry"),
                TraceKind::Shortcut => (icons::link(14.0, p.text_secondary).into(), "Shortcut"),
                TraceKind::ScheduledTask => (icons::clock(14.0, p.warning).into(), "Task"),
                TraceKind::Service => (icons::cpu(14.0, p.danger).into(), "Service"),
            };

            let display_path = if let Some(root) = trace.reg_root {
                format!("{}\\{}", root.as_str(), trace.path)
            } else {
                trace.path.clone()
            };

            let conf_badge = match trace.confidence {
                TraceConfidence::High => tag_pill(s.high_confidence, p.success_soft, p.success),
                TraceConfidence::Medium => tag_pill(s.medium_confidence, p.warning_soft, p.warning),
            };

            let sz = if trace.size_bytes > 0 {
                format_bytes(trace.size_bytes)
            } else {
                "—".into()
            };

            let trace_row = button(
                row![
                    container(if trace.selected {
                        Element::from(icons::check(10.0, Color::WHITE))
                    } else {
                        Space::new()
                            .width(Length::Fixed(10.0))
                            .height(Length::Fixed(10.0))
                            .into()
                    })
                    .padding(3)
                    .style(theme::badge_style(
                        if trace.selected { p.accent } else { p.bg_input },
                        Color::WHITE,
                        if trace.selected { p.accent } else { p.border },
                    )),
                    kind_icon,
                    tag_pill(kind_label, p.bg_input, p.text_secondary),
                    text(display_path)
                        .font(FONT_REGULAR)
                        .size(12)
                        .color(p.text_primary)
                        .width(Length::Fill),
                    conf_badge,
                    text(sz)
                        .font(FONT_MEDIUM)
                        .size(12)
                        .color(p.text_secondary)
                        .width(Length::Fixed(72.0)),
                ]
                .spacing(10)
                .align_y(Vertical::Center),
            )
            .width(Length::Fill)
            .padding([9, 12])
            .style(theme::btn_card_selectable(p, trace.selected))
            .on_press(Message::ToggleTrace(trace.id));

            list = list.push(trace_row);
        }
    }

    let scroll = scrollable(list)
        .height(Length::Fill)
        .style(theme::scrollable_style(p));

    column![header, toolbar, h_divider(p), scroll]
        .height(Length::Fill)
        .into()
}
