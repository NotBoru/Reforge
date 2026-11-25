use crate::app::{App, InstallTaskState, InstallerFilter, InstallerSort, Message};
use crate::core::catalog::{catalog, AppCategory, CatalogApp, PresetKind};
use crate::core::installer::{InstallStage, WingetSearchResult};
use crate::core::winapi::{format_bytes, format_speed};
use crate::i18n::{tr, Lang};
use crate::ui::components::{
    app_logo, checkbox_box, clean_list, destructive_subtle_button, empty_state, h_divider,
    hover_row, page_header, primary_button, search_bar, segmented_control, soft_panel, status_dot,
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

fn featured_rank(id: &str) -> usize {
    const FEATURED_ORDER: &[&str] = &[
        "claude_desktop",
        "chatgpt",
        "cursor",
        "opencode_desktop",
        "gemini_desktop",
        "intellij_idea",
        "chrome",
        "brave",
        "discord",
        "steam",
        "hydra",
        "lunar_client",
        "cheatbreaker",
        "minecraft_launcher",
        "vcredist_2015_x64",
        "directx_runtime",
        "vscode",
        "claude_code",
        "opencode_cli",
        "ollama",
        "7zip",
        "spotify",
        "vlc",
        "ddu",
        "logitech_ghub",
        "razer_synapse4",
        "process_lasso",
        "msi_afterburner",
        "obs",
        "blender",
        "obsidian",
        "anki",
        "git",
        "powertoys",
        "everything",
    ];
    FEATURED_ORDER
        .iter()
        .position(|&x| x == id)
        .unwrap_or(FEATURED_ORDER.len())
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

    // ── UNIFIED CURATED + 16,500+ WINGET INFINITE SCROLL LIST ────────────────
    let q = app.installer_search.trim().to_lowercase();
    let mut curated_filtered: Vec<&CatalogApp> = catalog()
        .iter()
        .filter(|item| {
            // When user types a search query, search across ALL categories so nothing is hidden!
            let cat_ok = !q.is_empty()
                || app.installer_category == AppCategory::All
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
        curated_filtered.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    } else if app.installer_category == AppCategory::All && q.is_empty() {
        curated_filtered.sort_by_key(|a| featured_rank(a.id));
    }

    let include_winget_tail = app.installer_filter != InstallerFilter::Installed
        && (app.installer_category == AppCategory::All || !q.is_empty());

    let winget_available = if include_winget_tail {
        app.winget_matched_indices.len()
    } else {
        0
    };
    let total_available = curated_filtered.len() + winget_available;
    let limit = app.installer_visible_limit.max(60);

    let mut rows: Vec<Element<'a, Message>> = Vec::with_capacity(limit.min(total_available));

    for item in curated_filtered.iter().take(limit) {
        rows.push(view_catalog_row(app, item, p));
    }

    if include_winget_tail && rows.len() < limit {
        let remaining = limit - rows.len();
        for &wg_idx in app.winget_matched_indices.iter().take(remaining) {
            if let Some(wg_item) = app.winget_full_index.get(wg_idx) {
                rows.push(view_winget_row(app, wg_item, p));
            }
        }
    }

    let shown_count = rows.len();

    let mut body = column![].spacing(12).padding(Padding {
        top: 12.0 + app.motion.tab_offset_y(),
        right: 20.0,
        bottom: 28.0,
        left: 20.0,
    });

    if !app.install_queue.is_empty() {
        body = body.push(view_install_queue(app, p));
    }

    if rows.is_empty() {
        if app.winget_searching {
            body = body.push(empty_state(
                p,
                motion::spinner(app.motion.spinner_phase, 22.0, p.accent, p.hairline),
                s.scanning,
                "Indexing 16,000+ Microsoft WinGet packages...",
            ));
        } else {
            body = body.push(empty_state(
                p,
                icons::search(22.0, p.text_muted).into(),
                s.empty_search,
                "",
            ));
        }
    } else {
        body = body.push(clean_list(rows));

        // Subtle infinite-scroll status footer
        let mut foot = row![].spacing(10).align_y(Vertical::Center);
        if app.winget_searching {
            foot = foot
                .push(motion::spinner(
                    app.motion.spinner_phase,
                    12.5,
                    p.accent,
                    p.hairline,
                ))
                .push(
                    text(if app.config.lang == Lang::PtBr {
                        "Indexando +16.000 pacotes WinGet em segundo plano..."
                    } else {
                        "Indexing 16,000+ WinGet packages in the background..."
                    })
                    .font(FONT_REGULAR)
                    .size(11.5)
                    .color(p.text_muted),
                );
        } else if shown_count < total_available {
            let lbl = if app.config.lang == Lang::PtBr {
                format!(
                    "Exibindo {} de {} pacotes  ·  Role para carregar mais",
                    format_count(shown_count),
                    format_count(total_available)
                )
            } else {
                format!(
                    "Showing {} of {} packages  ·  Scroll down to load more",
                    format_count(shown_count),
                    format_count(total_available)
                )
            };
            foot = foot.push(
                text(lbl)
                    .font(FONT_REGULAR)
                    .size(11.5)
                    .color(p.text_muted),
            );
            foot = foot.push(
                button(
                    text(if app.config.lang == Lang::PtBr {
                        "Carregar mais"
                    } else {
                        "Load more"
                    })
                    .font(FONT_MEDIUM)
                    .size(11.5)
                    .color(p.accent),
                )
                .padding([4, 8])
                .style(theme::btn_ghost(p))
                .on_press(Message::InstallerLoadMore),
            );
        } else {
            let lbl = if app.config.lang == Lang::PtBr {
                format!("{} pacotes disponíveis", format_count(total_available))
            } else {
                format!("{} packages available", format_count(total_available))
            };
            foot = foot.push(
                text(lbl)
                    .font(FONT_REGULAR)
                    .size(11.5)
                    .color(p.text_muted),
            );
        }

        body = body.push(
            container(foot)
                .width(Length::Fill)
                .center_x(Length::Fill)
                .padding([10, 0]),
        );
    }

    let scroll = scrollable(body)
        .height(Length::Fill)
        .on_scroll(Message::InstallerScrolled)
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
    let hkey = format!("catrow_{}", item.id);
    let ht = app.motion.hover_level(&hkey);

    let main_btn = button(
        row![
            checkbox_box(selected, p),
            app_logo(app.catalog_icon(item), item.name, 26.0, p),
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
            .width(Length::Fixed(205.0)),
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
    .padding([9, 12])
    .style(theme::btn_transparent(p))
    .on_press(Message::ToggleCatalogApp(item.id));

    let info_btn = button(icons::info(14.0, p.text_muted))
        .padding([8, 10])
        .style(theme::btn_ghost(p))
        .on_press(Message::OpenCatalogDetails(item));

    hover_row(
        p,
        ht,
        selected,
        Message::HoverEnter(hkey.clone()),
        Message::HoverExit(hkey),
        row![main_btn, info_btn]
            .align_y(Vertical::Center)
            .into(),
    )
}

fn view_winget_row<'a>(
    app: &'a App,
    item: &'a WingetSearchResult,
    p: Palette,
) -> Element<'a, Message> {
    let selected = app.selected_winget_ids.contains_key(&item.id);
    let hkey = format!("wg_{}", item.id);
    let ht = app.motion.hover_level(&hkey);
    let desc_line = if item.description.is_empty() {
        item.id.clone()
    } else {
        item.description.chars().take(78).collect()
    };

    let inner = button(
        row![
            checkbox_box(selected, p),
            app_logo(app.winget_icon(&item.id, &item.name), &item.name, 26.0, p),
            column![
                text(&item.name)
                    .font(FONT_SEMIBOLD)
                    .size(13)
                    .color(p.text_primary),
                text(if item.publisher.is_empty() {
                    item.id.clone()
                } else {
                    item.publisher.clone()
                })
                .font(FONT_REGULAR)
                .size(11.5)
                .color(p.text_muted),
            ]
            .spacing(1)
            .width(Length::Fixed(205.0)),
            text(desc_line)
                .font(FONT_REGULAR)
                .size(12)
                .color(p.text_secondary)
                .width(Length::Fill),
            text(&item.version)
                .font(FONT_REGULAR)
                .size(11.5)
                .color(p.text_muted),
        ]
        .spacing(12)
        .align_y(Vertical::Center),
    )
    .width(Length::Fill)
    .padding([9, 12])
    .style(theme::btn_transparent(p))
    .on_press(Message::ToggleWingetApp(item.id.clone(), item.name.clone()));

    hover_row(
        p,
        ht,
        selected,
        Message::HoverEnter(hkey.clone()),
        Message::HoverExit(hkey),
        inner.into(),
    )
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
        soft_panel(p, rows),
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
