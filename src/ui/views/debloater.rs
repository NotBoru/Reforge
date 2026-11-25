use crate::app::{App, Message};
use crate::core::debloater::{BloatPackage, BloatSafety};
use crate::i18n::{tr, Lang};
use crate::ui::components::{
    danger_button, empty_state, h_divider, page_header, secondary_button, tag_pill,
};
use crate::ui::icons;
use crate::ui::theme::{self, Palette, FONT_REGULAR, FONT_SEMIBOLD};
use iced::alignment::Vertical;
use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Color, Element, Length, Padding};

pub fn view<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    let s = tr(app.config.lang);
    let selected_count = app.bloat_packages.iter().filter(|b| b.selected).count();

    let actions = row![
        secondary_button(
            p,
            s.select_safe.into(),
            None,
            Some(Message::SelectSafeBloatOnly),
        ),
        secondary_button(
            p,
            if app.bloat_scanning {
                s.scanning.into()
            } else {
                s.refresh.into()
            },
            Some(icons::refresh(14.0, p.text_secondary).into()),
            if app.bloat_scanning || app.bloat_removing {
                None
            } else {
                Some(Message::ScanBloatware)
            },
        ),
        danger_button(
            p,
            if app.bloat_removing {
                s.working.into()
            } else if selected_count > 0 {
                format!("{} ({})", s.remove_selected, selected_count)
            } else {
                s.remove_selected.into()
            },
            Some(icons::package_x(14.0, p.danger).into()),
            if selected_count > 0 && !app.bloat_removing {
                Some(Message::RemoveSelectedBloat)
            } else {
                None
            },
        ),
    ]
    .spacing(8)
    .align_y(Vertical::Center);

    let header = page_header(p, s.debloater_title, s.debloater_sub, actions.into());

    let note = container(
        container(
            row![
                icons::shield_check(15.0, p.accent),
                text(s.debloat_safety_note)
                    .font(FONT_REGULAR)
                    .size(12)
                    .color(p.text_secondary),
            ]
            .spacing(9)
            .align_y(Vertical::Center),
        )
        .width(Length::Fill)
        .padding([10, 14])
        .style(theme::well_style(p)),
    )
    .padding(Padding {
        top: 0.0,
        right: 26.0,
        bottom: 14.0,
        left: 26.0,
    });

    let mut body = column![].spacing(10).padding(Padding {
        top: 14.0,
        right: 26.0,
        bottom: 24.0,
        left: 26.0,
    });

    if app.bloat_scanning {
        body = body.push(empty_state(
            p,
            icons::package_x(22.0, p.accent).into(),
            s.scanning,
            "Appx / UWP Packages",
        ));
    } else if app.bloat_packages.is_empty() {
        body = body.push(empty_state(
            p,
            icons::check(22.0, p.success).into(),
            s.debloat_clean_title,
            s.debloat_clean_sub,
        ));
    } else {
        for chunk in app.bloat_packages.chunks(2) {
            let mut r = row![].spacing(10);
            for pkg in chunk {
                r = r.push(view_bloat_card(pkg, app.config.lang, p));
            }
            if chunk.len() == 1 {
                r = r.push(Space::new().width(Length::Fill));
            }
            body = body.push(r);
        }
    }

    let scroll = scrollable(body)
        .height(Length::Fill)
        .style(theme::scrollable_style(p));

    column![header, note, h_divider(p), scroll]
        .height(Length::Fill)
        .into()
}

fn view_bloat_card<'a>(pkg: &'a BloatPackage, lang: Lang, p: Palette) -> Element<'a, Message> {
    let s = tr(lang);
    let desc = match lang {
        Lang::En => &pkg.desc_en,
        Lang::PtBr => &pkg.desc_pt,
    };

    let safety_pill = match pkg.safety {
        BloatSafety::Safe => tag_pill(s.safe_tag, p.success_soft, p.success),
        BloatSafety::Moderate => tag_pill(s.moderate_tag, p.warning_soft, p.warning),
    };

    let top = row![
        container(if pkg.selected {
            Element::from(icons::check(10.0, Color::WHITE))
        } else {
            Space::new()
                .width(Length::Fixed(10.0))
                .height(Length::Fixed(10.0))
                .into()
        })
        .padding(3)
        .style(theme::badge_style(
            if pkg.selected { p.accent } else { p.bg_input },
            Color::WHITE,
            if pkg.selected { p.accent } else { p.border },
        )),
        column![
            text(&pkg.display_name)
                .font(FONT_SEMIBOLD)
                .size(13)
                .color(p.text_primary),
            text(if pkg.version.is_empty() {
                pkg.package_name.clone()
            } else {
                format!("{}  ·  v{}", pkg.package_name, pkg.version)
            })
            .font(FONT_REGULAR)
            .size(10)
            .color(p.text_muted),
        ]
        .spacing(1)
        .width(Length::Fill),
        tag_pill(pkg.category, p.bg_input, p.text_secondary),
        safety_pill,
    ]
    .spacing(8)
    .align_y(Vertical::Center);

    button(
        column![
            top,
            text(desc)
                .font(FONT_REGULAR)
                .size(12)
                .color(p.text_secondary),
        ]
        .spacing(8),
    )
    .width(Length::Fill)
    .padding([12, 14])
    .style(theme::btn_card_selectable(p, pkg.selected))
    .on_press(Message::ToggleBloatPackage(pkg.package_full_name.clone()))
    .into()
}
