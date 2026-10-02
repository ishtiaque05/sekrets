//! Visual design tokens and reusable `iced` style functions for the Sekrets GUI.
//!
//! Every screen in `screen::view` builds its widgets from these instead of improvising
//! colors/spacing inline, so the app reads as one designed surface rather than a stack
//! of unrelated default-styled widgets.

use iced::widget::{button, container, text};
use iced::{Background, Border, Color, Theme};

pub const SPACE_XS: f32 = 6.0;
pub const SPACE_SM: f32 = 10.0;
pub const SPACE_MD: f32 = 16.0;
pub const SPACE_LG: f32 = 24.0;
pub const SPACE_XL: f32 = 40.0;

/// Width of a single-column form: auth screens, edit forms, confirmation prompts.
pub const FORM_WIDTH: f32 = 400.0;

pub const TITLE_SIZE: u16 = 26;
pub const HEADING_SIZE: u16 = 18;
pub const BODY_SIZE: u16 = 15;
pub const CAPTION_SIZE: u16 = 13;
pub const PASSWORD_SIZE: u16 = 20;

/// A cool charcoal base with a single cyan-teal accent reserved for primary actions —
/// kept off chrome (headers, cards, secondary buttons) so it still reads as an accent
/// rather than a wash of color, and out of "generic AI dark mode" territory by pairing
/// it with a warm coral danger color instead of pure red.
pub fn theme() -> Theme {
    Theme::custom(
        "Sekrets".to_string(),
        iced::theme::Palette {
            background: Color::from_rgb8(0x14, 0x16, 0x1b),
            text: Color::from_rgb8(0xe7, 0xe8, 0xec),
            primary: Color::from_rgb8(0x4f, 0xd6, 0xc0),
            success: Color::from_rgb8(0x6e, 0xe7, 0xa0),
            danger: Color::from_rgb8(0xf2, 0x68, 0x5b),
        },
    )
}

/// The bar running across the top of every unlocked-app screen.
pub fn header_bar(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style {
        background: Some(Background::Color(palette.background.weak.color)),
        border: Border {
            width: 0.0,
            radius: 0.0.into(),
            color: Color::TRANSPARENT,
        },
        ..container::Style::default()
    }
}

/// A raised card used to frame auth forms and confirmation prompts.
pub fn card(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style {
        background: Some(Background::Color(palette.background.weak.color)),
        border: Border {
            width: 1.0,
            radius: 12.0.into(),
            color: palette.background.strong.color,
        },
        ..container::Style::default()
    }
}

/// The monospace "readout" that displays a credential's password, masked or revealed —
/// the app's one signature element, reused everywhere a password is shown or entered
/// as a preview.
pub fn password_chip(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style {
        background: Some(Background::Color(palette.background.base.color)),
        border: Border {
            width: 1.0,
            radius: 8.0.into(),
            color: palette.background.strong.color,
        },
        ..container::Style::default()
    }
}

/// A single row in the credential list: transparent at rest, a faint highlight on
/// hover/press, so the list reads as rows rather than a stack of bordered buttons.
pub fn list_row(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => palette.background.weak.color,
        _ => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: palette.background.base.text,
        border: Border {
            width: 0.0,
            radius: 8.0.into(),
            color: Color::TRANSPARENT,
        },
        ..button::Style::default()
    }
}

/// A low-emphasis button for navigation (Back / Cancel) and secondary header actions
/// (Versions, Change master password) — so they never compete with a screen's one
/// primary action.
pub fn ghost_button(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::text(theme, status);
    style.text_color = theme
        .extended_palette()
        .background
        .base
        .text
        .scale_alpha(0.75);
    style
}

pub fn muted_text(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.palette().text.scale_alpha(0.6)),
    }
}

pub fn danger_text(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.extended_palette().danger.base.color),
    }
}

pub fn success_text(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.extended_palette().success.base.color),
    }
}
