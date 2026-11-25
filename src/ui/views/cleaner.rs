use crate::app::{App, Message};
use crate::core::winapi::format_bytes;
use crate::i18n::{tr, Lang};
use crate::ui::components::{
    h_divider, page_header, primary_button, secondary_button, stat_card, tag_pill,
};
use crate::ui::icons;
use crate::ui::theme::{self, Palette, FONT_MEDIUM, FONT_REGULAR, FONT_SEMIBOLD};
use iced::alignment::Vertical;
use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Color, Element, Length, Padding};

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

    let clean_enabled = any_selected && !app.cleaner_cleaning && !app.cleaner_scanning;
    let actions = row![
        secondary_button(
            p,
            if app.cleaner_scanning {
                s.scanning.into()
            } else {
                s.scan_junk.into()
            },
            Some(icons::refresh(14.0, p.text_secondary).into()),
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
            Some(
                icons::cleaner(
                    14.0,
                    if clean_enabled { Color::WHITE } else { p.text_muted },
                )
                .into(),
            ),
            if clean_enabled {
                Some(Message::CleanSelectedJunk)
            } else {
                None
            },
        ),
    ]
    .spacing(8)
    .align_y(Vertical::Center);

    let header = page_header(p, s.cleaner_title, s.cleaner_sub, actions.into());

    let stats = container(
        row![
            stat_card(
                p,
                s.reclaimable_space,
                format_bytes(app.clean_scan.total_bytes),
                p.accent,
            ),
            stat_card(
                p,
                s.files_detected,
                app.clean_scan.total_files.to_string(),
                p.text_primary,
            ),
            stat_card(
                p,
                s.scan_time,
                format!("{} ms", app.clean_scan.elapsed_ms),
                p.success,
            ),
        ]
        .spacing(12),
    )
    .padding(Padding {
        top: 0.0,
        right: 26.0,
        bottom: 14.0,
        left: 26.0,
    });

    let mut body = column![].spacing(8).padding(Padding {
        top: 14.0,
        right: 26.0,
        bottom: 24.0,
        left: 26.0,
    });

    if let Some(res) = &app.last_clean_exec {
        let banner = container(
            row![
                icons::check(15.0, p.success),
                text(format!(
                    "Cleaned {} across {} files in {} ms  ({} locked files skipped safely)",
                    format_bytes(res.freed_bytes),
                    res.deleted_files,
                    res.elapsed_ms,
                    res.skipped_files
                ))
                .font(FONT_MEDIUM)
                .size(12)
                .color(p.success),
            ]
            .spacing(8)
            .align_y(Vertical::Center),
        )
        .width(Length::Fill)
        .padding([10, 14])
        .style(theme::badge_style(
            p.success_soft,
            p.success,
            Color::TRANSPARENT,
        ));
        body = body.push(banner);
    }

    for cat in &app.clean_scan.categories {
        let (name, desc) = match app.config.lang {
            Lang::En => (cat.name_en, cat.desc_en),
            Lang::PtBr => (cat.name_pt, cat.desc_pt),
        };

        let mut tags = row![].spacing(6).align_y(Vertical::Center);
        if cat.safe_default {
            tags = tags.push(tag_pill(s.safe_tag, p.success_soft, p.success));
        } else {
            tags = tags.push(tag_pill(s.optional_tag, p.warning_soft, p.warning));
        }
        if cat.requires_admin {
            tags = tags.push(tag_pill(s.admin_tag, p.bg_input, p.text_muted));
        }

        let card = button(
            row![
                container(if cat.selected {
                    Element::from(icons::check(10.0, Color::WHITE))
                } else {
                    Space::new()
                        .width(Length::Fixed(10.0))
                        .height(Length::Fixed(10.0))
                        .into()
                })
                .padding(3)
                .style(theme::badge_style(
                    if cat.selected { p.accent } else { p.bg_input },
                    Color::WHITE,
                    if cat.selected { p.accent } else { p.border },
                )),
                column![
                    row![
                        icons::database(13.0, if cat.selected { p.accent } else { p.text_muted }),
                        text(name)
                            .font(FONT_SEMIBOLD)
                            .size(13)
                            .color(p.text_primary),
                        tags,
                    ]
                    .spacing(7)
                    .align_y(Vertical::Center),
                    text(desc)
                        .font(FONT_REGULAR)
                        .size(12)
                        .color(p.text_secondary),
                ]
                .spacing(3)
                .width(Length::Fill),
                column![
                    text(format_bytes(cat.size_bytes))
                        .font(FONT_SEMIBOLD)
                        .size(13)
                        .color(if cat.size_bytes > 0 {
                            p.text_primary
                        } else {
                            p.text_muted
                        }),
                    text(format!("{} files", cat.file_count))
                        .font(FONT_REGULAR)
                        .size(11)
                        .color(p.text_muted),
                ]
                .spacing(2)
                .align_x(iced::alignment::Horizontal::Right),
            ]
            .spacing(12)
            .align_y(Vertical::Center),
        )
        .width(Length::Fill)
        .padding([12, 14])
        .style(theme::btn_card_selectable(p, cat.selected))
        .on_press(Message::ToggleCleanCategory(cat.id));

        body = body.push(card);
    }

    let scroll = scrollable(body)
        .height(Length::Fill)
        .style(theme::scrollable_style(p));

    column![header, stats, h_divider(p), scroll]
        .height(Length::Fill)
        .into()
}
