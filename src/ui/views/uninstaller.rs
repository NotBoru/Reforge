use crate::app::{App, Message, UninstallerSort};
use crate::core::uninstaller::{InstalledApp, TraceConfidence, TraceKind};
use crate::core::winapi::format_bytes;
use crate::i18n::tr;
use crate::ui::components::{
    app_logo, checkbox_box, destructive_button, destructive_subtle_button, empty_state, h_divider,
    inset_group, page_header, search_bar, secondary_button, segmented_control, status_dot,
};
use crate::ui::icons;
use crate::ui::motion;
use crate::ui::theme::{self, Palette, FONT_MEDIUM, FONT_REGULAR, FONT_SEMIBOLD};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, checkbox, column, container, row, scrollable, text, Space};
use iced::{Color, Element, Length, Padding};

pub fn view<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    let s = tr(app.config.lang);

    if !app.trace_targets.is_empty() {
        return view_trace_review(app, p);
    }

    let sel_count = app.selected_uninstall_ids.len();
    let mut actions = row![].spacing(8).align_y(Vertical::Center);

    if sel_count > 0 {
        actions = actions
            .push(
                button(
                    text(s.clear)
                        .font(FONT_MEDIUM)
                        .size(12)
                        .color(p.text_secondary),
                )
                .padding([6, 10])
                .style(theme::btn_ghost(p))
                .on_press(Message::ClearUninstallSelection),
            )
            .push(secondary_button(
                p,
                s.scan_traces.into(),
                Some(icons::search(13.0, p.accent).into()),
                Some(Message::DeepScanSelectedApps),
            ))
            .push(destructive_button(
                p,
                format!("{} ({sel_count})", s.uninstall_selected),
                Some(icons::trash(13.0, Color::WHITE).into()),
                Some(Message::PromptBulkUninstall),
            ));
    } else {
        actions = actions.push(secondary_button(
            p,
            s.refresh.into(),
            Some(icons::refresh(13.0, p.text_secondary).into()),
            Some(Message::RefreshInstalledApps),
        ));
    }

    let header = page_header(p, s.uninstaller_title, s.uninstaller_sub, actions.into());

    let sort_seg = segmented_control(
        p,
        vec![
            (
                s.sort_name,
                app.uninstaller_sort == UninstallerSort::Name,
                Message::UninstallerSortChanged(UninstallerSort::Name),
            ),
            (
                s.sort_size,
                app.uninstaller_sort == UninstallerSort::Size,
                Message::UninstallerSortChanged(UninstallerSort::Size),
            ),
            (
                s.sort_date,
                app.uninstaller_sort == UninstallerSort::Date,
                Message::UninstallerSortChanged(UninstallerSort::Date),
            ),
        ],
    );

    let toolbar = container(
        row![
            search_bar(
                p,
                s.search_placeholder,
                &app.uninstaller_search,
                Message::UninstallerSearchChanged,
                Message::UninstallerSearchChanged(String::new()),
            ),
            sort_seg,
            checkbox(app.show_system_apps)
                .label(s.show_system)
                .size(15)
                .text_size(12)
                .style(theme::checkbox_style(p))
                .on_toggle(Message::ToggleSystemApps),
        ]
        .spacing(10)
        .align_y(Vertical::Center),
    )
    .padding(Padding {
        top: 0.0,
        right: 24.0,
        bottom: 12.0,
        left: 24.0,
    });

    let q = app.uninstaller_search.trim().to_lowercase();
    let mut filtered: Vec<&InstalledApp> = app
        .installed_apps
        .iter()
        .filter(|a| {
            q.is_empty()
                || a.name.to_lowercase().contains(&q)
                || a.publisher.to_lowercase().contains(&q)
        })
        .collect();

    match app.uninstaller_sort {
        UninstallerSort::Name => {
            filtered.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        }
        UninstallerSort::Size => {
            filtered.sort_by(|a, b| b.estimated_size_bytes.cmp(&a.estimated_size_bytes));
        }
        UninstallerSort::Date => {
            filtered.sort_by(|a, b| b.install_date.cmp(&a.install_date));
        }
    }

    let mut body = column![].spacing(8).padding(Padding {
        top: 12.0 + app.motion.tab_offset_y(),
        right: 24.0,
        bottom: 24.0,
        left: 24.0,
    });

    if filtered.is_empty() {
        body = body.push(empty_state(
            p,
            icons::trash(22.0, p.text_muted).into(),
            s.empty_search,
            "",
        ));
    } else {
        // Quiet Finder/Linear table column header
        let col_header = container(
            row![
                Space::new().width(Length::Fixed(56.0)),
                text(format!("Application ({})", filtered.len()))
                    .font(FONT_MEDIUM)
                    .size(11)
                    .color(p.text_muted)
                    .width(Length::Fill),
                text(s.sort_size)
                    .font(FONT_MEDIUM)
                    .size(11)
                    .color(p.text_muted)
                    .width(Length::Fixed(86.0))
                    .align_x(Horizontal::Right),
                text(s.sort_date)
                    .font(FONT_MEDIUM)
                    .size(11)
                    .color(p.text_muted)
                    .width(Length::Fixed(92.0))
                    .align_x(Horizontal::Right),
                Space::new().width(Length::Fixed(34.0)),
            ]
            .spacing(12)
            .align_y(Vertical::Center),
        )
        .padding([0, 14]);

        let rows: Vec<Element<'a, Message>> = filtered
            .into_iter()
            .map(|inst| view_installed_row(app, inst, p))
            .collect();

        body = body.push(col_header).push(inset_group(p, rows));
    }

    let scroll = scrollable(body)
        .height(Length::Fill)
        .style(theme::scrollable_style(p));

    column![header, toolbar, h_divider(p), scroll]
        .height(Length::Fill)
        .into()
}

fn view_installed_row<'a>(app: &'a App, inst: &'a InstalledApp, p: Palette) -> Element<'a, Message> {
    let selected = app.selected_uninstall_ids.contains(&inst.id);
    let size_str = if inst.estimated_size_bytes > 0 {
        format_bytes(inst.estimated_size_bytes)
    } else {
        "—".into()
    };
    let meta = match (inst.publisher.is_empty(), inst.version.is_empty()) {
        (false, false) => format!("{}  ·  {}", inst.publisher, inst.version),
        (false, true) => inst.publisher.clone(),
        (true, false) => inst.version.clone(),
        (true, true) => inst.reg_root.as_str().to_string(),
    };

    let row_btn = button(
        row![
            checkbox_box(selected, p),
            app_logo(app.installed_icon(&inst.id), &inst.name, 28.0, p),
            column![
                text(&inst.name)
                    .font(FONT_SEMIBOLD)
                    .size(13)
                    .color(p.text_primary),
                text(meta)
                    .font(FONT_REGULAR)
                    .size(11.5)
                    .color(p.text_muted),
            ]
            .spacing(1)
            .width(Length::Fill),
            text(size_str)
                .font(FONT_MEDIUM)
                .size(12)
                .color(p.text_secondary)
                .width(Length::Fixed(86.0))
                .align_x(Horizontal::Right),
            text(if inst.install_date.is_empty() {
                "—"
            } else {
                &inst.install_date
            })
            .font(FONT_REGULAR)
            .size(11.5)
            .color(p.text_muted)
            .width(Length::Fixed(92.0))
            .align_x(Horizontal::Right),
        ]
        .spacing(12)
        .align_y(Vertical::Center),
    )
    .width(Length::Fill)
    .padding([9, 14])
    .style(theme::btn_list_row(p, selected))
    .on_press(Message::ToggleUninstallSelection(inst.id.clone()));

    let details_btn = button(icons::info(14.0, p.text_muted))
        .padding([8, 10])
        .style(theme::btn_ghost(p))
        .on_press(Message::OpenInstalledDetails(inst.clone()));

    row![row_btn, details_btn]
        .align_y(Vertical::Center)
        .into()
}

fn view_trace_review<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    let s = tr(app.config.lang);
    let selected_count = app.detected_traces.iter().filter(|t| t.selected).count();
    let selected_bytes: u64 = app
        .detected_traces
        .iter()
        .filter(|t| t.selected)
        .map(|t| t.size_bytes)
        .sum();

    let target_summary = if app.trace_targets.len() == 1 {
        app.trace_targets[0].name.clone()
    } else {
        format!("{} applications", app.trace_targets.len())
    };

    let actions = row![
        secondary_button(p, s.back.into(), None, Some(Message::CloseTraceReview)),
        destructive_subtle_button(
            p,
            format!(
                "{} ({} · {})",
                s.clean_traces_btn,
                selected_count,
                format_bytes(selected_bytes)
            ),
            Some(icons::trash(13.0, p.danger).into()),
            if selected_count > 0 && !app.traces_scanning {
                Some(Message::DeleteSelectedTraces)
            } else {
                None
            },
        ),
    ]
    .spacing(8)
    .align_y(Vertical::Center);

    let header = container(
        row![
            column![
                text(s.traces_title)
                    .font(FONT_SEMIBOLD)
                    .size(20)
                    .color(p.text_primary),
                text(target_summary)
                    .font(FONT_REGULAR)
                    .size(12)
                    .color(p.text_secondary),
            ]
            .spacing(2),
            Space::new().width(Length::Fill),
            actions,
        ]
        .align_y(Vertical::Center),
    )
    .padding(Padding {
        top: 20.0,
        right: 24.0,
        bottom: 14.0,
        left: 24.0,
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
            text(format!("{} traces", app.detected_traces.len()))
                .font(FONT_REGULAR)
                .size(12)
                .color(p.text_muted),
        ]
        .spacing(8)
        .align_y(Vertical::Center),
    )
    .padding(Padding {
        top: 0.0,
        right: 24.0,
        bottom: 12.0,
        left: 24.0,
    });

    let mut body = column![].spacing(10).padding(Padding {
        top: 14.0,
        right: 24.0,
        bottom: 24.0,
        left: 24.0,
    });

    if app.traces_scanning {
        body = body.push(empty_state(
            p,
            motion::spinner(app.motion.spinner_phase, 22.0, p.accent, p.hairline),
            s.scanning,
            "Scanning filesystem, Registry, shortcuts, tasks & services",
        ));
    } else if app.detected_traces.is_empty() {
        body = body.push(empty_state(
            p,
            icons::check(22.0, p.success).into(),
            s.traces_empty,
            "",
        ));
    } else {
        let rows: Vec<Element<'a, Message>> = app
            .detected_traces
            .iter()
            .map(|trace| {
                let (kind_icon, kind_lbl): (Element<'a, Message>, &str) = match trace.kind {
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
                let conf = match trace.confidence {
                    TraceConfidence::High => status_dot(p.success, Some(s.high_confidence), p),
                    TraceConfidence::Medium => status_dot(p.warning, Some(s.medium_confidence), p),
                };
                let sz = if trace.size_bytes > 0 {
                    format_bytes(trace.size_bytes)
                } else {
                    "—".into()
                };

                button(
                    row![
                        checkbox_box(trace.selected, p),
                        kind_icon,
                        text(kind_lbl)
                            .font(FONT_MEDIUM)
                            .size(11.5)
                            .color(p.text_muted)
                            .width(Length::Fixed(60.0)),
                        text(display_path)
                            .font(FONT_REGULAR)
                            .size(12)
                            .color(p.text_primary)
                            .width(Length::Fill),
                        conf,
                        text(sz)
                            .font(FONT_MEDIUM)
                            .size(12)
                            .color(p.text_secondary)
                            .width(Length::Fixed(72.0))
                            .align_x(Horizontal::Right),
                    ]
                    .spacing(10)
                    .align_y(Vertical::Center),
                )
                .width(Length::Fill)
                .padding([9, 14])
                .style(theme::btn_list_row(p, trace.selected))
                .on_press(Message::ToggleTrace(trace.id))
                .into()
            })
            .collect();

        body = body.push(inset_group(p, rows));
    }

    let scroll = scrollable(body)
        .height(Length::Fill)
        .style(theme::scrollable_style(p));

    column![header, toolbar, h_divider(p), scroll]
        .height(Length::Fill)
        .into()
}
