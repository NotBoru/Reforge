use crate::app::{App, Message};
use crate::core::debloater::{BloatPackage, BloatSafety, DEEP_DEBLOAT_TOOLS};
use crate::i18n::{tr, Lang};
use crate::ui::components::{
    checkbox_box, clean_list, destructive_button, empty_state, h_divider, hover_row, page_header,
    secondary_button, section_heading, status_dot,
};
use crate::ui::icons;
use crate::ui::motion;
use crate::ui::theme::{self, Palette, FONT_MEDIUM, FONT_REGULAR};
use iced::alignment::Vertical;
use iced::widget::{button, checkbox, column, container, row, scrollable, text, Space};
use iced::{Color, Element, Length, Padding};
use std::collections::BTreeMap;

pub fn view<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    let s = tr(app.config.lang);
    let selected_count = app.bloat_packages.iter().filter(|b| b.selected).count();

    let actions = row![
        secondary_button(
            p,
            s.select_safe.into(),
            None,
            if app.bloat_packages.is_empty() {
                None
            } else {
                Some(Message::SelectSafeBloatOnly)
            },
        ),
        secondary_button(
            p,
            if app.bloat_scanning {
                s.scanning.into()
            } else {
                s.refresh.into()
            },
            if app.bloat_scanning {
                Some(motion::spinner(
                    app.motion.spinner_phase,
                    13.0,
                    p.accent,
                    p.hairline,
                ))
            } else {
                Some(icons::refresh(13.0, p.text_secondary).into())
            },
            if app.bloat_scanning || app.bloat_removing {
                None
            } else {
                Some(Message::ScanBloatware)
            },
        ),
        destructive_button(
            p,
            if app.bloat_removing {
                s.working.into()
            } else if selected_count > 0 {
                format!("{} ({selected_count})", s.remove_selected)
            } else {
                s.remove_selected.into()
            },
            if app.bloat_removing {
                Some(motion::spinner(
                    app.motion.spinner_phase,
                    13.0,
                    Color::WHITE,
                    p.hairline,
                ))
            } else {
                Some(
                    icons::trash(
                        13.0,
                        if selected_count > 0 {
                            Color::WHITE
                        } else {
                            p.text_muted
                        },
                    )
                    .into(),
                )
            },
            if selected_count > 0 && !app.bloat_removing {
                Some(Message::PromptDebloatConfirm)
            } else {
                None
            },
        ),
    ]
    .spacing(8)
    .align_y(Vertical::Center);

    let header = page_header(p, s.debloater_title, s.debloater_sub, actions.into());

    let toolbar = container(
        row![
            checkbox(app.show_all_uwp)
                .label(s.show_all_uwp)
                .size(15)
                .text_size(12)
                .style(theme::checkbox_style(p))
                .on_toggle(Message::ToggleShowAllUwp),
            Space::new().width(Length::Fill),
            icons::shield_check(13.5, p.text_muted),
            text(s.debloat_safety_note)
                .font(FONT_REGULAR)
                .size(11.5)
                .color(p.text_muted),
        ]
        .spacing(8)
        .align_y(Vertical::Center),
    )
    .padding(Padding {
        top: 0.0,
        right: 24.0,
        bottom: 12.0,
        left: 24.0,
    });

    let mut body = column![].spacing(18).padding(Padding {
        top: 14.0 + app.motion.tab_offset_y(),
        right: 20.0,
        bottom: 24.0,
        left: 20.0,
    });

    // 1. Deep System Debloat Actions (OneDrive, Widgets, Recall/Copilot, Auto-Bloat, Legacy Features, Telemetry Tasks)
    let deep_rows: Vec<Element<'a, Message>> = DEEP_DEBLOAT_TOOLS
        .iter()
        .map(|tool| {
            let (name, desc) = match app.config.lang {
                Lang::En => (tool.name_en, tool.desc_en),
                Lang::PtBr => (tool.name_pt, tool.desc_pt),
            };
            let is_busy = app.deep_debloat_busy == Some(tool.action);
            let any_busy = app.deep_debloat_busy.is_some();
            let hkey = format!("deep_{:?}", tool.action);
            let ht = app.motion.hover_level(&hkey);

            let content = container(
                row![
                    column![
                        text(name)
                            .font(FONT_MEDIUM)
                            .size(13)
                            .color(p.text_primary),
                        text(desc)
                            .font(FONT_REGULAR)
                            .size(12)
                            .color(p.text_secondary),
                    ]
                    .spacing(2)
                    .width(Length::Fill),
                    secondary_button(
                        p,
                        if is_busy {
                            s.working.into()
                        } else {
                            s.run_action.into()
                        },
                        if is_busy {
                            Some(motion::spinner(
                                app.motion.spinner_phase,
                                12.5,
                                p.accent,
                                p.hairline,
                            ))
                        } else {
                            None
                        },
                        if any_busy {
                            None
                        } else {
                            Some(Message::RunDeepDebloat(tool.action))
                        },
                    ),
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
                content.into(),
            )
        })
        .collect();

    body = body.push(
        column![
            section_heading(
                p,
                s.deep_debloat_section,
                Some(DEEP_DEBLOAT_TOOLS.len().to_string()),
            ),
            clean_list(deep_rows),
        ]
        .spacing(4),
    );

    if app.bloat_scanning {
        body = body.push(empty_state(
            p,
            motion::spinner(app.motion.spinner_phase, 22.0, p.accent, p.hairline),
            s.scanning,
            "",
        ));
    } else if app.bloat_packages.is_empty() {
        body = body.push(empty_state(
            p,
            icons::check(22.0, p.success).into(),
            s.debloat_clean_title,
            s.debloat_clean_sub,
        ));
    } else {
        let mut grouped: BTreeMap<&'static str, Vec<&BloatPackage>> = BTreeMap::new();
        for pkg in &app.bloat_packages {
            grouped.entry(pkg.category).or_default().push(pkg);
        }

        for (category, pkgs) in grouped {
            let count = pkgs.len();
            let rows: Vec<Element<'a, Message>> = pkgs
                .into_iter()
                .map(|pkg| {
                    let desc = match app.config.lang {
                        Lang::En => &pkg.desc_en,
                        Lang::PtBr => &pkg.desc_pt,
                    };
                    let safety_dot = match pkg.safety {
                        BloatSafety::Safe => status_dot(p.success, None, p),
                        BloatSafety::Moderate => status_dot(p.warning, None, p),
                    };
                    let hkey = format!("blt_{}", pkg.package_full_name);
                    let ht = app.motion.hover_level(&hkey);

                    let inner = button(
                        row![
                            checkbox_box(pkg.selected, p),
                            column![
                                text(&pkg.display_name)
                                    .font(FONT_MEDIUM)
                                    .size(13)
                                    .color(p.text_primary),
                                text(desc)
                                    .font(FONT_REGULAR)
                                    .size(12)
                                    .color(p.text_secondary),
                            ]
                            .spacing(2)
                            .width(Length::Fill),
                            text(&pkg.package_name)
                                .font(FONT_REGULAR)
                                .size(11)
                                .color(p.text_muted),
                            safety_dot,
                        ]
                        .spacing(12)
                        .align_y(Vertical::Center),
                    )
                    .width(Length::Fill)
                    .padding([9, 14])
                    .style(theme::btn_transparent(p))
                    .on_press(Message::ToggleBloatPackage(pkg.package_full_name.clone()));

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

            body = body.push(
                column![
                    section_heading(p, category, Some(count.to_string())),
                    clean_list(rows),
                ]
                .spacing(4),
            );
        }
    }

    let scroll = scrollable(body)
        .height(Length::Fill)
        .style(theme::scrollable_style(p));

    column![header, toolbar, h_divider(p), scroll]
        .height(Length::Fill)
        .into()
}
