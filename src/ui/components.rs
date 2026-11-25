use crate::ui::icons;
use crate::ui::theme::{self, Palette, FONT_MEDIUM, FONT_REGULAR, FONT_SEMIBOLD};
use iced::alignment::Vertical;
use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Color, Element, Length, Padding};

/// Clean Apple/Moonsworth page header with title, subtitle, and right-aligned action controls.
pub fn page_header<'a, M: 'a>(
    p: Palette,
    title: &'a str,
    subtitle: &'a str,
    actions: Element<'a, M>,
) -> Element<'a, M> {
    let left = column![
        text(title)
            .font(FONT_SEMIBOLD)
            .size(22)
            .color(p.text_primary),
        text(subtitle)
            .font(FONT_REGULAR)
            .size(13)
            .color(p.text_secondary),
    ]
    .spacing(3);

    container(
        row![left, Space::new().width(Length::Fill), actions]
            .align_y(Vertical::Center)
            .spacing(12),
    )
    .width(Length::Fill)
    .padding(Padding {
        top: 22.0,
        right: 26.0,
        bottom: 16.0,
        left: 26.0,
    })
    .into()
}

/// Crisp small status pill badge (e.g., "Installed", "Direct", "Safe", "Admin").
pub fn tag_pill<'a, M: 'a>(label: impl Into<String>, bg: Color, fg: Color) -> Element<'a, M> {
    container(
        text(label.into())
            .font(FONT_MEDIUM)
            .size(11)
            .color(fg),
    )
    .padding([2, 8])
    .style(theme::pill_badge_style(bg, fg))
    .into()
}

/// Primary accent CTA button with optional icon.
pub fn primary_button<'a, M: Clone + 'a>(
    p: Palette,
    label: String,
    icon: Option<Element<'a, M>>,
    on_press: Option<M>,
) -> Element<'a, M> {
    let content: Element<'a, M> = if let Some(ic) = icon {
        row![
            ic,
            text(label)
                .font(FONT_MEDIUM)
                .size(13)
        ]
        .spacing(7)
        .align_y(Vertical::Center)
        .into()
    } else {
        text(label).font(FONT_MEDIUM).size(13).into()
    };

    let mut b = button(content)
        .padding([8, 14])
        .style(theme::btn_primary(p));
    if let Some(m) = on_press {
        b = b.on_press(m);
    }
    b.into()
}

/// Secondary subtle-surface button with 1px crisp border.
pub fn secondary_button<'a, M: Clone + 'a>(
    p: Palette,
    label: String,
    icon: Option<Element<'a, M>>,
    on_press: Option<M>,
) -> Element<'a, M> {
    let content: Element<'a, M> = if let Some(ic) = icon {
        row![ic, text(label).font(FONT_MEDIUM).size(13)]
            .spacing(7)
            .align_y(Vertical::Center)
            .into()
    } else {
        text(label).font(FONT_MEDIUM).size(13).into()
    };

    let mut b = button(content)
        .padding([8, 13])
        .style(theme::btn_secondary(p));
    if let Some(m) = on_press {
        b = b.on_press(m);
    }
    b.into()
}

/// Danger tinted button (turns solid crimson on hover).
pub fn danger_button<'a, M: Clone + 'a>(
    p: Palette,
    label: String,
    icon: Option<Element<'a, M>>,
    on_press: Option<M>,
) -> Element<'a, M> {
    let content: Element<'a, M> = if let Some(ic) = icon {
        row![ic, text(label).font(FONT_MEDIUM).size(13)]
            .spacing(7)
            .align_y(Vertical::Center)
            .into()
    } else {
        text(label).font(FONT_MEDIUM).size(13).into()
    };

    let mut b = button(content)
        .padding([8, 13])
        .style(theme::btn_danger(p));
    if let Some(m) = on_press {
        b = b.on_press(m);
    }
    b.into()
}

/// Clean search field with magnifying glass icon and optional clear button.
pub fn search_bar<'a, M: Clone + 'a>(
    p: Palette,
    placeholder: &'a str,
    value: &'a str,
    on_input: impl Fn(String) -> M + 'a,
    on_clear: M,
) -> Element<'a, M> {
    let mut r = row![
        icons::search(15.0, p.text_muted),
        text_input(placeholder, value)
            .on_input(on_input)
            .font(FONT_REGULAR)
            .size(13)
            .padding([6, 4])
            .width(Length::Fill)
            .style(move |t, s| {
                let mut st = theme::input_style(p)(t, s);
                st.border.width = 0.0;
                st.background = iced::Background::Color(Color::TRANSPARENT);
                st
            }),
    ]
    .spacing(8)
    .align_y(Vertical::Center);

    if !value.is_empty() {
        r = r.push(
            button(icons::close(13.0, p.text_muted))
                .padding([4, 4])
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

/// Stat metric card for summaries (Cleaner, Settings).
pub fn stat_card<'a, M: 'a>(
    p: Palette,
    label: &'a str,
    value: String,
    accent: Color,
) -> Element<'a, M> {
    container(
        column![
            text(label)
                .font(FONT_MEDIUM)
                .size(12)
                .color(p.text_secondary),
            text(value).font(FONT_SEMIBOLD).size(22).color(accent),
        ]
        .spacing(4),
    )
    .width(Length::Fill)
    .padding([14, 16])
    .style(theme::card_style(p))
    .into()
}

/// Subtle horizontal 1px divider.
pub fn h_divider<'a, M: 'a>(p: Palette) -> Element<'a, M> {
    container(Space::new())
        .width(Length::Fill)
        .height(Length::Fixed(1.0))
        .style(theme::divider_style(p))
        .into()
}

/// Clean centered empty state with icon badge, title, and subtitle.
pub fn empty_state<'a, M: 'a>(
    p: Palette,
    icon: Element<'a, M>,
    title: &'a str,
    subtitle: &'a str,
) -> Element<'a, M> {
    container(
        column![
            container(icon)
                .padding(14)
                .style(theme::pill_badge_style(p.bg_surface, p.text_secondary)),
            text(title)
                .font(FONT_SEMIBOLD)
                .size(16)
                .color(p.text_primary),
            text(subtitle)
                .font(FONT_REGULAR)
                .size(13)
                .color(p.text_secondary),
        ]
        .spacing(8)
        .align_x(iced::alignment::Horizontal::Center),
    )
    .width(Length::Fill)
    .padding([48, 24])
    .center_x(Length::Fill)
    .into()
}
