use crate::ui::icons;
use crate::ui::theme::{self, Palette, FONT_MEDIUM, FONT_REGULAR, FONT_SEMIBOLD};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::image::{self, Handle as ImageHandle};
use iced::widget::{button, column, container, mouse_area, row, text, text_input, Space};
use iced::{Color, Element, Length, Padding};

/// Clean Apple-style view header with concise title, optional single-line subtitle, and trailing CTA.
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

/// Renders a real extracted/cached application logo, or a restrained monochrome monogram while loading.
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

/// Precision 16x16 macOS-style checkbox indicator.
pub fn checkbox_box<'a, M: 'a>(checked: bool, p: Palette) -> Element<'a, M> {
    container(if checked {
        Element::from(icons::check(10.0, Color::WHITE))
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

/// Quiet semantic status dot + optional crisp text (replaces noisy pill badges).
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
                .size(11)
                .color(p.text_secondary),
        ]
        .spacing(6)
        .align_y(Vertical::Center)
        .into()
    } else {
        dot.into()
    }
}

/// Stitches rows into a single macOS System Settings inset grouped surface with 1px hairline dividers.
pub fn inset_group<'a, M: 'a>(p: Palette, rows: Vec<Element<'a, M>>) -> Element<'a, M> {
    let mut col = column![];
    let total = rows.len();
    for (idx, item) in rows.into_iter().enumerate() {
        col = col.push(item);
        if idx + 1 < total {
            col = col.push(
                container(
                    container(Space::new())
                        .width(Length::Fill)
                        .height(Length::Fixed(1.0))
                        .style(theme::hairline_style(p)),
                )
                .padding([0, 14]),
            );
        }
    }

    container(col)
        .width(Length::Fill)
        .style(theme::inset_group_style(p))
        .into()
}

/// Quiet section label placed above an `inset_group`.
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
            right: 4.0,
            bottom: 6.0,
            left: 4.0,
        })
        .into()
}

/// Primary accent button.
pub fn primary_button<'a, M: Clone + 'a>(
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
        .padding([7, 13])
        .style(theme::btn_primary(p));
    if let Some(m) = on_press {
        b = b.on_press(m);
    }
    b.into()
}

/// Secondary crisp-bordered button.
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

/// Destructive solid red confirmation button (used inside confirmation modals / primary destructive CTAs).
pub fn destructive_button<'a, M: Clone + 'a>(
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
        .padding([7, 13])
        .style(theme::btn_destructive(p));
    if let Some(m) = on_press {
        b = b.on_press(m);
    }
    b.into()
}

/// Subtle red-tinted destructive button for toolbar actions before confirmation.
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

/// Clean macOS-style search field.
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

/// macOS Segmented Control container.
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

/// Full-width 1px structural divider.
pub fn h_divider<'a, M: 'a>(p: Palette) -> Element<'a, M> {
    container(Space::new())
        .width(Length::Fill)
        .height(Length::Fixed(1.0))
        .style(theme::border_divider_style(p))
        .into()
}

/// Clean centered empty state.
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

/// Centered, proportion-aware macOS sheet/modal with animated scrim and trailing action footer.
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
                .size(16)
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
            container(Space::new())
                .width(Length::Fill)
                .height(Length::Fixed(1.0))
                .style(theme::hairline_style(p)),
            container(body).padding([16, 20]),
            container(Space::new())
                .width(Length::Fill)
                .height(Length::Fixed(1.0))
                .style(theme::hairline_style(p)),
            container(footer).padding([14, 20]),
        ],
    )
    .width(Length::Fixed(width))
    .style(theme::modal_sheet_style(p));

    // Stop click propagation through the sheet itself
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
