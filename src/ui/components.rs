use crate::ui::icons;
use crate::ui::theme::{self, Palette, FONT_MEDIUM, FONT_REGULAR, FONT_SEMIBOLD};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::image::{self, Handle as ImageHandle};
use iced::widget::{button, column, container, mouse_area, row, text, text_input, Space};
use iced::{Color, Element, Length, Padding};

pub fn page_header<'a, M: 'a>(
    p: Palette,
    title: &'a str,
    subtitle: &'a str,
    actions: Element<'a, M>,
) -> Element<'a, M> {
    let mut left = column![text(title)
        .font(FONT_SEMIBOLD)
        .size(20)
        .color(p.text_primary)]
    .spacing(2);

    if !subtitle.is_empty() {
        left = left.push(
            text(subtitle)
                .font(FONT_REGULAR)
                .size(12)
                .color(p.text_secondary),
        );
    }

    container(
        row![left, Space::new().width(Length::Fill), actions]
            .align_y(Vertical::Center)
            .spacing(12),
    )
    .width(Length::Fill)
    .padding(Padding {
        top: 20.0,
        right: 24.0,
        bottom: 14.0,
        left: 24.0,
    })
    .into()
}

pub fn app_logo<'a, M: 'a>(
    handle: Option<&'a ImageHandle>,
    name: &str,
    size: f32,
    p: Palette,
) -> Element<'a, M> {
    if let Some(h) = handle {
        container(
            image::Image::new(h.clone())
                .width(Length::Fixed(size))
                .height(Length::Fixed(size))
                .content_fit(iced::ContentFit::Contain),
        )
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .center_x(Length::Fixed(size))
        .center_y(Length::Fixed(size))
        .into()
    } else {
        let initial: String = name
            .chars()
            .find(|c| c.is_alphanumeric())
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_else(|| "•".into());
        container(
            text(initial)
                .font(FONT_SEMIBOLD)
                .size((size * 0.44).max(11.0))
                .color(p.text_secondary),
        )
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .center_x(Length::Fixed(size))
        .center_y(Length::Fixed(size))
        .style(theme::monogram_tile_style(p))
        .into()
    }
}

pub fn checkbox_box<'a, M: 'a>(checked: bool, p: Palette) -> Element<'a, M> {
    container(if checked {
        Element::from(icons::check(10.0, p.accent_fg))
    } else {
        Space::new()
            .width(Length::Fixed(10.0))
            .height(Length::Fixed(10.0))
            .into()
    })
    .width(Length::Fixed(16.0))
    .height(Length::Fixed(16.0))
    .center_x(Length::Fixed(16.0))
    .center_y(Length::Fixed(16.0))
    .style(theme::checkbox_tile_style(p, checked))
    .into()
}

pub fn status_dot<'a, M: 'a>(color: Color, label: Option<&'a str>, p: Palette) -> Element<'a, M> {
    let dot = container(Space::new())
        .width(Length::Fixed(6.0))
        .height(Length::Fixed(6.0))
        .style(theme::dot_style(color));

    if let Some(lbl) = label {
        row![
            dot,
            text(lbl)
                .font(FONT_MEDIUM)
                .size(11.5)
                .color(p.text_secondary),
        ]
        .spacing(6)
        .align_y(Vertical::Center)
        .into()
    } else {
        dot.into()
    }
}

/// Clean borderless list stack (zero visible row separators, subtle 3px breathing rhythm).
pub fn clean_list<'a, M: 'a>(rows: Vec<Element<'a, M>>) -> Element<'a, M> {
    let mut col = column![].spacing(3).width(Length::Fill);
    for r in rows {
        col = col.push(r);
    }
    col.into()
}

/// Subtle panel container with soft inner padding and zero hard horizontal row separators.
pub fn soft_panel<'a, M: 'a>(p: Palette, rows: Vec<Element<'a, M>>) -> Element<'a, M> {
    let mut col = column![].spacing(2).width(Length::Fill);
    for item in rows {
        col = col.push(item);
    }
    container(col)
        .width(Length::Fill)
        .padding(6)
        .style(theme::surface_panel_style(p))
        .into()
}

/// Full-width borderless row with smooth 60fps hover fade and zero corner-bleed selection.
pub fn hover_row<'a, M: Clone + 'a>(
    p: Palette,
    hover_t: f32,
    selected: bool,
    on_enter: M,
    on_exit: M,
    content: Element<'a, M>,
) -> Element<'a, M> {
    mouse_area(
        container(content)
            .width(Length::Fill)
            .style(theme::animated_row_style(p, selected, hover_t)),
    )
    .on_enter(on_enter)
    .on_exit(on_exit)
    .into()
}

pub fn section_heading<'a, M: 'a>(
    p: Palette,
    title: &'a str,
    trailing: Option<String>,
) -> Element<'a, M> {
    let mut r = row![text(title)
        .font(FONT_SEMIBOLD)
        .size(12)
        .color(p.text_secondary)]
    .align_y(Vertical::Center);

    if let Some(meta) = trailing {
        r = r
            .push(Space::new().width(Length::Fill))
            .push(text(meta).font(FONT_REGULAR).size(11).color(p.text_muted));
    }

    container(r)
        .width(Length::Fill)
        .padding(Padding {
            top: 4.0,
            right: 6.0,
            bottom: 6.0,
            left: 6.0,
        })
        .into()
}

pub fn primary_button<'a, M: Clone + 'a>(
    p: Palette,
    label: String,
    icon: Option<Element<'a, M>>,
    on_press: Option<M>,
) -> Element<'a, M> {
    let content: Element<'a, M> = if let Some(ic) = icon {
        row![ic, text(label).font(FONT_SEMIBOLD).size(12.5)]
            .spacing(6)
            .align_y(Vertical::Center)
            .into()
    } else {
        text(label).font(FONT_SEMIBOLD).size(12.5).into()
    };

    let mut b = button(content)
        .padding([7, 13])
        .style(theme::btn_primary(p));
    if let Some(m) = on_press {
        b = b.on_press(m);
    }
    b.into()
}

pub fn secondary_button<'a, M: Clone + 'a>(
    p: Palette,
    label: String,
    icon: Option<Element<'a, M>>,
    on_press: Option<M>,
) -> Element<'a, M> {
    let content: Element<'a, M> = if let Some(ic) = icon {
        row![ic, text(label).font(FONT_MEDIUM).size(12.5)]
            .spacing(6)
            .align_y(Vertical::Center)
            .into()
    } else {
        text(label).font(FONT_MEDIUM).size(12.5).into()
    };

    let mut b = button(content)
        .padding([7, 12])
        .style(theme::btn_secondary(p));
    if let Some(m) = on_press {
        b = b.on_press(m);
    }
    b.into()
}

pub fn destructive_button<'a, M: Clone + 'a>(
    p: Palette,
    label: String,
    icon: Option<Element<'a, M>>,
    on_press: Option<M>,
) -> Element<'a, M> {
    let content: Element<'a, M> = if let Some(ic) = icon {
        row![ic, text(label).font(FONT_SEMIBOLD).size(12.5)]
            .spacing(6)
            .align_y(Vertical::Center)
            .into()
    } else {
        text(label).font(FONT_SEMIBOLD).size(12.5).into()
    };

    let mut b = button(content)
        .padding([7, 13])
        .style(theme::btn_destructive(p));
    if let Some(m) = on_press {
        b = b.on_press(m);
    }
    b.into()
}

pub fn destructive_subtle_button<'a, M: Clone + 'a>(
    p: Palette,
    label: String,
    icon: Option<Element<'a, M>>,
    on_press: Option<M>,
) -> Element<'a, M> {
    let content: Element<'a, M> = if let Some(ic) = icon {
        row![ic, text(label).font(FONT_MEDIUM).size(12.5)]
            .spacing(6)
            .align_y(Vertical::Center)
            .into()
    } else {
        text(label).font(FONT_MEDIUM).size(12.5).into()
    };

    let mut b = button(content)
        .padding([7, 12])
        .style(theme::btn_destructive_subtle(p));
    if let Some(m) = on_press {
        b = b.on_press(m);
    }
    b.into()
}

pub fn search_bar<'a, M: Clone + 'a>(
    p: Palette,
    placeholder: &'a str,
    value: &'a str,
    on_input: impl Fn(String) -> M + 'a,
    on_clear: M,
) -> Element<'a, M> {
    let mut r = row![
        icons::search(14.0, p.text_muted),
        text_input(placeholder, value)
            .on_input(on_input)
            .font(FONT_REGULAR)
            .size(12.5)
            .padding([5, 4])
            .width(Length::Fill)
            .style(move |t, s| {
                let mut st = theme::input_style(p)(t, s);
                st.border.width = 0.0;
                st.background = iced::Background::Color(Color::TRANSPARENT);
                st
            }),
    ]
    .spacing(7)
    .align_y(Vertical::Center);

    if !value.is_empty() {
        r = r.push(
            button(icons::close(12.0, p.text_muted))
                .padding([3, 3])
                .style(theme::btn_ghost(p))
                .on_press(on_clear),
        );
    }

    container(r)
        .padding([2, 10])
        .width(Length::Fill)
        .style(theme::well_style(p))
        .into()
}

pub fn segmented_control<'a, M: Clone + 'a>(
    p: Palette,
    items: Vec<(&'a str, bool, M)>,
) -> Element<'a, M> {
    let mut r = row![].spacing(2).align_y(Vertical::Center);
    for (label, active, msg) in items {
        r = r.push(
            button(
                text(label)
                    .font(if active { FONT_SEMIBOLD } else { FONT_MEDIUM })
                    .size(11.5),
            )
            .padding([5, 10])
            .style(theme::btn_segment(p, active))
            .on_press(msg),
        );
    }
    container(r)
        .padding(2)
        .style(theme::segmented_track_style(p))
        .into()
}

pub fn h_divider<'a, M: 'a>(p: Palette) -> Element<'a, M> {
    container(Space::new())
        .width(Length::Fill)
        .height(Length::Fixed(1.0))
        .style(theme::hairline_style(p))
        .into()
}

pub fn empty_state<'a, M: 'a>(
    p: Palette,
    icon: Element<'a, M>,
    title: &'a str,
    subtitle: &'a str,
) -> Element<'a, M> {
    let mut col = column![
        icon,
        text(title)
            .font(FONT_SEMIBOLD)
            .size(15)
            .color(p.text_primary),
    ]
    .spacing(8)
    .align_x(Horizontal::Center);

    if !subtitle.is_empty() {
        col = col.push(
            text(subtitle)
                .font(FONT_REGULAR)
                .size(12.5)
                .color(p.text_secondary),
        );
    }

    container(col)
        .width(Length::Fill)
        .padding([56, 24])
        .center_x(Length::Fill)
        .into()
}

pub fn modal_shell<'a, M: Clone + 'a>(
    p: Palette,
    alpha: f32,
    width: f32,
    title: &'a str,
    subtitle: String,
    body: Element<'a, M>,
    footer: Element<'a, M>,
    on_dismiss: Option<M>,
) -> Element<'a, M> {
    let mut header_row = row![
        column![
            text(title)
                .font(FONT_SEMIBOLD)
                .size(15.5)
                .color(p.text_primary),
            text(subtitle)
                .font(FONT_REGULAR)
                .size(12)
                .color(p.text_secondary),
        ]
        .spacing(2)
        .width(Length::Fill),
    ]
    .align_y(Vertical::Center);

    if let Some(dismiss_msg) = on_dismiss.clone() {
        header_row = header_row.push(
            button(icons::close(14.0, p.text_muted))
                .padding([5, 5])
                .style(theme::btn_ghost(p))
                .on_press(dismiss_msg),
        );
    }

    let sheet = container(
        column![
            container(header_row).padding(Padding {
                top: 18.0,
                right: 20.0,
                bottom: 14.0,
                left: 20.0,
            }),
            h_divider(p),
            container(body).padding([16, 20]),
            h_divider(p),
            container(footer).padding([14, 20]),
        ],
    )
    .width(Length::Fixed(width))
    .style(theme::modal_sheet_style(p));

    let sheet_guard: Element<'a, M> = mouse_area(sheet).into();

    let offset_top = (1.0 - alpha.clamp(0.0, 1.0)) * 10.0;
    let backdrop_content = container(
        column![
            Space::new().height(Length::Fixed(offset_top)),
            sheet_guard,
        ]
        .align_x(Horizontal::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .style(theme::modal_scrim_style(p, alpha));

    if let Some(dismiss_msg) = on_dismiss {
        mouse_area(backdrop_content).on_press(dismiss_msg).into()
    } else {
        backdrop_content.into()
    }
}
