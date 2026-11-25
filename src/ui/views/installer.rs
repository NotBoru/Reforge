use crate::app::{App, InstallTaskState, InstallerFilter, InstallerSort, Message};
use crate::core::catalog::{catalog, AppCategory, CatalogApp, PresetKind};
use crate::core::installer::InstallStage;
use crate::core::winapi::{format_bytes, format_speed};
use crate::i18n::{tr, Lang};
use crate::ui::components::{
    app_logo, checkbox_box, destructive_subtle_button, empty_state, h_divider, inset_group,
    page_header, primary_button, search_bar, secondary_button, section_heading, segmented_control,
    status_dot,
};
use crate::ui::icons;
use crate::ui::motion;
use crate::ui::theme::{self, Palette, FONT_MEDIUM, FONT_REGULAR, FONT_SEMIBOLD};
use iced::alignment::Vertical;
use iced::widget::{button, column, container, progress_bar, row, scrollable, text, Space};
use iced::{Color, Element, Length, Padding};

fn cat_label(cat: AppCategory, lang: Lang) -> &'static str {
    let s = tr(lang);
    match cat {
        AppCategory::All => s.cat_all,
        AppCategory::Browsers => s.cat_browsers,
        AppCategory::Communication => s.cat_comm,
        AppCategory::Development => s.cat_dev,
        AppCategory::Gaming => s.cat_gaming,
        AppCategory::Media => s.cat_media,
        AppCategory::Utilities => s.cat_utils,
        AppCategory::Productivity => s.cat_prod,
    }
}

pub fn view<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    let s = tr(app.config.lang);
    let total_selected = app.selected_catalog_ids.len() + app.selected_winget_ids.len();

    let mut header_actions = row![].spacing(8).align_y(Vertical::Center);
    if app.installing_active {
        header_actions = header_actions.push(destructive_subtle_button(
            p,
            s.cancel_batch.into(),
            None,
            Some(Message::CancelInstallBatch),
        ));
    } else if total_selected > 0 {
        header_actions = header_actions.push(
            button(
                text(s.clear)
                    .font(FONT_MEDIUM)
                    .size(12)
                    .color(p.text_secondary),
            )
            .padding([6, 10])
            .style(theme::btn_ghost(p))
            .on_press(Message::ClearInstallSelection),
        );
    }

    let can_install = total_selected > 0 && !app.installing_active;
    let cta_label = if app.installing_active {
        s.working.into()
    } else if total_selected > 0 {
        format!("{} ({total_selected})", s.install_selected)
    } else {
        s.install_selected.into()
    };
    header_actions = header_actions.push(primary_button(
        p,
        cta_label,
        if app.installing_active {
            Some(motion::spinner(app.motion.spinner_phase, 13.0, Color::WHITE, p.hairline))
        } else {
            Some(
                icons::download(
                    14.0,
                    if can_install { Color::WHITE } else { p.text_muted },
                )
                .into(),
            )
        },
        if can_install {
            Some(Message::StartInstallBatch)
        } else {
            None
        },
    ));

    let header = page_header(p, s.installer_title, s.installer_sub, header_actions.into());

    // Unified clean toolbar: Search + WinGet query + Filter segment + Sort segment
    let filter_seg = segmented_control(
        p,
        vec![
            (
                s.filter_all,
                app.installer_filter == InstallerFilter::All,
                Message::InstallerFilterChanged(InstallerFilter::All),
            ),
            (
                s.filter_not_installed,
                app.installer_filter == InstallerFilter::NotInstalled,
                Message::InstallerFilterChanged(InstallerFilter::NotInstalled),
            ),
            (
                s.filter_installed,
                app.installer_filter == InstallerFilter::Installed,
                Message::InstallerFilterChanged(InstallerFilter::Installed),
            ),
        ],
    );

    let sort_seg = segmented_control(
        p,
        vec![
            (
                s.sort_featured,
                app.installer_sort == InstallerSort::Featured,
                Message::InstallerSortChanged(InstallerSort::Featured),
            ),
            (
                s.sort_name,
                app.installer_sort == InstallerSort::Name,
                Message::InstallerSortChanged(InstallerSort::Name),
            ),
        ],
    );

    let search_strip = row![
        search_bar(
            p,
            s.search_placeholder,
            &app.installer_search,
            Message::InstallerSearchChanged,
            Message::InstallerSearchChanged(String::new()),
        ),
        secondary_button(
            p,
            if app.winget_searching {
                s.scanning.into()
            } else {
                s.search_winget.into()
            },
            if app.winget_searching {
                Some(motion::spinner(app.motion.spinner_phase, 13.0, p.accent, p.hairline))
            } else {
                Some(icons::globe(13.0, p.text_secondary).into())
            },
            if app.installer_search.trim().len() >= 2 && !app.winget_searching {
                Some(Message::SearchWingetCatalog)
            } else {
                None
            },
        ),
        filter_seg,
        sort_seg,
    ]
    .spacing(8)
    .align_y(Vertical::Center);

    // Subtle category tab strip + quiet quick-select presets on the trailing edge
    let mut cat_row = row![].spacing(2).align_y(Vertical::Center);
    for &cat in AppCategory::ALL {
        let active = app.installer_category == cat;
        cat_row = cat_row.push(
            button(
                text(cat_label(cat, app.config.lang))
                    .font(if active { FONT_SEMIBOLD } else { FONT_MEDIUM })
                    .size(12)
                    .color(if active { p.accent } else { p.text_secondary }),
            )
            .padding([5, 9])
            .style(theme::btn_nav(p, active))
            .on_press(Message::InstallerCategoryChanged(cat)),
        );
    }

    let preset_link = |kind: PresetKind, label: &'a str| {
        button(
            text(label)
                .font(FONT_MEDIUM)
                .size(11.5)
                .color(p.text_muted),
        )
        .padding([4, 7])
        .style(theme::btn_ghost(p))
        .on_press(Message::ApplyInstallPreset(kind))
    };

    let sub_strip = row![
        cat_row,
        Space::new().width(Length::Fill),
        preset_link(PresetKind::Essentials, s.preset_essentials),
        preset_link(PresetKind::Developer, s.preset_developer),
        preset_link(PresetKind::Gaming, s.preset_gaming),
        preset_link(PresetKind::Creator, s.preset_creator),
    ]
    .spacing(2)
    .align_y(Vertical::Center);

    let toolbar = container(column![search_strip, sub_strip].spacing(10)).padding(Padding {
        top: 0.0,
        right: 24.0,
        bottom: 12.0,
        left: 24.0,
    });

    // Scrollable main content
    let mut body = column![].spacing(16).padding(Padding {
        top: 14.0 + app.motion.tab_offset_y(),
        right: 24.0,
        bottom: 24.0,
        left: 24.0,
    });

    if !app.install_queue.is_empty() {
        body = body.push(view_install_queue(app, p));
    }

    if !app.winget_results.is_empty() {
        body = body.push(view_winget_results(app, p));
    }

    let q = app.installer_search.trim().to_lowercase();
    let mut filtered: Vec<&CatalogApp> = catalog()
        .iter()
        .filter(|item| {
            let cat_ok = app.installer_category == AppCategory::All
                || item.category == app.installer_category;
            let inst = app.is_catalog_app_installed(item);
            let filter_ok = match app.installer_filter {
                InstallerFilter::All => true,
                InstallerFilter::Installed => inst,
                InstallerFilter::NotInstalled => !inst,
            };
            let q_ok = q.is_empty()
                || item.name.to_lowercase().contains(&q)
                || item.publisher.to_lowercase().contains(&q)
                || item.desc_en.to_lowercase().contains(&q)
                || item.desc_pt.to_lowercase().contains(&q)
                || item.winget_id.to_lowercase().contains(&q);
            cat_ok && filter_ok && q_ok
        })
        .collect();

    if app.installer_sort == InstallerSort::Name {
        filtered.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    }

    if filtered.is_empty() {
        body = body.push(empty_state(
            p,
            icons::search(22.0, p.text_muted).into(),
            s.empty_search,
            "Try searching the live WinGet catalog with the button above.",
        ));
    } else {
        // Unified Apple/Linear hairline inset table — ZERO boxed card soup!
        let rows: Vec<Element<'a, Message>> = filtered
            .into_iter()
            .map(|item| view_catalog_row(app, item, p))
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

fn view_catalog_row<'a>(
    app: &'a App,
    item: &'static CatalogApp,
    p: Palette,
) -> Element<'a, Message> {
    let s = tr(app.config.lang);
    let selected = app.selected_catalog_ids.contains(item.id);
    let is_installed = app.is_catalog_app_installed(item);
    let desc = match app.config.lang {
        Lang::En => item.desc_en,
        Lang::PtBr => item.desc_pt,
    };

    let main_click = button(
        row![
            checkbox_box(selected, p),
            app_logo(app.catalog_icon(item), item.name, 28.0, p),
            column![
                text(item.name)
                    .font(FONT_SEMIBOLD)
                    .size(13)
                    .color(p.text_primary),
                text(item.publisher)
                    .font(FONT_REGULAR)
                    .size(11.5)
                    .color(p.text_muted),
            ]
            .spacing(1)
            .width(Length::Fixed(185.0)),
            text(desc)
                .font(FONT_REGULAR)
                .size(12)
                .color(p.text_secondary)
                .width(Length::Fill),
            if is_installed {
                status_dot(p.success, Some(s.installed), p)
            } else {
                Space::new().width(Length::Fixed(68.0)).into()
            },
        ]
        .spacing(12)
        .align_y(Vertical::Center),
    )
    .width(Length::Fill)
    .padding([10, 14])
    .style(theme::btn_list_row(p, selected))
    .on_press(Message::ToggleCatalogApp(item.id));

    let info_btn = button(icons::info(14.0, p.text_muted))
        .padding([8, 10])
        .style(theme::btn_ghost(p))
        .on_press(Message::OpenCatalogDetails(item));

    row![main_click, info_btn]
        .align_y(Vertical::Center)
        .into()
}

fn view_install_queue<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    let s = tr(app.config.lang);
    let mut header_row = row![
        text(s.queue_title)
            .font(FONT_SEMIBOLD)
            .size(12)
            .color(p.text_secondary),
        Space::new().width(Length::Fill),
    ]
    .align_y(Vertical::Center);

    if !app.installing_active {
        header_row = header_row.push(
            button(
                text(s.dismiss_done)
                    .font(FONT_MEDIUM)
                    .size(11.5)
                    .color(p.text_muted),
            )
            .padding([3, 7])
            .style(theme::btn_ghost(p))
            .on_press(Message::ClearInstallQueue),
        );
    }

    let rows: Vec<Element<'a, Message>> = app
        .install_queue
        .iter()
        .map(|t| view_queue_row(app, t, p))
        .collect();

    column![
        container(header_row).padding([0, 4]),
        inset_group(p, rows),
    ]
    .spacing(6)
    .into()
}

fn view_queue_row<'a>(app: &'a App, task: &'a InstallTaskState, p: Palette) -> Element<'a, Message> {
    let (status_text, bar_val, bar_color) = match &task.stage {
        InstallStage::Queued => ("Queued".to_string(), 0.04, p.text_muted),
        InstallStage::Resolving => ("Resolving...".to_string(), 0.10, p.accent),
        InstallStage::Downloading {
            downloaded,
            total,
            speed_bps,
        } => {
            let pct = if *total > 0 {
                (*downloaded as f32 / *total as f32).clamp(0.05, 0.88)
            } else {
                0.45
            };
            let lbl = if *total > 0 {
                format!(
                    "{} / {}  ·  {}",
                    format_bytes(*downloaded),
                    format_bytes(*total),
                    format_speed(*speed_bps)
                )
            } else {
                format!(
                    "{}  ·  {}",
                    format_bytes(*downloaded),
                    format_speed(*speed_bps)
                )
            };
            (lbl, pct, p.accent)
        }
        InstallStage::Verifying => ("Verifying signature...".to_string(), 0.90, p.accent),
        InstallStage::Installing { method, signed } => {
            let sig = if *signed { " · verified" } else { "" };
            (format!("Installing ({method}{sig})..."), 0.94, p.warning)
        }
        InstallStage::Done { elapsed_ms, method } => (
            format!("Installed via {method} ({:.1}s)", *elapsed_ms as f64 / 1000.0),
            1.0,
            p.success,
        ),
        InstallStage::Cancelled => ("Cancelled".to_string(), 0.0, p.text_muted),
        InstallStage::Failed(err) => (format!("Failed: {err}"), 1.0, p.danger),
    };

    let logo_handle = catalog()
        .iter()
        .find(|c| c.id == task.app_id)
        .and_then(|c| app.catalog_icon(c));

    container(
        row![
            app_logo(logo_handle, &task.app_name, 22.0, p),
            column![
                row![
                    text(&task.app_name)
                        .font(FONT_MEDIUM)
                        .size(12.5)
                        .color(p.text_primary),
                    Space::new().width(Length::Fill),
                    text(status_text)
                        .font(FONT_REGULAR)
                        .size(11.5)
                        .color(bar_color),
                ]
                .align_y(Vertical::Center),
                progress_bar(0.0..=1.0, bar_val)
                    .girth(Length::Fixed(3.5))
                    .style(theme::progress_style(p, bar_color)),
            ]
            .spacing(5)
            .width(Length::Fill),
        ]
        .spacing(10)
        .align_y(Vertical::Center),
    )
    .padding([9, 14])
    .into()
}

fn view_winget_results<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    let s = tr(app.config.lang);
    let head = row![
        section_heading(
            p,
            s.winget_results_title,
            Some(format!("{} results", app.winget_results.len())),
        ),
        button(
            text(s.close)
                .font(FONT_MEDIUM)
                .size(11.5)
                .color(p.text_muted),
        )
        .padding([3, 8])
        .style(theme::btn_ghost(p))
        .on_press(Message::ClearWingetResults),
    ]
    .align_y(Vertical::Center);

    let rows: Vec<Element<'a, Message>> = app
        .winget_results
        .iter()
        .map(|item| {
            let selected = app.selected_winget_ids.contains_key(&item.id);
            button(
                row![
                    checkbox_box(selected, p),
                    app_logo(app.winget_icon(&item.id), &item.name, 24.0, p),
                    text(&item.name)
                        .font(FONT_MEDIUM)
                        .size(12.5)
                        .color(p.text_primary),
                    text(&item.id)
                        .font(FONT_REGULAR)
                        .size(11.5)
                        .color(p.text_muted)
                        .width(Length::Fill),
                    text(&item.version)
                        .font(FONT_REGULAR)
                        .size(11.5)
                        .color(p.text_secondary),
                ]
                .spacing(10)
                .align_y(Vertical::Center),
            )
            .width(Length::Fill)
            .padding([9, 14])
            .style(theme::btn_list_row(p, selected))
            .on_press(Message::ToggleWingetApp(item.id.clone(), item.name.clone()))
            .into()
        })
        .collect();

    column![head, inset_group(p, rows)].spacing(4).into()
}
