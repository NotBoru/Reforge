use crate::app::{App, Message};
use crate::core::cleaner::CleanCategoryId;
use crate::core::winapi::format_bytes;
use crate::i18n::{tr, Lang};
use crate::ui::components::{
    checkbox_box, clean_list, h_divider, hover_row, page_header, primary_button, secondary_button,
};
use crate::ui::icons;
use crate::ui::motion::{self, StorageSegment};
use crate::ui::theme::{self, Palette, FONT_MEDIUM, FONT_REGULAR, FONT_SEMIBOLD};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Color, Element, Length, Padding};

fn category_color(id: CleanCategoryId, p: Palette) -> Color {
    match id {
        CleanCategoryId::UserTemp => p.accent,
        CleanCategoryId::WindowsTemp => p.success,
        CleanCategoryId::BrowserCaches => Color::from_rgb(0.95, 0.65, 0.45),
        CleanCategoryId::GpuShaderCaches => Color::from_rgb(0.48, 0.85, 0.76),
        CleanCategoryId::WindowsLogsAndDumps => Color::from_rgb(0.96, 0.48, 0.48),
        CleanCategoryId::WindowsUpdateCache => Color::from_rgb(0.85, 0.72, 0.50),
        CleanCategoryId::AppCaches => Color::from_rgb(0.62, 0.78, 0.85),
        CleanCategoryId::ExplorerCaches => Color::from_rgb(0.82, 0.64, 0.56),
        CleanCategoryId::RecycleBin => Color::from_rgb(0.56, 0.56, 0.58),
        CleanCategoryId::PrefetchAndDns => Color::from_rgb(0.74, 0.64, 0.78),
    }
}

pub fn view<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    let s = tr(app.config.lang);
    let selected_bytes: u64 = app
        .clean_scan
        .categories
        .iter()
        .filter(|c| c.selected)
        .map(|c| c.size_bytes)
        .sum();
    let any_selected = app.clean_scan.categories.iter().any(|c| c.selected);
    let can_clean = any_selected && !app.cleaner_cleaning && !app.cleaner_scanning;

    let actions = row![
        secondary_button(
            p,
            if app.cleaner_scanning {
                s.scanning.into()
            } else {
                s.scan_junk.into()
            },
            if app.cleaner_scanning {
                Some(motion::spinner(
                    app.motion.spinner_phase,
                    13.0,
                    p.accent,
                    p.hairline,
                ))
            } else {
                Some(icons::refresh(13.0, p.text_secondary).into())
            },
            if app.cleaner_scanning || app.cleaner_cleaning {
                None
            } else {
                Some(Message::ScanJunk)
            },
        ),
        primary_button(
            p,
            if app.cleaner_cleaning {
                s.working.into()
            } else {
                format!("{} ({})", s.clean_selected, format_bytes(selected_bytes))
            },
            if app.cleaner_cleaning {
                Some(motion::spinner(
                    app.motion.spinner_phase,
                    13.0,
                    p.accent_fg,
                    p.hairline,
                ))
            } else {
                Some(
                    icons::cleaner(
                        13.0,
                        if can_clean { p.accent_fg } else { p.text_muted },
                    )
                    .into(),
                )
            },
            if can_clean {
                Some(Message::CleanSelectedJunk)
            } else {
                None
            },
        ),
    ]
    .spacing(8)
    .align_y(Vertical::Center);

    let header = page_header(p, s.cleaner_title, s.cleaner_sub, actions.into());

    let segments: Vec<StorageSegment> = app
        .clean_scan
        .categories
        .iter()
        .map(|c| StorageSegment {
            bytes: c.size_bytes,
            color: category_color(c.id, p),
        })
        .collect();

    let summary_inner = {
        let mut col = column![
            row![
                text(format_bytes(app.clean_scan.total_bytes))
                    .font(FONT_SEMIBOLD)
                    .size(22)
                    .color(p.text_primary),
                text(format!(
                    "{}  ·  {} {}  ·  {} ms",
                    s.reclaimable,
                    app.clean_scan.total_files,
                    s.files,
                    app.clean_scan.elapsed_ms
                ))
                .font(FONT_REGULAR)
                .size(12.5)
                .color(p.text_secondary),
                Space::new().width(Length::Fill),
                if app.cleaner_scanning {
                    motion::spinner(app.motion.spinner_phase, 15.0, p.accent, p.hairline)
                } else {
                    Space::new().into()
                },
            ]
            .spacing(10)
            .align_y(Vertical::Center),
            motion::storage_breakdown_bar(segments, app.clean_scan.total_bytes, p.hairline),
        ]
        .spacing(12);

        if let Some(res) = &app.last_clean_exec {
            col = col.push(
                row![
                    icons::check(13.0, p.success),
                    text(format!(
                        "Freed {} across {} files in {} ms ({} locked skipped)",
                        format_bytes(res.freed_bytes),
                        res.deleted_files,
                        res.elapsed_ms,
                        res.skipped_files
                    ))
                    .font(FONT_MEDIUM)
                    .size(12)
                    .color(p.success),
                ]
                .spacing(6)
                .align_y(Vertical::Center),
            );
        }

        container(col).padding([16, 18])
    };

    let summary_box = container(summary_inner)
        .width(Length::Fill)
        .style(theme::surface_panel_style(p));

    let rows: Vec<Element<'a, Message>> = app
        .clean_scan
        .categories
        .iter()
        .map(|cat| {
            let (name, desc) = match app.config.lang {
                Lang::En => (cat.name_en, cat.desc_en),
                Lang::PtBr => (cat.name_pt, cat.desc_pt),
            };
            let dot_color = category_color(cat.id, p);
            let hkey = format!("cln_{:?}", cat.id);
            let ht = app.motion.hover_level(&hkey);

            let mut title_row = row![
                container(Space::new())
                    .width(Length::Fixed(8.0))
                    .height(Length::Fixed(8.0))
                    .style(theme::dot_style(dot_color)),
                text(name)
                    .font(FONT_MEDIUM)
                    .size(13)
                    .color(p.text_primary),
            ]
            .spacing(8)
            .align_y(Vertical::Center);

            if cat.requires_admin && !app.sys_info.is_admin {
                title_row = title_row.push(icons::lock(11.5, p.text_muted));
            }

            let inner = button(
                row![
                    checkbox_box(cat.selected, p),
                    column![
                        title_row,
                        text(desc)
                            .font(FONT_REGULAR)
                            .size(12)
                            .color(p.text_secondary),
                    ]
                    .spacing(2)
                    .width(Length::Fill),
                    column![
                        text(format_bytes(cat.size_bytes))
                            .font(FONT_SEMIBOLD)
                            .size(12.5)
                            .color(if cat.size_bytes > 0 {
                                p.text_primary
                            } else {
                                p.text_muted
                            }),
                        text(format!("{} {}", cat.file_count, s.files))
                            .font(FONT_REGULAR)
                            .size(11)
                            .color(p.text_muted),
                    ]
                    .spacing(1)
                    .align_x(Horizontal::Right),
                ]
                .spacing(12)
                .align_y(Vertical::Center),
            )
            .width(Length::Fill)
            .padding([10, 14])
            .style(theme::btn_transparent(p))
            .on_press(Message::ToggleCleanCategory(cat.id));

            hover_row(
                p,
                ht,
                false,
                Message::HoverEnter(hkey.clone()),
                Message::HoverExit(hkey),
                inner.into(),
            )
        })
        .collect();

    let body = column![summary_box, clean_list(rows)]
        .spacing(14)
        .padding(Padding {
            top: 14.0 + app.motion.tab_offset_y(),
            right: 20.0,
            bottom: 24.0,
            left: 20.0,
        });

    let scroll = scrollable(body)
        .height(Length::Fill)
        .style(theme::scrollable_style(p));

    column![header, h_divider(p), scroll]
        .height(Length::Fill)
        .into()
}
