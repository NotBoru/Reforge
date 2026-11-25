use crate::app::{App, InstallTaskState, Message};
use crate::core::catalog::{catalog, AppCategory, CatalogApp, PresetKind};
use crate::core::installer::InstallStage;
use crate::core::winapi::{format_bytes, format_speed};
use crate::i18n::{tr, Lang};
use crate::ui::components::{
    empty_state, h_divider, page_header, primary_button, search_bar, secondary_button, tag_pill,
};
use crate::ui::icons;
use crate::ui::theme::{self, Palette, FONT_MEDIUM, FONT_REGULAR, FONT_SEMIBOLD};
use iced::alignment::Vertical;
use iced::widget::{button, column, container, progress_bar, row, scrollable, text, Space};
use iced::{Color, Element, Length, Padding};

fn category_icon<'a>(cat: AppCategory, color: Color) -> Element<'a, Message> {
    match cat {
        AppCategory::All => icons::layers(12.0, color).into(),
        AppCategory::Browsers => icons::globe(12.0, color).into(),
        AppCategory::Communication => icons::message_square(12.0, color).into(),
        AppCategory::Development => icons::terminal(12.0, color).into(),
        AppCategory::Gaming => icons::gamepad(12.0, color).into(),
        AppCategory::Media => icons::film(12.0, color).into(),
        AppCategory::Utilities => icons::wrench(12.0, color).into(),
        AppCategory::Productivity => icons::briefcase(12.0, color).into(),
    }
}

fn category_label(cat: AppCategory, lang: Lang) -> &'static str {
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
    if total_selected > 0 && !app.installing_active {
        header_actions = header_actions.push(secondary_button(
            p,
            s.clear.into(),
            None,
            Some(Message::ClearInstallSelection),
        ));
    }
    let install_label = if app.installing_active {
        s.working.to_string()
    } else if total_selected > 0 {
        format!("{} ({})", s.install_selected, total_selected)
    } else {
        s.install_selected.to_string()
    };
    let install_enabled = total_selected > 0 && !app.installing_active;
    header_actions = header_actions.push(primary_button(
        p,
        install_label,
        Some(
            icons::download(
                15.0,
                if install_enabled { Color::WHITE } else { p.text_muted },
            )
            .into(),
        ),
        if install_enabled {
            Some(Message::StartInstallBatch)
        } else {
            None
        },
    ));

    let header = page_header(p, s.installer_title, s.installer_sub, header_actions.into());

    // Presets strip + Search + Winget live search
    let preset_btn = |kind: PresetKind, label: &'a str, icon: Element<'a, Message>| {
        button(
            row![icon, text(label).font(FONT_MEDIUM).size(12)]
                .spacing(6)
                .align_y(Vertical::Center),
        )
        .padding([6, 11])
        .style(theme::btn_secondary(p))
        .on_press(Message::ApplyInstallPreset(kind))
    };

    let presets_bar = row![
        text(s.presets_label)
            .font(FONT_MEDIUM)
            .size(12)
            .color(p.text_muted),
        preset_btn(
            PresetKind::Essentials,
            s.preset_essentials,
            icons::sparkles(13.0, p.accent).into()
        ),
        preset_btn(
            PresetKind::Developer,
            s.preset_developer,
            icons::terminal(13.0, p.text_secondary).into()
        ),
        preset_btn(
            PresetKind::Gaming,
            s.preset_gaming,
            icons::gamepad(13.0, p.text_secondary).into()
        ),
        preset_btn(
            PresetKind::Creator,
            s.preset_creator,
            icons::film(13.0, p.text_secondary).into()
        ),
    ]
    .spacing(8)
    .align_y(Vertical::Center);

    let search_row = row![
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
            Some(icons::globe(14.0, p.accent).into()),
            if app.installer_search.trim().len() >= 2 && !app.winget_searching {
                Some(Message::SearchWingetCatalog)
            } else {
                None
            },
        ),
    ]
    .spacing(10)
    .align_y(Vertical::Center);

    let mut cat_pills = row![].spacing(6).align_y(Vertical::Center);
    for &cat in AppCategory::ALL {
        let active = app.installer_category == cat;
        let ic_col = if active { p.accent } else { p.text_muted };
        cat_pills = cat_pills.push(
            button(
                row![
                    category_icon(cat, ic_col),
                    text(category_label(cat, app.config.lang))
                        .font(if active { FONT_SEMIBOLD } else { FONT_MEDIUM })
                        .size(12),
                ]
                .spacing(5)
                .align_y(Vertical::Center),
            )
            .padding([5, 11])
            .style(theme::btn_pill(p, active))
            .on_press(Message::InstallerCategoryChanged(cat)),
        );
    }

    let toolbar = container(column![presets_bar, search_row, cat_pills].spacing(10)).padding(
        Padding {
            top: 0.0,
            right: 26.0,
            bottom: 14.0,
            left: 26.0,
        },
    );

    // Body scrollable content
    let mut body = column![].spacing(14).padding(Padding {
        top: 14.0,
        right: 26.0,
        bottom: 24.0,
        left: 26.0,
    });

    // Live Install Queue Panel (if active or completed items exist)
    if !app.install_queue.is_empty() {
        body = body.push(view_install_queue(app, p));
    }

    // Live Winget external search results (if any)
    if !app.winget_results.is_empty() {
        body = body.push(view_winget_results(app, p));
    }

    // Filtered curated catalog grid (2 crisp balanced columns)
    let q = app.installer_search.trim().to_lowercase();
    let filtered: Vec<&CatalogApp> = catalog()
        .iter()
        .filter(|item| {
            let cat_ok = app.installer_category == AppCategory::All
                || item.category == app.installer_category;
            let q_ok = q.is_empty()
                || item.name.to_lowercase().contains(&q)
                || item.publisher.to_lowercase().contains(&q)
                || item.desc_en.to_lowercase().contains(&q)
                || item.desc_pt.to_lowercase().contains(&q)
                || item.winget_id.to_lowercase().contains(&q);
            cat_ok && q_ok
        })
        .collect();

    if filtered.is_empty() {
        body = body.push(empty_state(
            p,
            icons::search(22.0, p.text_muted).into(),
            s.empty_search,
            s.search_winget,
        ));
    } else {
        let mut grid = column![].spacing(10);
        for chunk in filtered.chunks(2) {
            let mut r = row![].spacing(10);
            for &item in chunk {
                r = r.push(view_catalog_card(app, item, p));
            }
            if chunk.len() == 1 {
                r = r.push(Space::new().width(Length::Fill));
            }
            grid = grid.push(r);
        }
        body = body.push(grid);
    }

    let scroll = scrollable(body)
        .height(Length::Fill)
        .style(theme::scrollable_style(p));

    column![header, toolbar, h_divider(p), scroll]
        .height(Length::Fill)
        .into()
}

fn view_catalog_card<'a>(app: &'a App, item: &'static CatalogApp, p: Palette) -> Element<'a, Message> {
    let s = tr(app.config.lang);
    let selected = app.selected_catalog_ids.contains(item.id);
    let is_installed = app.is_catalog_app_installed(item);
    let desc = match app.config.lang {
        Lang::En => item.desc_en,
        Lang::PtBr => item.desc_pt,
    };

    // Crisp selection checkbox indicator
    let indicator = container(if selected {
        Element::from(icons::check(11.0, Color::WHITE))
    } else {
        Space::new().width(Length::Fixed(11.0)).height(Length::Fixed(11.0)).into()
    })
    .padding(3)
    .style(theme::badge_style(
        if selected { p.accent } else { p.bg_input },
        Color::WHITE,
        if selected { p.accent } else { p.border },
    ));

    let mut badges = row![].spacing(5).align_y(Vertical::Center);
    if is_installed {
        badges = badges.push(tag_pill(s.installed, p.success_soft, p.success));
    }
    if item.direct.is_some() {
        badges = badges.push(tag_pill(s.direct_fast, p.accent_soft, p.accent));
    }

    let top_line = row![
        indicator,
        column![
            text(item.name)
                .font(FONT_SEMIBOLD)
                .size(13)
                .color(p.text_primary),
            text(item.publisher)
                .font(FONT_REGULAR)
                .size(11)
                .color(p.text_muted),
        ]
        .spacing(1),
        Space::new().width(Length::Fill),
        badges,
    ]
    .spacing(10)
    .align_y(Vertical::Center);

    let card_content = column![
        top_line,
        text(desc)
            .font(FONT_REGULAR)
            .size(12)
            .color(p.text_secondary),
    ]
    .spacing(8);

    button(card_content)
        .width(Length::Fill)
        .padding([12, 14])
        .style(theme::btn_card_selectable(p, selected))
        .on_press(Message::ToggleCatalogApp(item.id))
        .into()
}

fn view_install_queue<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    let s = tr(app.config.lang);
    let mut header = row![
        icons::zap(15.0, p.accent),
        text(s.queue_title)
            .font(FONT_SEMIBOLD)
            .size(13)
            .color(p.text_primary),
        Space::new().width(Length::Fill),
    ]
    .spacing(8)
    .align_y(Vertical::Center);

    if !app.installing_active {
        header = header.push(
            button(
                text(s.dismiss_done)
                    .font(FONT_MEDIUM)
                    .size(11)
                    .color(p.text_secondary),
            )
            .padding([4, 9])
            .style(theme::btn_ghost(p))
            .on_press(Message::ClearInstallQueue),
        );
    }

    let mut rows = column![header].spacing(10);
    for task in &app.install_queue {
        rows = rows.push(view_queue_row(task, p));
    }

    container(rows)
        .width(Length::Fill)
        .padding([14, 16])
        .style(theme::card_style(p))
        .into()
}

fn view_queue_row<'a>(task: &'a InstallTaskState, p: Palette) -> Element<'a, Message> {
    let (status_text, bar_val, bar_color) = match &task.stage {
        InstallStage::Queued => ("Queued".to_string(), 0.05, p.text_muted),
        InstallStage::Resolving => ("Resolving direct link...".to_string(), 0.12, p.accent),
        InstallStage::Downloading {
            downloaded,
            total,
            speed_bps,
        } => {
            let pct = if *total > 0 {
                (*downloaded as f32 / *total as f32).clamp(0.05, 0.90)
            } else {
                0.45
            };
            let label = if *total > 0 {
                format!(
                    "Downloading {} / {}  ·  {}",
                    format_bytes(*downloaded),
                    format_bytes(*total),
                    format_speed(*speed_bps)
                )
            } else {
                format!(
                    "Downloading {}  ·  {}",
                    format_bytes(*downloaded),
                    format_speed(*speed_bps)
                )
            };
            (label, pct, p.accent)
        }
        InstallStage::Installing { method } => (format!("Installing silently ({method})..."), 0.92, p.warning),
        InstallStage::Done { elapsed_ms, method } => (
            format!("Installed via {method} ({:.1}s)", *elapsed_ms as f64 / 1000.0),
            1.0,
            p.success,
        ),
        InstallStage::Failed(err) => (format!("Failed: {err}"), 1.0, p.danger),
    };

    column![
        row![
            text(&task.app_name)
                .font(FONT_MEDIUM)
                .size(12)
                .color(p.text_primary),
            Space::new().width(Length::Fill),
            text(status_text)
                .font(FONT_REGULAR)
                .size(11)
                .color(bar_color),
        ]
        .align_y(Vertical::Center),
        progress_bar(0.0..=1.0, bar_val)
            .girth(Length::Fixed(5.0))
            .style(theme::progress_style(p, bar_color)),
    ]
    .spacing(5)
    .into()
}

fn view_winget_results<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    let s = tr(app.config.lang);
    let mut list = column![
        row![
            icons::globe(14.0, p.accent),
            text(s.winget_results_title)
                .font(FONT_SEMIBOLD)
                .size(13)
                .color(p.text_primary),
            Space::new().width(Length::Fill),
            button(
                text(s.clear_winget_results)
                    .font(FONT_MEDIUM)
                    .size(11)
                    .color(p.text_secondary),
            )
            .padding([4, 8])
            .style(theme::btn_ghost(p))
            .on_press(Message::ClearWingetResults),
        ]
        .spacing(8)
        .align_y(Vertical::Center),
    ]
    .spacing(8);

    for item in &app.winget_results {
        let selected = app.selected_winget_ids.contains_key(&item.id);
        let row_btn = button(
            row![
                container(if selected {
                    Element::from(icons::check(10.0, Color::WHITE))
                } else {
                    Space::new()
                        .width(Length::Fixed(10.0))
                        .height(Length::Fixed(10.0))
                        .into()
                })
                .padding(3)
                .style(theme::badge_style(
                    if selected { p.accent } else { p.bg_input },
                    Color::WHITE,
                    if selected { p.accent } else { p.border },
                )),
                text(&item.name)
                    .font(FONT_MEDIUM)
                    .size(13)
                    .color(p.text_primary),
                text(&item.id)
                    .font(FONT_REGULAR)
                    .size(11)
                    .color(p.text_muted),
                Space::new().width(Length::Fill),
                tag_pill(item.version.clone(), p.bg_input, p.text_secondary),
            ]
            .spacing(10)
            .align_y(Vertical::Center),
        )
        .width(Length::Fill)
        .padding([9, 12])
        .style(theme::btn_card_selectable(p, selected))
        .on_press(Message::ToggleWingetApp(item.id.clone(), item.name.clone()));

        list = list.push(row_btn);
    }

    container(list)
        .width(Length::Fill)
        .padding([14, 16])
        .style(theme::card_style(p))
        .into()
}
