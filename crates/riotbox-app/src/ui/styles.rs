use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;

pub(in crate::ui) fn style_primary_control() -> Style {
    Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD)
}

pub(in crate::ui) fn style_pending_cue() -> Style {
    Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD)
}

pub(in crate::ui) fn style_pending_detail() -> Style {
    Style::default().fg(Color::Yellow)
}

pub(in crate::ui) fn style_confirmation() -> Style {
    Style::default().fg(Color::Green)
}

pub(in crate::ui) fn style_confirmation_strong() -> Style {
    style_confirmation().add_modifier(Modifier::BOLD)
}

pub(in crate::ui) fn style_warning_label() -> Style {
    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
}

pub(in crate::ui) fn style_warning_detail() -> Style {
    Style::default().fg(Color::Yellow)
}

pub(in crate::ui) fn style_low_emphasis() -> Style {
    Style::default().fg(Color::DarkGray)
}

pub(in crate::ui) fn line_with_primary_keys(text: impl Into<String>) -> Line<'static> {
    let text = text.into();
    let mut spans = Vec::new();
    let mut rest = text.as_str();

    while let Some(start) = rest.find('[') {
        let (prefix, key_and_tail) = rest.split_at(start);
        if !prefix.is_empty() {
            spans.push(Span::raw(prefix.to_owned()));
        }

        let Some(end) = key_and_tail.find(']') else {
            spans.push(Span::raw(key_and_tail.to_owned()));
            return Line::from(spans);
        };
        let key_end = end + 1;
        let (key, tail) = key_and_tail.split_at(key_end);
        spans.push(Span::styled(key.to_owned(), style_primary_control()));
        rest = tail;
    }

    if !rest.is_empty() || spans.is_empty() {
        spans.push(Span::raw(rest.to_owned()));
    }

    Line::from(spans)
}

pub(in crate::ui) fn line_with_primary_key_prefixes(text: impl Into<String>) -> Line<'static> {
    let text = text.into();
    let mut spans = Vec::new();

    for (index, segment) in text.split(" | ").enumerate() {
        if index > 0 {
            spans.push(Span::raw(" | "));
        }

        let Some(colon) = segment.find(':') else {
            spans.push(Span::raw(segment.to_owned()));
            continue;
        };

        let (key, detail) = segment.split_at(colon);
        spans.push(Span::styled(key.to_owned(), style_primary_control()));
        spans.push(Span::raw(detail.to_owned()));
    }

    Line::from(spans)
}
