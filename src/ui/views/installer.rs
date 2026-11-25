use crate::app::{
    App, InstallTaskState, InstallerFilter, InstallerSort, Message, INSTALLER_PAGE_SIZE,
    INSTALLER_ROW_HEIGHT,
};
use crate::core::catalog::{catalog, AppCategory, CatalogApp, PresetKind};
use crate::core::installer::{InstallStage, WingetSearchResult};
use crate::core::winapi::{format_bytes, format_speed};
use crate::i18n::{tr, Lang};
use crate::ui::components::{
    app_logo, checkbox_box, clean_list, destructive_subtle_button, empty_state, h_divider,
    hover_row, page_header, primary_button, search_bar, secondary_button, segmented_control,
    soft_panel, status_dot,
};
use crate::ui::icons;
use crate::ui::motion;
use crate::ui::theme::{self, Palette, FONT_MEDIUM, FONT_REGULAR, FONT_SEMIBOLD};
use iced::alignment::Vertical;
use iced::widget::{button, column, container, progress_bar, row, scrollable, text, Space};
use iced::{Element, Length, Padding};

fn format_count(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}

fn truncate_str(s: &str, max_chars: usize) -> String {
    let mut chars = s.chars();
    let prefix: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{prefix}...")
    } else {
        prefix
    }
}

fn cat_label(cat: AppCategory, lang: Lang) -> &'static str {
    let s = tr(lang);
    match cat {
        AppCategory::All => s.cat_all,
        AppCategory::Ai => s.cat_ai,
        AppCategory::Dependencies => s.cat_deps,
        AppCategory::Fixes => s.cat_fixes,
        AppCategory::Gaming => s.cat_gaming,
        AppCategory::Browsers => s.cat_browsers,
        AppCategory::Communication => s.cat_comm,
        AppCategory::Development => s.cat_dev,
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
            Some(motion::spinner(
                app.motion.spinner_phase,
                13.0,
                p.accent_fg,
                p.hairline,
            ))
        } else {
            Some(
                icons::download(
                    14.0,
                    if can_install { p.accent_fg } else { p.text_muted },
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
        filter_seg,
        sort_seg,
    ]
    .spacing(8)
    .align_y(Vertical::Center);

    let mut cat_row = row![].spacing(2).align_y(Vertical::Center);
    for &cat in AppCategory::ALL {
        let active = app.installer_category == cat;
        let key = format!("cat_{cat:?}");
        let ht = app.motion.hover_level(&key);
        cat_row = cat_row.push(
            iced::widget::mouse_area(
                button(
                    text(cat_label(cat, app.config.lang))
                        .font(if active { FONT_SEMIBOLD } else { FONT_MEDIUM })
                        .size(11.5)
                        .color(if active { p.accent } else { p.text_secondary }),
                )
                .padding([5, 7])
                .style(theme::btn_nav_animated(p, active, ht))
                .on_press(Message::InstallerCategoryChanged(cat)),
            )
            .on_enter(Message::HoverEnter(key.clone()))
            .on_exit(Message::HoverExit(key)),
        );
    }

    let preset_link = |kind: PresetKind, label: &'a str| {
        button(
            text(label)
                .font(FONT_MEDIUM)
                .size(11.5)
                .color(p.text_muted),
        )
        .padding([4, 6])
        .style(theme::btn_ghost(p))
        .on_press(Message::ApplyInstallPreset(kind))
    };

    let cat_and_presets = row![
        cat_row,
        Space::new().width(Length::Fill),
        preset_link(PresetKind::Ai, s.preset_ai),
        preset_link(PresetKind::Dependencies, s.preset_deps),
        preset_link(PresetKind::Essentials, s.preset_essentials),
        preset_link(PresetKind::Gaming, s.preset_gaming),
        preset_link(PresetKind::Developer, s.preset_developer),
    ]
    .spacing(2)
    .align_y(Vertical::Center);

    let toolbar = container(column![search_strip, cat_and_presets].spacing(9)).padding(Padding {
        top: 0.0,
        right: 24.0,
        bottom: 12.0,
        left: 24.0,
    });

    // ── CRISP 50-ITEM PAGINATED UNIFIED CATALOG (THICK SCROLLBAR, 0ms LAG) ───
    let curated_len = app.curated_matched.len();
    let include_winget = app.installer_include_winget();
    let winget_len = if include_winget {
        app.winget_matched_indices.len()
    } else {
        0
    };
    let total_rows = curated_len + winget_len;
    let total_pages = (total_rows.saturating_sub(1) / INSTALLER_PAGE_SIZE) + 1;
    let page = app.installer_page.clamp(1, total_pages.max(1));

    let mut page_col = column![header, toolbar, h_divider(p)].height(Length::Fill);

    // Keep active installation queue pinned cleanly above the list
    if !app.install_queue.is_empty() {
        page_col = page_col
            .push(container(view_install_queue(app, p)).padding(Padding {
                top: 10.0,
                right: 20.0,
                bottom: 10.0,
                left: 20.0,
            }))
            .push(h_divider(p));
    }

    let scroll_body: Element<'a, Message> = if total_rows == 0 {
        container(if app.winget_searching {
            empty_state(
                p,
                motion::spinner(app.motion.spinner_phase, 22.0, p.accent, p.hairline),
                s.scanning,
                "Indexing 16,000+ Microsoft WinGet packages...",
            )
        } else {
            empty_state(
                p,
                icons::search(22.0, p.text_muted).into(),
                s.empty_search,
                "",
            )
        })
        .padding([12, 20])
        .into()
    } else {
        let start = (page - 1) * INSTALLER_PAGE_SIZE;
        let end = (start + INSTALLER_PAGE_SIZE).min(total_rows);

        let mut rows: Vec<Element<'a, Message>> = Vec::with_capacity(end.saturating_sub(start));
        for idx in start..end {
            if idx < curated_len {
                rows.push(view_catalog_row(app, app.curated_matched[idx], p));
            } else {
                let wg_pos = idx - curated_len;
                if let Some(&full_idx) = app.winget_matched_indices.get(wg_pos) {
                    if let Some(wg_item) = app.winget_full_index.get(full_idx) {
                        rows.push(view_winget_row(app, wg_item, p));
                    }
                }
            }
        }

        let footer: Element<'a, Message> = if total_pages > 1 {
            let prev_lbl = if app.config.lang == Lang::PtBr {
                "← Anterior"
            } else {
                "← Previous"
            };
            let next_lbl = if app.config.lang == Lang::PtBr {
                "Próxima →"
            } else {
                "Next →"
            };
            let page_meta = if app.config.lang == Lang::PtBr {
                format!(
                    "Página {} de {}  ·  {} pacotes",
                    page,
                    format_count(total_pages),
                    format_count(total_rows)
                )
            } else {
                format!(
                    "Page {} of {}  ·  {} packages",
                    page,
                    format_count(total_pages),
                    format_count(total_rows)
                )
            };

            container(
                row![
                    secondary_button(
                        p,
                        prev_lbl.into(),
                        None,
                        if page > 1 {
                            Some(Message::InstallerPageChanged(page - 1))
                        } else {
                            None
                        },
                    ),
                    Space::new().width(Length::Fill),
                    text(page_meta)
                        .font(FONT_REGULAR)
                        .size(12)
                        .color(p.text_muted),
                    Space::new().width(Length::Fill),
                    secondary_button(
                        p,
                        next_lbl.into(),
                        None,
                        if page < total_pages {
                            Some(Message::InstallerPageChanged(page + 1))
                        } else {
                            None
                        },
                    ),
                ]
                .align_y(Vertical::Center),
            )
            .width(Length::Fill)
            .padding(Padding {
                top: 14.0,
                right: 6.0,
                bottom: 4.0,
                left: 6.0,
            })
            .into()
        } else {
            let footer_txt = if app.winget_searching {
                if app.config.lang == Lang::PtBr {
                    "Indexando +16.000 pacotes WinGet em segundo plano...".to_string()
                } else {
                    "Indexing 16,000+ Microsoft WinGet packages in the background...".to_string()
                }
            } else if app.config.lang == Lang::PtBr {
                format!("{} pacotes encontrados", format_count(total_rows))
            } else {
                format!("{} packages matched", format_count(total_rows))
            };

            container(
                text(footer_txt)
                    .font(FONT_REGULAR)
                    .size(11.5)
                    .color(p.text_muted),
            )
            .width(Length::Fill)
            .center_x(Length::Fill)
            .padding([12, 0])
            .into()
        };

        container(column![clean_list(rows), footer].spacing(4))
            .width(Length::Fill)
            .padding(Padding {
                top: 12.0 + app.motion.tab_offset_y(),
                right: 20.0,
                bottom: 20.0,
                left: 20.0,
            })
            .into()
    };

    let scroll = scrollable(scroll_body)
        .id(app.installer_scroll_id.clone())
        .height(Length::Fill)
        .style(theme::scrollable_style(p));

    page_col.push(scroll).into()
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
    let hkey = format!("catrow_{}", item.id);
    let ht = app.motion.hover_level(&hkey);

    let main_btn = button(
        row![
            checkbox_box(selected, p),
            app_logo(app.catalog_icon(item), item.name, 26.0, p),
            column![
                text(truncate_str(item.name, 26))
                    .font(FONT_SEMIBOLD)
                    .size(13)
                    .color(p.text_primary),
                text(truncate_str(item.publisher, 28))
                    .font(FONT_REGULAR)
                    .size(11.5)
                    .color(p.text_muted),
            ]
            .spacing(1)
            .width(Length::Fixed(205.0)),
            text(truncate_str(desc, 64))
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
    .height(Length::Fixed(INSTALLER_ROW_HEIGHT))
    .padding([0, 12])
    .style(theme::btn_transparent(p))
    .on_press(Message::ToggleCatalogApp(item.id));

    let info_btn = button(icons::info(14.0, p.text_muted))
        .padding([8, 10])
        .style(theme::btn_ghost(p))
        .on_press(Message::OpenCatalogDetails(item));

    container(hover_row(
        p,
        ht,
        selected,
        Message::HoverEnter(hkey.clone()),
        Message::HoverExit(hkey),
        row![main_btn, info_btn]
            .align_y(Vertical::Center)
            .into(),
    ))
    .height(Length::Fixed(INSTALLER_ROW_HEIGHT))
    .width(Length::Fill)
    .into()
}

fn view_winget_row<'a>(
    app: &'a App,
    item: &'a WingetSearchResult,
    p: Palette,
) -> Element<'a, Message> {
    let selected = app.selected_winget_ids.contains_key(&item.id);
    let hkey = format!("wg_{}", item.id);
    let ht = app.motion.hover_level(&hkey);
    let desc_src = if item.description.is_empty() {
        item.id.as_str()
    } else {
        item.description.as_str()
    };
    let pub_src = if item.publisher.is_empty() {
        item.id.as_str()
    } else {
        item.publisher.as_str()
    };

    let inner = button(
        row![
            checkbox_box(selected, p),
            app_logo(app.winget_icon(item), &item.name, 26.0, p),
            column![
                text(truncate_str(&item.name, 26))
                    .font(FONT_SEMIBOLD)
                    .size(13)
                    .color(p.text_primary),
                text(truncate_str(pub_src, 28))
                    .font(FONT_REGULAR)
                    .size(11.5)
                    .color(p.text_muted),
            ]
            .spacing(1)
            .width(Length::Fixed(205.0)),
            text(truncate_str(desc_src, 58))
                .font(FONT_REGULAR)
                .size(12)
                .color(p.text_secondary)
                .width(Length::Fill),
            text(truncate_str(&item.version, 16))
                .font(FONT_REGULAR)
                .size(11.5)
                .color(p.text_muted),
        ]
        .spacing(12)
        .align_y(Vertical::Center),
    )
    .width(Length::Fill)
    .height(Length::Fixed(INSTALLER_ROW_HEIGHT))
    .padding([0, 12])
    .style(theme::btn_transparent(p))
    .on_press(Message::ToggleWingetApp(item.id.clone(), item.name.clone()));

    container(hover_row(
        p,
        ht,
        selected,
        Message::HoverEnter(hkey.clone()),
        Message::HoverExit(hkey),
        inner.into(),
    ))
    .height(Length::Fixed(INSTALLER_ROW_HEIGHT))
    .width(Length::Fill)
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
        scrollable(soft_panel(p, rows))
            .height(Length::Fixed(
                (app.install_queue.len() as f32 * 48.0).clamp(54.0, 170.0)
            ))
            .style(theme::scrollable_style(p)),
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
    .padding([8, 10])
    .into()
}
