use iced::Color;
use iced::widget::container;

pub(crate) fn container_bg(theme: &iced::Theme, alpha: f32) -> container::Style {
    let palette = theme.palette();
    container::Style {
        background: Some(iced::Background::Color(
            palette.background.scale_alpha(alpha),
        )),
        border: iced::Border {
            width: 0.0,
            color: palette.background,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

pub(crate) fn outlined_container_bg(
    theme: &iced::Theme,
    bg_alpha: f32,
    border_alpha: f32,
) -> container::Style {
    let palette = theme.palette();
    container::Style {
        background: Some(iced::Background::Color(
            palette.background.scale_alpha(bg_alpha),
        )),
        border: iced::Border {
            width: 1.0,
            color: palette.background.scale_alpha(border_alpha),
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

pub(crate) fn tooltip_container_style(theme: &iced::Theme) -> container::Style {
    let palette = theme.palette();
    container::Style {
        background: Some(iced::Background::Color(Color::from_rgba(
            0.08, 0.09, 0.12, 0.97,
        ))),
        text_color: Some(Color::WHITE),
        border: iced::Border {
            width: 1.0,
            color: palette.background.scale_alpha(0.75),
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

pub(crate) fn modal_backdrop_style_with_alpha(alpha: f32) -> impl Fn(&iced::Theme) -> container::Style {
    move |_theme: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(Color::from_rgba(
            0.02,
            0.03,
            0.05,
            0.78 * alpha.clamp(0.0, 1.0),
        ))),
        ..Default::default()
    }
}

pub(crate) fn modal_card_style_with_alpha(
    alpha: f32,
) -> impl Fn(&iced::Theme) -> container::Style {
    move |theme: &iced::Theme| {
        let palette = theme.palette();
        container::Style {
            background: Some(iced::Background::Color(Color::from_rgba(
                0.10,
                0.12,
                0.17,
                0.98 * alpha.clamp(0.0, 1.0),
            ))),
            text_color: Some(Color::from_rgba(1.0, 1.0, 1.0, alpha.clamp(0.0, 1.0))),
            border: iced::Border {
                width: 1.0,
                color: palette.primary.scale_alpha(0.45 * alpha.clamp(0.0, 1.0)),
                radius: 12.0.into(),
            },
            ..Default::default()
        }
    }
}
