use crate::app::{App, Message};
use crate::i18n::tr;
use crate::ui::components::{
    empty_state, h_divider, page_header, search_bar, secondary_button, tag_pill,
};
use crate::ui::icons;
use crate::ui::theme::{self, Palette, FONT_MEDIUM, FONT_REGULAR, FONT_SEMIBOLD};
use iced::alignment::Vertical;
use iced::widget::{button, column, container, row, scrollable, text, toggler};
use iced::{Element, Length, Padding};

pub fn view<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    let s = tr(app.config.lang);

    let actions = row![secondary_button(
        p,
        s.refresh.into(),
        Some(icons::refresh(14.0, p.text_secondary).into()),
        Some(Message::RefreshStartupItems),
    )]
    .spacing(8);

    let header = page_header(p, s.startup_title, s.startup_sub, actions.into());

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
            tag_pill(
                format!("{enabled_count} / {} {}", app.startup_items.len(), s.enabled),
                p.bg_surface,
                p.text_secondary,
            ),
        ]
        .spacing(12)
        .align_y(Vertical::Center),
    )
    .padding(Padding {
        top: 0.0,
        right: 26.0,
        bottom: 14.0,
        left: 26.0,
    });

    let mut body = column![].spacing(7).padding(Padding {
        top: 14.0,
        right: 26.0,
        bottom: 24.0,
        left: 26.0,
    });

    if filtered.is_empty() {
        body = body.push(empty_state(
            p,
            icons::zap(22.0, p.text_muted).into(),
            s.empty_search,
            "",
        ));
    } else {
        for item in filtered {
            let cmd_display: String = if item.command.chars().count() > 84 {
                format!("{}...", item.command.chars().take(84).collect::<String>())
            } else {
                item.command.clone()
            };

            let mut badges = row![tag_pill(
                item.source.label(),
                p.bg_input,
                p.text_secondary
            )]
            .spacing(6);
            if item.source.needs_admin() {
                badges = badges.push(tag_pill(s.admin_tag, p.warning_soft, p.warning));
            }

            let card = container(
                row![
                    column![
                        row![
                            text(&item.name)
                                .font(FONT_SEMIBOLD)
                                .size(13)
                                .color(p.text_primary),
                            badges,
                        ]
                        .spacing(8)
                        .align_y(Vertical::Center),
                        text(cmd_display)
                            .font(FONT_REGULAR)
                            .size(11)
                            .color(p.text_muted),
                    ]
                    .spacing(3)
                    .width(Length::Fill),
                    text(if item.enabled { s.enabled } else { s.disabled })
                        .font(FONT_MEDIUM)
                        .size(11)
                        .color(if item.enabled { p.success } else { p.text_muted }),
                    toggler(item.enabled)
                        .size(20)
                        .style(theme::toggler_style(p))
                        .on_toggle({
                            let id = item.id.clone();
                            move |val| Message::ToggleStartupItem(id.clone(), val)
                        }),
                    button(icons::trash(14.0, p.text_muted))
                        .padding([6, 8])
                        .style(theme::btn_ghost(p))
                        .on_press(Message::DeleteStartupItem(item.id.clone())),
                ]
                .spacing(12)
                .align_y(Vertical::Center),
            )
            .width(Length::Fill)
            .padding([11, 14])
            .style(theme::card_style(p));

            body = body.push(card);
        }
    }

    let scroll = scrollable(body)
        .height(Length::Fill)
        .style(theme::scrollable_style(p));

    column![header, toolbar, h_divider(p), scroll]
        .height(Length::Fill)
        .into()
}
