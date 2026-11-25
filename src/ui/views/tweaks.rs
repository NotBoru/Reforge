use crate::app::{App, Message};
use crate::core::tweaks::TweakCategory;
use crate::i18n::{tr, Lang};
use crate::ui::components::{h_divider, page_header, primary_button, secondary_button, tag_pill};
use crate::ui::icons;
use crate::ui::theme::{self, Palette, FONT_MEDIUM, FONT_REGULAR, FONT_SEMIBOLD};
use iced::alignment::Vertical;
use iced::widget::{button, column, container, row, scrollable, text, toggler};
use iced::{Color, Element, Length, Padding};

pub fn view<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    let s = tr(app.config.lang);

    let actions = row![
        secondary_button(
            p,
            s.restore_defaults.into(),
            Some(icons::rotate_ccw(14.0, p.text_secondary).into()),
            Some(Message::RestoreAllTweaks),
        ),
        primary_button(
            p,
            s.apply_recommended.into(),
            Some(icons::sparkles(14.0, Color::WHITE).into()),
            Some(Message::ApplyRecommendedTweaks),
        ),
    ]
    .spacing(8)
    .align_y(Vertical::Center);

    let header = page_header(p, s.tweaks_title, s.tweaks_sub, actions.into());

    let pill = |filter: Option<TweakCategory>, label: &'a str| {
        let active = app.tweak_filter == filter;
        button(
            text(label)
                .font(if active { FONT_SEMIBOLD } else { FONT_MEDIUM })
                .size(12),
        )
        .padding([5, 12])
        .style(theme::btn_pill(p, active))
        .on_press(Message::TweakFilterChanged(filter))
    };

    let toolbar = container(
        row![
            pill(None, s.cat_all),
            pill(Some(TweakCategory::Privacy), s.tweak_privacy),
            pill(Some(TweakCategory::Performance), s.tweak_performance),
            pill(Some(TweakCategory::Interface), s.tweak_interface),
            iced::widget::Space::new().width(Length::Fill),
            secondary_button(
                p,
                s.create_restore_point.into(),
                Some(icons::shield_check(14.0, p.accent).into()),
                Some(Message::CreateRestorePoint),
            ),
        ]
        .spacing(7)
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

    for tweak in &app.tweaks {
        if let Some(cat) = app.tweak_filter {
            if tweak.category != cat {
                continue;
            }
        }

        let (name, desc) = match app.config.lang {
            Lang::En => (tweak.name_en, tweak.desc_en),
            Lang::PtBr => (tweak.name_pt, tweak.desc_pt),
        };

        let cat_label = match tweak.category {
            TweakCategory::Privacy => s.tweak_privacy,
            TweakCategory::Performance => s.tweak_performance,
            TweakCategory::Interface => s.tweak_interface,
        };

        let mut tags = row![tag_pill(cat_label, p.bg_input, p.text_secondary)].spacing(6);
        if tweak.recommended {
            tags = tags.push(tag_pill(s.recommended, p.accent_soft, p.accent));
        }
        if tweak.requires_admin {
            tags = tags.push(tag_pill(s.admin_tag, p.warning_soft, p.warning));
        }

        let id = tweak.id;
        let card = container(
            row![
                column![
                    row![
                        text(name)
                            .font(FONT_SEMIBOLD)
                            .size(13)
                            .color(p.text_primary),
                        tags,
                    ]
                    .spacing(8)
                    .align_y(Vertical::Center),
                    text(desc)
                        .font(FONT_REGULAR)
                        .size(12)
                        .color(p.text_secondary),
                ]
                .spacing(3)
                .width(Length::Fill),
                toggler(tweak.enabled)
                    .size(21)
                    .style(theme::toggler_style(p))
                    .on_toggle(move |val| Message::ToggleTweak(id, val)),
            ]
            .spacing(16)
            .align_y(Vertical::Center),
        )
        .width(Length::Fill)
        .padding([12, 16])
        .style(theme::card_style(p));

        body = body.push(card);
    }

    let scroll = scrollable(body)
        .height(Length::Fill)
        .style(theme::scrollable_style(p));

    column![header, toolbar, h_divider(p), scroll]
        .height(Length::Fill)
        .into()
}
