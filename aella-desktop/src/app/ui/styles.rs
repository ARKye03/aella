use iced::widget::{button, container, text_editor, text_input};
use iced::{Background, Border, Color, Shadow, Vector};

pub(crate) const SHELL_BG: Color = Color::from_rgb(
    0x0D as f32 / 255.0,
    0x11 as f32 / 255.0,
    0x17 as f32 / 255.0,
);
pub(crate) const PANEL_BG: Color = Color::from_rgb(
    0x16 as f32 / 255.0,
    0x1B as f32 / 255.0,
    0x22 as f32 / 255.0,
);
pub(crate) const PANEL_BG_SOFT: Color = Color::from_rgb(
    0x1B as f32 / 255.0,
    0x22 as f32 / 255.0,
    0x2C as f32 / 255.0,
);
pub(crate) const BORDER_SUBTLE: Color = Color::from_rgb(
    0x30 as f32 / 255.0,
    0x36 as f32 / 255.0,
    0x3D as f32 / 255.0,
);
pub(crate) const ACCENT: Color = Color::from_rgb(
    0x5E as f32 / 255.0,
    0x5C as f32 / 255.0,
    0xE6 as f32 / 255.0,
);
pub(crate) const TEXT_HIGH: Color = Color::from_rgb(
    0xFF as f32 / 255.0,
    0xFF as f32 / 255.0,
    0xFF as f32 / 255.0,
);
pub(crate) const TEXT_BODY: Color = Color::from_rgb(
    0xC9 as f32 / 255.0,
    0xD1 as f32 / 255.0,
    0xD9 as f32 / 255.0,
);
pub(crate) const TEXT_LOW: Color = Color::from_rgb(
    0x8B as f32 / 255.0,
    0x94 as f32 / 255.0,
    0x9E as f32 / 255.0,
);

fn with_alpha(color: Color, alpha: f32) -> Color {
    Color { a: alpha, ..color }
}

pub(crate) fn app_shell_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(SHELL_BG)),
        text_color: Some(TEXT_BODY),
        ..Default::default()
    }
}

pub(crate) fn content_card_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(PANEL_BG)),
        text_color: Some(TEXT_BODY),
        border: Border {
            width: 1.0,
            color: BORDER_SUBTLE,
            radius: 12.0.into(),
        },
        shadow: Shadow {
            color: with_alpha(Color::BLACK, 0.28),
            offset: Vector::new(0.0, 6.0),
            blur_radius: 18.0,
        },
        ..Default::default()
    }
}

pub(crate) fn sidebar_shell_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(with_alpha(PANEL_BG, 0.92))),
        text_color: Some(TEXT_BODY),
        border: Border {
            width: 1.0,
            color: BORDER_SUBTLE,
            radius: 12.0.into(),
        },
        ..Default::default()
    }
}

pub(crate) fn sidebar_toggle_button_style(
    _theme: &iced::Theme,
    status: button::Status,
) -> button::Style {
    let mut style = button::Style {
        text_color: TEXT_BODY,
        border: Border {
            width: 1.0,
            color: with_alpha(BORDER_SUBTLE, 0.8),
            radius: 10.0.into(),
        },
        background: Some(Background::Color(with_alpha(PANEL_BG_SOFT, 0.88))),
        ..Default::default()
    };

    if matches!(status, button::Status::Hovered) {
        style.background = Some(Background::Color(with_alpha(PANEL_BG_SOFT, 1.0)));
        style.border.color = ACCENT;
    }

    style
}

pub(crate) fn sidebar_item_style(
    selected: bool,
) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_theme: &iced::Theme, status: button::Status| {
        let (background, border_color, text_color) = if selected {
            (
                with_alpha(ACCENT, 0.22),
                with_alpha(ACCENT, 0.72),
                Color::from_rgb(0.93, 0.93, 1.0),
            )
        } else {
            (
                Color::TRANSPARENT,
                with_alpha(BORDER_SUBTLE, 0.0),
                TEXT_BODY,
            )
        };

        let mut style = button::Style {
            text_color,
            background: Some(Background::Color(background)),
            border: Border {
                width: if selected { 1.0 } else { 0.0 },
                color: border_color,
                radius: 10.0.into(),
            },
            ..Default::default()
        };

        if matches!(status, button::Status::Hovered) {
            style.background = Some(Background::Color(with_alpha(PANEL_BG_SOFT, 0.9)));
            style.border.width = 1.0;
            style.border.color = if selected {
                with_alpha(ACCENT, 0.85)
            } else {
                with_alpha(BORDER_SUBTLE, 0.75)
            };
        }

        if matches!(status, button::Status::Pressed) {
            style.background = Some(Background::Color(with_alpha(ACCENT, 0.26)));
        }

        if matches!(status, button::Status::Disabled) {
            style.text_color = with_alpha(TEXT_LOW, 0.55);
            style.background = Some(Background::Color(with_alpha(PANEL_BG, 0.35)));
        }

        style
    }
}

pub(crate) fn compact_avatar_button_style(
    selected: bool,
    tint: Color,
) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_theme: &iced::Theme, status: button::Status| {
        let tint_bg = with_alpha(tint, 0.16);
        let tint_border = with_alpha(tint, 0.34);

        let mut style = button::Style {
            text_color: if selected {
                Color::from_rgb(0.93, 0.93, 1.0)
            } else {
                TEXT_BODY
            },
            background: Some(Background::Color(if selected {
                with_alpha(ACCENT, 0.24)
            } else {
                tint_bg
            })),
            border: Border {
                width: 1.0,
                color: if selected {
                    with_alpha(ACCENT, 0.85)
                } else {
                    tint_border
                },
                radius: 999.0.into(),
            },
            ..Default::default()
        };

        if matches!(status, button::Status::Hovered) {
            style.background = Some(Background::Color(if selected {
                with_alpha(ACCENT, 0.34)
            } else {
                with_alpha(tint, 0.24)
            }));
            style.border.color = if selected {
                with_alpha(ACCENT, 0.95)
            } else {
                with_alpha(tint, 0.46)
            };
            style.text_color = TEXT_HIGH;
        }

        if matches!(status, button::Status::Pressed) {
            style.background = Some(Background::Color(with_alpha(ACCENT, 0.42)));
            style.text_color = TEXT_HIGH;
        }

        style
    }
}

pub(crate) fn segmented_button_style(
    selected: bool,
) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_theme: &iced::Theme, status: button::Status| {
        let mut style = button::Style {
            text_color: if selected { TEXT_HIGH } else { TEXT_LOW },
            background: Some(Background::Color(if selected {
                with_alpha(ACCENT, 0.9)
            } else {
                with_alpha(PANEL_BG_SOFT, 0.7)
            })),
            border: Border {
                width: 1.0,
                color: if selected {
                    with_alpha(ACCENT, 0.95)
                } else {
                    with_alpha(BORDER_SUBTLE, 0.8)
                },
                radius: 10.0.into(),
            },
            ..Default::default()
        };

        if matches!(status, button::Status::Hovered) {
            style.background = Some(Background::Color(if selected {
                with_alpha(ACCENT, 1.0)
            } else {
                with_alpha(PANEL_BG_SOFT, 0.95)
            }));
            style.text_color = if selected { TEXT_HIGH } else { TEXT_BODY };
        }

        if matches!(status, button::Status::Disabled) {
            style.text_color = with_alpha(TEXT_LOW, 0.55);
            style.background = Some(Background::Color(with_alpha(PANEL_BG_SOFT, 0.45)));
            style.border.color = with_alpha(BORDER_SUBTLE, 0.45);
        }

        style
    }
}

pub(crate) fn ghost_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let mut style = button::Style {
        text_color: TEXT_LOW,
        background: Some(Background::Color(Color::TRANSPARENT)),
        border: Border {
            width: 1.0,
            color: with_alpha(BORDER_SUBTLE, 0.0),
            radius: 9.0.into(),
        },
        ..Default::default()
    };

    if matches!(status, button::Status::Hovered) {
        style.background = Some(Background::Color(with_alpha(PANEL_BG_SOFT, 0.8)));
        style.border.color = with_alpha(BORDER_SUBTLE, 0.85);
        style.text_color = TEXT_BODY;
    }

    if matches!(status, button::Status::Pressed) {
        style.background = Some(Background::Color(with_alpha(ACCENT, 0.24)));
        style.text_color = TEXT_HIGH;
    }

    style
}

pub(crate) fn danger_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let mut style = button::Style {
        text_color: Color::from_rgb(1.0, 0.82, 0.82),
        background: Some(Background::Color(with_alpha(
            Color::from_rgb(0.55, 0.16, 0.20),
            0.35,
        ))),
        border: Border {
            width: 1.0,
            color: with_alpha(Color::from_rgb(0.75, 0.3, 0.35), 0.75),
            radius: 9.0.into(),
        },
        ..Default::default()
    };

    if matches!(status, button::Status::Hovered) {
        style.background = Some(Background::Color(with_alpha(
            Color::from_rgb(0.62, 0.18, 0.22),
            0.58,
        )));
    }

    style
}

pub(crate) fn title_trash_button_style(
    _theme: &iced::Theme,
    status: button::Status,
) -> button::Style {
    let mut style = button::Style {
        text_color: TEXT_LOW,
        background: Some(Background::Color(Color::TRANSPARENT)),
        border: Border {
            width: 1.0,
            color: with_alpha(BORDER_SUBTLE, 0.0),
            radius: 9.0.into(),
        },
        ..Default::default()
    };

    if matches!(status, button::Status::Hovered) {
        style.text_color = Color::from_rgb(1.0, 0.79, 0.79);
        style.background = Some(Background::Color(with_alpha(
            Color::from_rgb(0.62, 0.18, 0.22),
            0.2,
        )));
        style.border.color = with_alpha(Color::from_rgb(0.75, 0.3, 0.35), 0.6);
    }

    if matches!(status, button::Status::Pressed) {
        style.text_color = Color::from_rgb(1.0, 0.84, 0.84);
        style.background = Some(Background::Color(with_alpha(
            Color::from_rgb(0.62, 0.18, 0.22),
            0.34,
        )));
        style.border.color = with_alpha(Color::from_rgb(0.82, 0.34, 0.39), 0.76);
    }

    style
}

pub(crate) fn command_input_style(
    _theme: &iced::Theme,
    status: text_input::Status,
) -> text_input::Style {
    let mut border = Border {
        radius: 12.0.into(),
        width: 1.0,
        color: with_alpha(BORDER_SUBTLE, 0.9),
    };

    if matches!(status, text_input::Status::Focused { .. }) {
        border.color = with_alpha(ACCENT, 0.92);
    }

    text_input::Style {
        background: Background::Color(with_alpha(PANEL_BG_SOFT, 0.95)),
        border,
        icon: TEXT_LOW,
        placeholder: TEXT_LOW,
        value: TEXT_BODY,
        selection: with_alpha(ACCENT, 0.35),
    }
}

pub(crate) fn document_title_input_style(
    _theme: &iced::Theme,
    _status: text_input::Status,
) -> text_input::Style {
    text_input::Style {
        background: Background::Color(Color::TRANSPARENT),
        border: Border {
            radius: 0.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        icon: TEXT_LOW,
        placeholder: with_alpha(TEXT_LOW, 0.85),
        value: TEXT_HIGH,
        selection: with_alpha(ACCENT, 0.35),
    }
}

pub(crate) fn raw_editor_style(
    _theme: &iced::Theme,
    status: text_editor::Status,
) -> text_editor::Style {
    let mut border = Border {
        radius: 12.0.into(),
        width: 1.0,
        color: with_alpha(BORDER_SUBTLE, 0.85),
    };

    if matches!(status, text_editor::Status::Focused { .. }) {
        border.color = with_alpha(ACCENT, 0.75);
    }

    text_editor::Style {
        background: Background::Color(with_alpha(PANEL_BG_SOFT, 0.9)),
        border,
        placeholder: TEXT_LOW,
        value: TEXT_BODY,
        selection: with_alpha(ACCENT, 0.32),
    }
}

pub(crate) fn top_header_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(with_alpha(PANEL_BG, 0.93))),
        border: Border {
            width: 1.0,
            color: with_alpha(BORDER_SUBTLE, 0.8),
            radius: 12.0.into(),
        },
        ..Default::default()
    }
}

pub(crate) fn floating_panel_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(with_alpha(PANEL_BG, 0.96))),
        text_color: Some(TEXT_BODY),
        border: Border {
            width: 1.0,
            color: with_alpha(BORDER_SUBTLE, 0.95),
            radius: 12.0.into(),
        },
        shadow: Shadow {
            color: with_alpha(Color::BLACK, 0.4),
            offset: Vector::new(0.0, 7.0),
            blur_radius: 24.0,
        },
        ..Default::default()
    }
}

pub(crate) fn collapsed_status_pill_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(with_alpha(Color::BLACK, 0.62))),
        text_color: Some(TEXT_BODY),
        border: Border {
            width: 1.0,
            color: with_alpha(BORDER_SUBTLE, 0.5),
            radius: 999.0.into(),
        },
        shadow: Shadow {
            color: with_alpha(Color::BLACK, 0.45),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 14.0,
        },
        ..Default::default()
    }
}

pub(crate) fn footer_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(with_alpha(PANEL_BG, 0.82))),
        border: Border {
            width: 1.0,
            color: with_alpha(BORDER_SUBTLE, 0.72),
            radius: 10.0.into(),
        },
        ..Default::default()
    }
}

pub(crate) fn tooltip_container_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(with_alpha(PANEL_BG, 0.98))),
        text_color: Some(TEXT_BODY),
        border: Border {
            width: 1.0,
            color: with_alpha(BORDER_SUBTLE, 0.95),
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

pub(crate) fn modal_backdrop_style_with_alpha(
    alpha: f32,
) -> impl Fn(&iced::Theme) -> container::Style {
    move |_theme: &iced::Theme| container::Style {
        background: Some(Background::Color(with_alpha(
            Color::from_rgb(0.02, 0.03, 0.05),
            0.78 * alpha.clamp(0.0, 1.0),
        ))),
        ..Default::default()
    }
}

pub(crate) fn modal_card_style_with_alpha(alpha: f32) -> impl Fn(&iced::Theme) -> container::Style {
    move |_theme: &iced::Theme| container::Style {
        background: Some(Background::Color(with_alpha(
            PANEL_BG_SOFT,
            0.98 * alpha.clamp(0.0, 1.0),
        ))),
        text_color: Some(with_alpha(TEXT_HIGH, alpha.clamp(0.0, 1.0))),
        border: Border {
            width: 1.0,
            color: with_alpha(ACCENT, 0.4 * alpha.clamp(0.0, 1.0)),
            radius: 12.0.into(),
        },
        ..Default::default()
    }
}
