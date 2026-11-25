use crate::app::{App, Message, Tab};
use crate::i18n::tr;
use crate::ui::icons;
use crate::ui::theme::{self, Palette, ThemeMode, FONT_BOLD, FONT_MEDIUM, FONT_SEMIBOLD};
use iced::alignment::Vertical;
use iced::widget::{button, column, container, row, text, Space};
use iced::{Color, Element, Length, Padding};

pub fn view<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    let s = tr(app.config.lang);

    // Brand header
    let brand = container(
        row![
            container(icons::brand_mark(18.0, Color::WHITE))
                .padding(6)
                .style(theme::badge_style(p.accent, Color::WHITE, Color::TRANSPARENT)),
            column![
                row![
                    text("Reforge")
                        .font(FONT_BOLD)
                        .size(15)
                        .color(p.text_primary),
                    container(
                        text("v1.0")
                            .font(FONT_MEDIUM)
                            .size(10)
                            .color(p.text_muted)
                    )
                    .padding([1, 6])
                    .style(theme::pill_badge_style(p.bg_surface, p.text_muted)),
                ]
                .spacing(6)
                .align_y(Vertical::Center),
                text("Windows Toolkit")
                    .font(FONT_MEDIUM)
                    .size(11)
                    .color(p.text_muted),
            ]
            .spacing(1),
        ]
        .spacing(10)
        .align_y(Vertical::Center),
    )
    .padding(Padding {
        top: 20.0,
        right: 16.0,
        bottom: 18.0,
        left: 16.0,
    });

    let selected_install_count = app.selected_catalog_ids.len() + app.selected_winget_ids.len();
    let updates_count = app
        .updates
        .iter()
        .filter(|u| matches!(u.status, crate::core::updater::UpdateItemStatus::Available))
        .count();

    let nav_item = |tab: Tab,
                    label: &'a str,
                    icon: Element<'a, Message>,
                    badge: Option<(String, Color, Color)>|
     -> Element<'a, Message> {
        let active = app.active_tab == tab;
        let mut r = row![
            icon,
            text(label)
                .font(if active { FONT_SEMIBOLD } else { FONT_MEDIUM })
                .size(13),
            Space::new().width(Length::Fill),
        ]
        .spacing(10)
        .align_y(Vertical::Center);

        if let Some((btxt, bbg, bfg)) = badge {
            r = r.push(
                container(text(btxt).font(FONT_SEMIBOLD).size(11).color(bfg))
                    .padding([1, 7])
                    .style(theme::pill_badge_style(bbg, bfg)),
            );
        }

        button(r)
            .width(Length::Fill)
            .padding([9, 12])
            .style(theme::btn_nav(p, active))
            .on_press(Message::SelectTab(tab))
            .into()
    };

    let ic_color = |tab: Tab| {
        if app.active_tab == tab {
            p.accent
        } else {
            p.text_secondary
        }
    };

    let section_label = |title: &'a str| {
        container(
            text(title)
                .font(FONT_SEMIBOLD)
                .size(10)
                .color(p.text_muted),
        )
        .padding(Padding {
            top: 10.0,
            right: 12.0,
            bottom: 4.0,
            left: 12.0,
        })
    };

    let nav = column![
        section_label(s.nav_section_apps),
        nav_item(
            Tab::Installer,
            s.tab_installer,
            icons::download(16.0, ic_color(Tab::Installer)).into(),
            if selected_install_count > 0 {
                Some((selected_install_count.to_string(), p.accent, Color::WHITE))
            } else {
                None
            },
        ),
        nav_item(
            Tab::Updater,
            s.tab_updater,
            icons::refresh(16.0, ic_color(Tab::Updater)).into(),
            if updates_count > 0 {
                Some((updates_count.to_string(), p.warning_soft, p.warning))
            } else {
                None
            },
        ),
        nav_item(
            Tab::Uninstaller,
            s.tab_uninstaller,
            icons::trash(16.0, ic_color(Tab::Uninstaller)).into(),
            if !app.installed_apps.is_empty() {
                Some((
                    app.installed_apps.len().to_string(),
                    p.bg_surface,
                    p.text_muted,
                ))
            } else {
                None
            },
        ),
        section_label(s.nav_section_system),
        nav_item(
            Tab::Cleaner,
            s.tab_cleaner,
            icons::cleaner(16.0, ic_color(Tab::Cleaner)).into(),
            None,
        ),
        nav_item(
            Tab::Debloater,
            s.tab_debloater,
            icons::package_x(16.0, ic_color(Tab::Debloater)).into(),
            if !app.bloat_packages.is_empty() {
                Some((
                    app.bloat_packages.len().to_string(),
                    p.bg_surface,
                    p.text_muted,
                ))
            } else {
                None
            },
        ),
        nav_item(
            Tab::Startup,
            s.tab_startup,
            icons::zap(16.0, ic_color(Tab::Startup)).into(),
            None,
        ),
        nav_item(
            Tab::Tweaks,
            s.tab_tweaks,
            icons::sliders(16.0, ic_color(Tab::Tweaks)).into(),
            None,
        ),
    ]
    .spacing(3)
    .padding([0, 10]);

    // Admin shield status / Elevate CTA
    let admin_block: Element<'a, Message> = if app.sys_info.is_admin {
        container(
            row![
                icons::shield_check(14.0, p.success),
                text(s.admin_active)
                    .font(FONT_MEDIUM)
                    .size(12)
                    .color(p.success),
            ]
            .spacing(7)
            .align_y(Vertical::Center),
        )
        .width(Length::Fill)
        .padding([7, 10])
        .style(theme::badge_style(
            p.success_soft,
            p.success,
            Color::TRANSPARENT,
        ))
        .into()
    } else {
        button(
            row![
                icons::shield(14.0, p.warning),
                text(s.elevate_admin)
                    .font(FONT_MEDIUM)
                    .size(12)
                    .color(p.warning),
            ]
            .spacing(7)
            .align_y(Vertical::Center),
        )
        .width(Length::Fill)
        .padding([7, 10])
        .style(move |_, status| {
            let bg = match status {
                button::Status::Hovered => p.warning_soft,
                _ => p.bg_surface,
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: p.warning,
                border: iced::Border {
                    color: p.border,
                    width: 1.0,
                    radius: iced::border::Radius::from(8.0),
                },
                ..Default::default()
            }
        })
        .on_press(Message::ElevateAsAdmin)
        .into()
    };

    // Quick bottom utility row: Theme cycle + Language toggle + Settings
    let theme_icon = match app.config.theme {
        ThemeMode::Dark => icons::moon(15.0, p.text_secondary),
        ThemeMode::Light => icons::sun(15.0, p.text_secondary),
        ThemeMode::System => icons::monitor(15.0, p.text_secondary),
    };

    let utility_row = row![
        button(container(theme_icon).center_x(Length::Fill))
            .width(Length::Fill)
            .padding([7, 8])
            .style(theme::btn_secondary(p))
            .on_press(Message::CycleTheme),
        button(
            container(
                row![
                    icons::globe(14.0, p.text_secondary),
                    text(app.config.lang.code())
                        .font(FONT_SEMIBOLD)
                        .size(11)
                        .color(p.text_primary),
                ]
                .spacing(5)
                .align_y(Vertical::Center)
            )
            .center_x(Length::Fill)
        )
        .width(Length::Fill)
        .padding([7, 8])
        .style(theme::btn_secondary(p))
        .on_press(Message::ToggleLanguage),
        button(
            container(icons::settings(15.0, ic_color(Tab::Settings))).center_x(Length::Fill)
        )
        .width(Length::Fill)
        .padding([7, 8])
        .style({
            let active = app.active_tab == Tab::Settings;
            move |t, st| {
                if active {
                    theme::btn_nav(p, true)(t, st)
                } else {
                    theme::btn_secondary(p)(t, st)
                }
            }
        })
        .on_press(Message::SelectTab(Tab::Settings)),
    ]
    .spacing(6)
    .width(Length::Fill);

    let footer = container(
        column![admin_block, utility_row].spacing(8),
    )
    .padding(Padding {
        top: 12.0,
        right: 12.0,
        bottom: 14.0,
        left: 12.0,
    });

    row![
        container(column![brand, nav, Space::new().height(Length::Fill), footer])
            .width(Length::Fixed(224.0))
            .height(Length::Fill)
            .style(theme::sidebar_style(p)),
        container(Space::new())
            .width(Length::Fixed(1.0))
            .height(Length::Fill)
            .style(theme::divider_style(p)),
    ]
    .into()
}
