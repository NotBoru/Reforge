use crate::app::{App, Message};
use crate::core::updater::UpdateItemStatus;
use crate::i18n::tr;
use crate::ui::components::{
    empty_state, h_divider, page_header, primary_button, secondary_button, tag_pill,
};
use crate::ui::icons;
use crate::ui::theme::{self, Palette, FONT_MEDIUM, FONT_REGULAR, FONT_SEMIBOLD};
use iced::alignment::Vertical;
use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Color, Element, Length, Padding};

pub fn view<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    let s = tr(app.config.lang);
    let selected_count = app
        .updates
        .iter()
        .filter(|u| u.selected && u.status == UpdateItemStatus::Available)
        .count();

    let actions = row![
        secondary_button(
            p,
            if app.updates_checking {
                s.scanning.into()
            } else {
                s.check_updates.into()
            },
            Some(icons::refresh(14.0, p.text_secondary).into()),
            if app.updates_checking || app.updates_running {
                None
            } else {
                Some(Message::CheckUpdates)
            },
        ),
        primary_button(
            p,
            if app.updates_running {
                s.working.into()
            } else if selected_count > 0 {
                format!("{} ({})", s.update_selected, selected_count)
            } else {
                s.update_selected.into()
            },
            Some(icons::download(14.0, Color::WHITE).into()),
            if selected_count > 0 && !app.updates_running {
                Some(Message::RunSelectedUpdates)
            } else {
                None
            },
        ),
    ]
    .spacing(8)
    .align_y(Vertical::Center);

    let header = page_header(p, s.updater_title, s.updater_sub, actions.into());

    let mut body = column![].spacing(8).padding(Padding {
        top: 16.0,
        right: 26.0,
        bottom: 24.0,
        left: 26.0,
    });

    if app.updates_checking {
        body = body.push(empty_state(
            p,
            icons::refresh(22.0, p.accent).into(),
            s.scanning,
            "Winget & Reforge Catalog",
        ));
    } else if !app.updates_checked_once {
        body = body.push(empty_state(
            p,
            icons::refresh(22.0, p.text_secondary).into(),
            s.updater_title,
            s.not_checked_yet,
        ));
    } else if app.updates.is_empty() {
        body = body.push(empty_state(
            p,
            icons::check(22.0, p.success).into(),
            s.up_to_date_title,
            s.up_to_date_sub,
        ));
    } else {
        for upd in &app.updates {
            let status_widget: Element<'a, Message> = match &upd.status {
                UpdateItemStatus::Available => button(
                    text(s.update_one)
                        .font(FONT_MEDIUM)
                        .size(12)
                        .color(p.accent),
                )
                .padding([5, 11])
                .style(theme::btn_secondary(p))
                .on_press_maybe(if app.updates_running {
                    None
                } else {
                    Some(Message::RunSingleUpdate(upd.id.clone()))
                })
                .into(),
                UpdateItemStatus::Updating => tag_pill(s.working, p.warning_soft, p.warning),
                UpdateItemStatus::Updated => tag_pill("Updated", p.success_soft, p.success),
                UpdateItemStatus::Failed(e) => {
                    let short: String = e.chars().take(28).collect();
                    tag_pill(short, p.danger_soft, p.danger)
                }
            };

            let version_diff = row![
                tag_pill(upd.current_version.clone(), p.bg_input, p.text_secondary),
                icons::arrow_right(13.0, p.text_muted),
                tag_pill(upd.available_version.clone(), p.accent_soft, p.accent),
            ]
            .spacing(6)
            .align_y(Vertical::Center);

            let card = button(
                row![
                    container(if upd.selected {
                        Element::from(icons::check(10.0, Color::WHITE))
                    } else {
                        Space::new()
                            .width(Length::Fixed(10.0))
                            .height(Length::Fixed(10.0))
                            .into()
                    })
                    .padding(3)
                    .style(theme::badge_style(
                        if upd.selected { p.accent } else { p.bg_input },
                        Color::WHITE,
                        if upd.selected { p.accent } else { p.border },
                    )),
                    column![
                        text(&upd.name)
                            .font(FONT_SEMIBOLD)
                            .size(13)
                            .color(p.text_primary),
                        text(&upd.id)
                            .font(FONT_REGULAR)
                            .size(11)
                            .color(p.text_muted),
                    ]
                    .spacing(2)
                    .width(Length::Fill),
                    version_diff,
                    status_widget,
                ]
                .spacing(12)
                .align_y(Vertical::Center),
            )
            .width(Length::Fill)
            .padding([11, 14])
            .style(theme::btn_card_selectable(p, upd.selected))
            .on_press(Message::ToggleUpdateSelection(upd.id.clone()));

            body = body.push(card);
        }
    }

    let scroll = scrollable(body)
        .height(Length::Fill)
        .style(theme::scrollable_style(p));

    column![header, h_divider(p), scroll]
        .height(Length::Fill)
        .into()
}
