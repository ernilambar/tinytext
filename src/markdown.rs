use std::sync::Arc;

use gpui_kit::App;
use gpui_kit::component::Theme;
use gpui_kit::component::highlighter::ThemeStyle;

use crate::settings::{MAX_FONT_WEIGHT, MIN_FONT_WEIGHT};

/// How much heavier headings and `**bold**` render than the editor's base
/// weight. Weights are snapped to the theme's 100-unit scale.
const EMPHASIS_STEP: f32 = 200.;

/// Styles the tree-sitter `@title` (Markdown headings) and `@emphasis.strong`
/// (`**bold**`) tokens [`EMPHASIS_STEP`] above `base_weight`, in the normal
/// foreground.
///
/// Per-token styles win over `editor.font_weight`, so without this the theme's
/// fixed 600/700 would show through and clash with a light or heavy base.
/// Re-run whenever the setting changes.
pub(crate) fn apply_emphasis(cx: &mut App, base_weight: f32) {
    let Ok(style) = serde_json::from_value::<ThemeStyle>(
        serde_json::json!({ "font_weight": emphasis_weight(base_weight) }),
    ) else {
        return;
    };

    Theme::update(cx, |theme| {
        let highlight = Arc::make_mut(&mut theme.highlight_theme);
        highlight.style.syntax.title = Some(style);
        highlight.style.syntax.emphasis_strong = Some(style);
    });
}

fn emphasis_weight(base: f32) -> u16 {
    let stepped = ((base + EMPHASIS_STEP) / 100.).round() * 100.;
    stepped.clamp(MIN_FONT_WEIGHT, MAX_FONT_WEIGHT) as u16
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::DEFAULT_FONT_WEIGHT;
    use gpui_kit::{FontWeight, HighlightStyle};

    #[test]
    fn emphasis_is_two_steps_above_base() {
        assert_eq!(emphasis_weight(DEFAULT_FONT_WEIGHT), 600);
        assert_eq!(emphasis_weight(300.), 500);
        assert_eq!(emphasis_weight(100.), 300);
    }

    #[test]
    fn emphasis_is_clamped_to_theme_range() {
        assert_eq!(emphasis_weight(700.), 900);
        assert_eq!(emphasis_weight(900.), 900);
    }

    #[test]
    fn emphasis_style_is_heavier_and_uncolored() {
        let style: ThemeStyle =
            serde_json::from_value(serde_json::json!({ "font_weight": emphasis_weight(400.) }))
                .unwrap();
        let resolved: HighlightStyle = style.into();

        assert_eq!(resolved.font_weight, Some(FontWeight::SEMIBOLD));
        assert_eq!(resolved.color, None);
    }
}
