use crate::app::{App, Message, Tab};
use crate::core::updater::UpdateItemStatus;
use crate::i18n::tr;
use crate::ui::icons;
use crate::ui::theme::{self, Palette, ThemeMode, FONT_MEDIUM, FONT_SEMIBOLD};
use iced::alignment::Vertical;
use iced::widget::{button, column, container, row, text, Space};
use iced::{Color, Element, Length, Padding};

pub fn view<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    let s = tr(app.config.lang);

    // Restrained brand mark — zero version badge clutter
    let brand = container(
        row![
            container(icons::brand_mark(16.0, Color::WHITE))
                .padding(5)
                .style(theme::accent_count_style(p)),
            text("Reforge")
                .font(FONT_SEMIBOLD)
                .size(14.5)
                .color(p.text_primary),
        ]
        .spacing(9)
        .align_y(Vertical::Center),
    )
    .padding(Padding {
        top: 18.0,
        right: 16.0,
        bottom: 14.0,
        left: 16.0,
    });

    let selected_install = app.selected_catalog_ids.len() + app.selected_winget_ids.len();
    let updates_avail = app
        .updates
        .iter()
        .filter(|u| u.status == UpdateItemStatus::Available)
        .count();

    let ic_col = |tab: Tab| {
        if app.active_tab == tab {
            p.accent
        } else {
            p.text_secondary
        }
    };

    let nav_item = |tab: Tab,
                    label: &'a str,
                    icon: Element<'a, Message>,
                    count: Option<(usize, bool)>|
     -> Element<'a, Message> {
        let active = app.active_tab == tab;
        let mut r = row![
            icon,
            text(label)
                .font(if active { FONT_SEMIBOLD } else { FONT_MEDIUM })
                .size(13),
            Space::new().width(Length::Fill),
        ]
        .spacing(9)
        .align_y(Vertical::Center);

        if let Some((n, accent_badge)) = count {
            if n > 0 {
                r = r.push(
                    container(
                        text(n.to_string())
                            .font(FONT_SEMIBOLD)
                            .size(10.5)
                            .color(if accent_badge {
                                Color::WHITE
                            } else {
                                p.text_muted
                            }),
                    )
                    .padding([1, 6])
                    .style(move |t| {
                        if accent_badge {
                            theme::accent_count_style(p)(t)
                        } else {
                            theme::subtle_count_style(p)(t)
                        }
                    }),
                );
            }
        }

        button(r)
            .width(Length::Fill)
            .padding([8, 10])
            .style(theme::btn_nav(p, active))
            .on_press(Message::SelectTab(tab))
            .into()
    };

    let group_header = |title: &'a str| {
        container(
            text(title)
                .font(FONT_SEMIBOLD)
                .size(10)
                .color(p.text_muted),
        )
        .padding(Padding {
            top: 10.0,
            right: 10.0,
            bottom: 4.0,
            left: 10.0,
        })
    };

    let nav = column![
        group_header(s.nav_apps),
        nav_item(
            Tab::Installer,
            s.tab_installer,
            icons::download(15.0, ic_col(Tab::Installer)).into(),
            if selected_install > 0 {
                Some((selected_install, true))
            } else {
                None
            },
        ),
        nav_item(
            Tab::Updater,
            s.tab_updater,
            icons::refresh(15.0, ic_col(Tab::Updater)).into(),
            if updates_avail > 0 {
                Some((updates_avail, true))
            } else {
                None
            },
        ),
        nav_item(
            Tab::Uninstaller,
            s.tab_uninstaller,
            icons::trash(15.0, ic_col(Tab::Uninstaller)).into(),
            if !app.selected_uninstall_ids.is_empty() {
                Some((app.selected_uninstall_ids.len(), true))
            } else if !app.installed_apps.is_empty() {
                Some((app.installed_apps.len(), false))
            } else {
                None
            },
        ),
        group_header(s.nav_system),
        nav_item(
            Tab::Cleaner,
            s.tab_cleaner,
            icons::cleaner(15.0, ic_col(Tab::Cleaner)).into(),
            None,
        ),
        nav_item(
            Tab::Debloater,
            s.tab_debloater,
            icons::package_x(15.0, ic_col(Tab::Debloater)).into(),
            if !app.bloat_packages.is_empty() {
                Some((app.bloat_packages.len(), false))
            } else {
                None
            },
        ),
        nav_item(
            Tab::Startup,
            s.tab_startup,
            icons::zap(15.0, ic_col(Tab::Startup)).into(),
            None,
        ),
        nav_item(
            Tab::Tweaks,
            s.tab_tweaks,
            icons::sliders(15.0, ic_col(Tab::Tweaks)).into(),
            None,
        ),
    ]
    .spacing(2)
    .padding([0, 10]);

    // Quiet admin status or subtle elevate button
    let admin_ctrl: Element<'a, Message> = if app.sys_info.is_admin {
        container(
            row![
                icons::shield_check(13.0, p.success),
                text(s.admin_active)
                    .font(FONT_MEDIUM)
                    .size(11.5)
                    .color(p.text_secondary),
            ]
            .spacing(7)
            .align_y(Vertical::Center),
        )
        .width(Length::Fill)
        .padding([6, 10])
        .into()
    } else {
        button(
            row![
                icons::shield(13.0, p.warning),
                text(s.elevate_admin)
                    .font(FONT_MEDIUM)
                    .size(11.5)
                    .color(p.text_secondary),
            ]
            .spacing(7)
            .align_y(Vertical::Center),
        )
        .width(Length::Fill)
        .padding([7, 10])
        .style(theme::btn_secondary(p))
        .on_press(Message::ElevateAsAdmin)
        .into()
    };

    let theme_icon = match app.config.theme {
        ThemeMode::Dark => icons::moon(14.0, p.text_secondary),
        ThemeMode::Light => icons::sun(14.0, p.text_secondary),
        ThemeMode::System => icons::monitor(14.0, p.text_secondary),
    };

    let utility_row = row![
        button(container(theme_icon).center_x(Length::Fill))
            .width(Length::Fill)
            .padding([7, 6])
            .style(theme::btn_secondary(p))
            .on_press(Message::CycleTheme),
        button(
            container(
                text(app.config.lang.code())
                    .font(FONT_SEMIBOLD)
                    .size(11)
                    .color(p.text_primary),
            )
            .center_x(Length::Fill)
        )
        .width(Length::Fill)
        .padding([7, 6])
        .style(theme::btn_secondary(p))
        .on_press(Message::ToggleLanguage),
        button(container(icons::settings(14.0, ic_col(Tab::Settings))).center_x(Length::Fill))
            .width(Length::Fill)
            .padding([7, 6])
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

    let footer = container(column![admin_ctrl, utility_row].spacing(8)).padding(Padding {
        top: 10.0,
        right: 12.0,
        bottom: 14.0,
        left: 12.0,
    });

    row![
        container(column![brand, nav, Space::new().height(Length::Fill), footer])
            .width(Length::Fixed(214.0))
            .height(Length::Fill)
            .style(theme::sidebar_style(p)),
        container(Space::new())
            .width(Length::Fixed(1.0))
            .height(Length::Fill)
            .style(theme::border_divider_style(p)),
    ]
    .into()
}
