use crate::app::{App, Message};
use crate::core::tweaks::{SystemTweak, TweakCategory};
use crate::i18n::{tr, Lang};
use crate::ui::components::{
    h_divider, inset_group, page_header, primary_button, secondary_button, section_heading,
    segmented_control,
};
use crate::ui::icons;
use crate::ui::theme::{self, Palette, FONT_MEDIUM, FONT_REGULAR};
use iced::alignment::Vertical;
use iced::widget::{column, container, row, scrollable, text, toggler, Space};
use iced::{Color, Element, Length, Padding};

fn cat_name(cat: TweakCategory, lang: Lang) -> &'static str {
    let s = tr(lang);
    match cat {
        TweakCategory::Privacy => s.tweak_privacy,
        TweakCategory::Performance => s.tweak_performance,
        TweakCategory::Gaming => s.tweak_gaming,
        TweakCategory::Explorer => s.tweak_explorer,
        TweakCategory::PowerAndSystem => s.tweak_power,
        TweakCategory::Maintenance => s.tweak_maintenance,
    }
}

pub fn view<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    let s = tr(app.config.lang);

    let actions = row![
        secondary_button(
            p,
            s.restore_defaults.into(),
            Some(icons::rotate_ccw(13.0, p.text_secondary).into()),
            Some(Message::PromptTweaksConfirm(false)),
        ),
        primary_button(
            p,
            s.apply_recommended.into(),
            Some(icons::sparkles(13.0, Color::WHITE).into()),
            Some(Message::PromptTweaksConfirm(true)),
        ),
    ]
    .spacing(8)
    .align_y(Vertical::Center);

    let header = page_header(p, s.tweaks_title, s.tweaks_sub, actions.into());

    let seg = segmented_control(
        p,
        vec![
            (
                s.cat_all,
                app.tweak_filter.is_none(),
                Message::TweakFilterChanged(None),
            ),
            (
                s.tweak_privacy,
                app.tweak_filter == Some(TweakCategory::Privacy),
                Message::TweakFilterChanged(Some(TweakCategory::Privacy)),
            ),
            (
                s.tweak_performance,
                app.tweak_filter == Some(TweakCategory::Performance),
                Message::TweakFilterChanged(Some(TweakCategory::Performance)),
            ),
            (
                s.tweak_gaming,
                app.tweak_filter == Some(TweakCategory::Gaming),
                Message::TweakFilterChanged(Some(TweakCategory::Gaming)),
            ),
            (
                s.tweak_explorer,
                app.tweak_filter == Some(TweakCategory::Explorer),
                Message::TweakFilterChanged(Some(TweakCategory::Explorer)),
            ),
            (
                s.tweak_power,
                app.tweak_filter == Some(TweakCategory::PowerAndSystem),
                Message::TweakFilterChanged(Some(TweakCategory::PowerAndSystem)),
            ),
            (
                s.tweak_maintenance,
                app.tweak_filter == Some(TweakCategory::Maintenance),
                Message::TweakFilterChanged(Some(TweakCategory::Maintenance)),
            ),
        ],
    );

    let toolbar = container(
        row![
            seg,
            Space::new().width(Length::Fill),
            secondary_button(
                p,
                s.create_restore_point.into(),
                Some(icons::shield_check(13.0, p.accent).into()),
                Some(Message::CreateRestorePoint),
            ),
        ]
        .spacing(10)
        .align_y(Vertical::Center),
    )
    .padding(Padding {
        top: 0.0,
        right: 24.0,
        bottom: 12.0,
        left: 24.0,
    });

    // macOS System Settings grouped sections — ZERO inline pill confetti!
    let mut body = column![].spacing(18).padding(Padding {
        top: 14.0 + app.motion.tab_offset_y(),
        right: 24.0,
        bottom: 24.0,
        left: 24.0,
    });

    for &cat in TweakCategory::ALL {
        if let Some(filter) = app.tweak_filter {
            if filter != cat {
                continue;
            }
        }
        let items: Vec<&SystemTweak> = app.tweaks.iter().filter(|t| t.category == cat).collect();
        if items.is_empty() {
            continue;
        }
        let active_in_cat = items.iter().filter(|t| t.enabled).count();
        let meta = format!("{active_in_cat} / {}", items.len());

        let rows: Vec<Element<'a, Message>> = items
            .into_iter()
            .map(|tweak| {
                let (name, desc) = match app.config.lang {
                    Lang::En => (tweak.name_en, tweak.desc_en),
                    Lang::PtBr => (tweak.name_pt, tweak.desc_pt),
                };

                let mut title_line = row![text(name)
                    .font(FONT_MEDIUM)
                    .size(13)
                    .color(p.text_primary)]
                .spacing(6)
                .align_y(Vertical::Center);

                if tweak.requires_admin && !app.sys_info.is_admin {
                    title_line = title_line.push(icons::lock(11.5, p.text_muted));
                }

                let id = tweak.id;
                container(
                    row![
                        column![
                            title_line,
                            text(desc)
                                .font(FONT_REGULAR)
                                .size(12)
                                .color(p.text_secondary),
                        ]
                        .spacing(2)
                        .width(Length::Fill),
                        toggler(tweak.enabled)
                            .size(20)
                            .style(theme::toggler_style(p))
                            .on_toggle(move |val| Message::ToggleTweak(id, val)),
                    ]
                    .spacing(16)
                    .align_y(Vertical::Center),
                )
                .padding([11, 16])
                .into()
            })
            .collect();

        body = body.push(
            column![
                section_heading(p, cat_name(cat, app.config.lang), Some(meta)),
                inset_group(p, rows),
            ]
            .spacing(2),
        );
    }

    let scroll = scrollable(body)
        .height(Length::Fill)
        .style(theme::scrollable_style(p));

    column![header, toolbar, h_divider(p), scroll]
        .height(Length::Fill)
        .into()
}
