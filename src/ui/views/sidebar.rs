use crate::app::{App, Message, Tab};
use crate::core::updater::UpdateItemStatus;
use crate::i18n::tr;
use crate::ui::components::h_divider;
use crate::ui::icons;
use crate::ui::theme::{self, Palette, ThemeMode, FONT_MEDIUM, FONT_SEMIBOLD};
use iced::alignment::Vertical;
use iced::widget::{button, column, container, mouse_area, row, text, Space};
use iced::{Element, Length, Padding};

pub fn view<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    let s = tr(app.config.lang);

    let brand = container(
        row![
            container(icons::brand_mark(15.0, p.accent_fg))
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
                    key: &'static str,
                    label: &'a str,
                    icon: Element<'a, Message>,
                    count: Option<(usize, bool)>|
     -> Element<'a, Message> {
        let active = app.active_tab == tab;
        let hover_t = app.motion.hover_level(key);
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
                                p.accent_fg
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

        mouse_area(
            button(r)
                .width(Length::Fill)
                .padding([8, 10])
                .style(theme::btn_nav_animated(p, active, hover_t))
                .on_press(Message::SelectTab(tab)),
        )
        .on_enter(Message::HoverEnter(key.to_string()))
        .on_exit(Message::HoverExit(key.to_string()))
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
            "nav_inst",
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
            "nav_upd",
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
            "nav_uninst",
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
            "nav_clean",
            s.tab_cleaner,
            icons::cleaner(15.0, ic_col(Tab::Cleaner)).into(),
            None,
        ),
        nav_item(
            Tab::Debloater,
            "nav_debloat",
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
            "nav_start",
            s.tab_startup,
            icons::zap(15.0, ic_col(Tab::Startup)).into(),
            None,
        ),
        nav_item(
            Tab::Tweaks,
            "nav_tweak",
            s.tab_tweaks,
            icons::sliders(15.0, ic_col(Tab::Tweaks)).into(),
            None,
        ),
    ]
    .spacing(2)
    .padding([0, 10]);

    // Refactored Settings nav item + ultra-clean borderless utility strip (zero chunky boxed buttons!)
    let settings_nav = container(nav_item(
        Tab::Settings,
        "nav_set",
        s.tab_settings,
        icons::settings(15.0, ic_col(Tab::Settings)).into(),
        None,
    ))
    .padding(Padding {
        top: 0.0,
        right: 10.0,
        bottom: 8.0,
        left: 10.0,
    });

    let admin_indicator: Element<'a, Message> = if app.sys_info.is_admin {
        row![
            icons::shield_check(13.0, p.success),
            text(s.admin_active)
                .font(FONT_MEDIUM)
                .size(11.5)
                .color(p.text_muted),
        ]
        .spacing(6)
        .align_y(Vertical::Center)
        .into()
    } else {
        button(
            row![
                icons::shield(13.0, p.accent),
                text(s.elevate_admin)
                    .font(FONT_MEDIUM)
                    .size(11.5)
                    .color(p.text_secondary),
            ]
            .spacing(6)
            .align_y(Vertical::Center),
        )
        .padding([4, 6])
        .style(theme::btn_ghost(p))
        .on_press(Message::ElevateAsAdmin)
        .into()
    };

    let theme_icon = match app.config.theme {
        ThemeMode::Dark => icons::moon(13.5, p.text_secondary),
        ThemeMode::GeminiDark => icons::sparkles(13.5, p.accent),
        ThemeMode::Light => icons::sun(13.5, p.text_secondary),
        ThemeMode::System => icons::monitor(13.5, p.text_secondary),
    };

    let theme_btn = button(theme_icon)
        .padding([5, 6])
        .style(theme::btn_ghost(p))
        .on_press(Message::CycleTheme);

    let lang_btn = button(
        text(app.config.lang.code())
            .font(FONT_SEMIBOLD)
            .size(11)
            .color(p.text_secondary),
    )
    .padding([4, 6])
    .style(theme::btn_ghost(p))
    .on_press(Message::ToggleLanguage);

    let bottom_strip = container(
        row![
            admin_indicator,
            Space::new().width(Length::Fill),
            theme_btn,
            lang_btn,
        ]
        .spacing(2)
        .align_y(Vertical::Center),
    )
    .padding(Padding {
        top: 10.0,
        right: 12.0,
        bottom: 12.0,
        left: 14.0,
    });

    row![
        container(column![
            brand,
            nav,
            Space::new().height(Length::Fill),
            settings_nav,
            h_divider(p),
            bottom_strip,
        ])
        .width(Length::Fixed(214.0))
        .height(Length::Fill)
        .style(theme::sidebar_style(p)),
        container(Space::new())
            .width(Length::Fixed(1.0))
            .height(Length::Fill)
            .style(theme::hairline_style(p)),
    ]
    .into()
}
