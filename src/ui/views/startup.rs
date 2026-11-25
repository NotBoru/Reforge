use crate::app::{App, Message};
use crate::i18n::tr;
use crate::ui::components::{
    app_logo, clean_list, empty_state, h_divider, hover_row, page_header, search_bar,
    secondary_button,
};
use crate::ui::icons;
use crate::ui::theme::{self, Palette, FONT_MEDIUM, FONT_REGULAR, FONT_SEMIBOLD};
use iced::alignment::Vertical;
use iced::widget::{button, column, container, row, scrollable, text, toggler};
use iced::{Element, Length, Padding};

pub fn view<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    let s = tr(app.config.lang);

    let actions = secondary_button(
        p,
        s.refresh.into(),
        Some(icons::refresh(13.0, p.text_secondary).into()),
        Some(Message::RefreshStartupItems),
    );

    let header = page_header(p, s.startup_title, s.startup_sub, actions);

    let q = app.startup_search.trim().to_lowercase();
    let filtered: Vec<_> = app
        .startup_items
        .iter()
        .filter(|i| {
            q.is_empty()
                || i.name.to_lowercase().contains(&q)
                || i.command.to_lowercase().contains(&q)
        })
        .collect();

    let enabled_count = app.startup_items.iter().filter(|i| i.enabled).count();

    let toolbar = container(
        row![
            search_bar(
                p,
                s.search_placeholder,
                &app.startup_search,
                Message::StartupSearchChanged,
                Message::StartupSearchChanged(String::new()),
            ),
            text(format!(
                "{enabled_count} / {} {}",
                app.startup_items.len(),
                s.enabled.to_lowercase()
            ))
            .font(FONT_REGULAR)
            .size(12)
            .color(p.text_muted),
        ]
        .spacing(12)
        .align_y(Vertical::Center),
    )
    .padding(Padding {
        top: 0.0,
        right: 24.0,
        bottom: 12.0,
        left: 24.0,
    });

    let mut body = column![].spacing(10).padding(Padding {
        top: 12.0 + app.motion.tab_offset_y(),
        right: 20.0,
        bottom: 24.0,
        left: 20.0,
    });

    if filtered.is_empty() {
        body = body.push(empty_state(
            p,
            icons::zap(22.0, p.text_muted).into(),
            s.empty_search,
            "",
        ));
    } else {
        let rows: Vec<Element<'a, Message>> = filtered
            .into_iter()
            .map(|item| {
                let cmd_short: String = if item.command.chars().count() > 74 {
                    format!("{}...", item.command.chars().take(74).collect::<String>())
                } else {
                    item.command.clone()
                };
                let hkey = format!("st_{}", item.id);
                let ht = app.motion.hover_level(&hkey);

                let mut name_row = row![text(&item.name)
                    .font(FONT_SEMIBOLD)
                    .size(13)
                    .color(p.text_primary)]
                .spacing(6)
                .align_y(Vertical::Center);
                if item.source.needs_admin() && !app.sys_info.is_admin {
                    name_row = name_row.push(icons::lock(11.5, p.text_muted));
                }

                let inner = container(
                    row![
                        app_logo(app.startup_icon(&item.id), &item.name, 26.0, p),
                        column![
                            name_row,
                            text(cmd_short)
                                .font(FONT_REGULAR)
                                .size(11.5)
                                .color(p.text_muted),
                        ]
                        .spacing(1)
                        .width(Length::Fill),
                        text(item.source.label())
                            .font(FONT_MEDIUM)
                            .size(11.5)
                            .color(p.text_muted),
                        toggler(item.enabled)
                            .size(19)
                            .style(theme::toggler_style(p))
                            .on_toggle({
                                let id = item.id.clone();
                                move |val| Message::ToggleStartupItem(id.clone(), val)
                            }),
                        button(icons::trash(13.5, p.text_muted))
                            .padding([6, 7])
                            .style(theme::btn_ghost(p))
                            .on_press(Message::DeleteStartupItem(item.id.clone())),
                    ]
                    .spacing(12)
                    .align_y(Vertical::Center),
                )
                .padding([9, 14]);

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

        body = body.push(clean_list(rows));
    }

    let scroll = scrollable(body)
        .height(Length::Fill)
        .style(theme::scrollable_style(p));

    column![header, toolbar, h_divider(p), scroll]
        .height(Length::Fill)
        .into()
}
