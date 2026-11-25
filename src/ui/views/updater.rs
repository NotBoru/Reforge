use crate::app::{App, Message};
use crate::core::updater::UpdateItemStatus;
use crate::i18n::tr;
use crate::ui::components::{
    app_logo, checkbox_box, clean_list, empty_state, h_divider, hover_row, page_header,
    primary_button, secondary_button, status_dot,
};
use crate::ui::icons;
use crate::ui::motion;
use crate::ui::theme::{self, Palette, FONT_MEDIUM, FONT_REGULAR, FONT_SEMIBOLD};
use iced::alignment::Vertical;
use iced::widget::{button, column, row, scrollable, text};
use iced::{Element, Length, Padding};

pub fn view<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    let s = tr(app.config.lang);
    let selected_count = app
        .updates
        .iter()
        .filter(|u| u.selected && u.status == UpdateItemStatus::Available)
        .count();
    let failed_count = app
        .updates
        .iter()
        .filter(|u| matches!(u.status, UpdateItemStatus::Failed(_)))
        .count();

    let mut actions = row![].spacing(8).align_y(Vertical::Center);
    if failed_count > 0 && !app.updates_running {
        actions = actions.push(secondary_button(
            p,
            format!("{} ({failed_count})", s.retry_failed),
            Some(icons::rotate_ccw(13.0, p.warning).into()),
            Some(Message::RetryFailedUpdates),
        ));
    }

    actions = actions
        .push(secondary_button(
            p,
            if app.updates_checking {
                s.scanning.into()
            } else {
                s.check_updates.into()
            },
            if app.updates_checking {
                Some(motion::spinner(
                    app.motion.spinner_phase,
                    13.0,
                    p.accent,
                    p.hairline,
                ))
            } else {
                Some(icons::refresh(13.0, p.text_secondary).into())
            },
            if app.updates_checking || app.updates_running {
                None
            } else {
                Some(Message::CheckUpdates)
            },
        ))
        .push(primary_button(
            p,
            if app.updates_running {
                s.working.into()
            } else if selected_count > 0 {
                format!("{} ({selected_count})", s.update_selected)
            } else {
                s.update_selected.into()
            },
            if app.updates_running {
                Some(motion::spinner(
                    app.motion.spinner_phase,
                    13.0,
                    p.accent_fg,
                    p.hairline,
                ))
            } else {
                Some(
                    icons::download(
                        13.0,
                        if selected_count > 0 {
                            p.accent_fg
                        } else {
                            p.text_muted
                        },
                    )
                    .into(),
                )
            },
            if selected_count > 0 && !app.updates_running {
                Some(Message::RunSelectedUpdates)
            } else {
                None
            },
        ));

    let header = page_header(p, s.updater_title, s.updater_sub, actions.into());

    let mut body = column![].spacing(10).padding(Padding {
        top: 12.0 + app.motion.tab_offset_y(),
        right: 20.0,
        bottom: 24.0,
        left: 20.0,
    });

    if app.updates_checking {
        body = body.push(empty_state(
            p,
            motion::spinner(app.motion.spinner_phase, 22.0, p.accent, p.hairline),
            s.scanning,
            "Querying available package versions",
        ));
    } else if !app.updates_checked_once {
        body = body.push(empty_state(
            p,
            icons::refresh(22.0, p.text_muted).into(),
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
        // Calm borderless rows — NEVER paint a wall of colored rows when all 47 are checked!
        let rows: Vec<Element<'a, Message>> = app
            .updates
            .iter()
            .map(|upd| {
                let hkey = format!("upd_{}", upd.id);
                let ht = app.motion.hover_level(&hkey);
                let logo = app.update_icon(upd);
                let version_diff = row![
                    text(&upd.current_version)
                        .font(FONT_REGULAR)
                        .size(12)
                        .color(p.text_muted),
                    icons::arrow_right(12.0, p.text_muted),
                    text(&upd.available_version)
                        .font(FONT_SEMIBOLD)
                        .size(12)
                        .color(p.accent),
                ]
                .spacing(6)
                .align_y(Vertical::Center);

                let trailing: Element<'a, Message> = match &upd.status {
                    UpdateItemStatus::Available => button(
                        text(s.update_one)
                            .font(FONT_MEDIUM)
                            .size(12)
                            .color(p.accent),
                    )
                    .padding([6, 11])
                    .style(theme::btn_ghost(p))
                    .on_press_maybe(if app.updates_running {
                        None
                    } else {
                        Some(Message::RunSingleUpdate(upd.id.clone()))
                    })
                    .into(),
                    UpdateItemStatus::Updating => row![
                        motion::spinner(app.motion.spinner_phase, 13.0, p.accent, p.hairline),
                        text(s.working)
                            .font(FONT_MEDIUM)
                            .size(11.5)
                            .color(p.accent),
                    ]
                    .spacing(6)
                    .align_y(Vertical::Center)
                    .into(),
                    UpdateItemStatus::Updated => status_dot(p.success, Some("Updated"), p),
                    UpdateItemStatus::Failed(_) => button(
                        text(s.retry)
                            .font(FONT_MEDIUM)
                            .size(12)
                            .color(p.danger),
                    )
                    .padding([6, 11])
                    .style(theme::btn_ghost(p))
                    .on_press_maybe(if app.updates_running {
                        None
                    } else {
                        Some(Message::RunSingleUpdate(upd.id.clone()))
                    })
                    .into(),
                };

                let row_btn = button(
                    row![
                        checkbox_box(upd.selected, p),
                        app_logo(logo, &upd.name, 28.0, p),
                        column![
                            text(&upd.name)
                                .font(FONT_SEMIBOLD)
                                .size(13)
                                .color(p.text_primary),
                            text(&upd.id)
                                .font(FONT_REGULAR)
                                .size(11.5)
                                .color(p.text_muted),
                        ]
                        .spacing(1)
                        .width(Length::Fill),
                        version_diff,
                    ]
                    .spacing(12)
                    .align_y(Vertical::Center),
                )
                .width(Length::Fill)
                .padding([9, 12])
                .style(theme::btn_transparent(p))
                .on_press(Message::ToggleUpdateSelection(upd.id.clone()));

                hover_row(
                    p,
                    ht,
                    false,
                    Message::HoverEnter(hkey.clone()),
                    Message::HoverExit(hkey),
                    row![row_btn, trailing]
                        .spacing(6)
                        .align_y(Vertical::Center)
                        .into(),
                )
            })
            .collect();

        body = body.push(clean_list(rows));
    }

    let scroll = scrollable(body)
        .height(Length::Fill)
        .style(theme::scrollable_style(p));

    column![header, h_divider(p), scroll]
        .height(Length::Fill)
        .into()
}
