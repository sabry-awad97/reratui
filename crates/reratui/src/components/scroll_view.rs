//! ScrollView component for scrollable content.
//!
//! Provides a container that can scroll content taller than the viewport,
//! similar to Ink's scrolling behavior but within Ratatui's fullscreen model.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget},
};

use crate::Component;
use crate::hooks::{use_keyboard, use_state};
use crossterm::event::KeyCode;

/// Configuration for scroll indicators
#[derive(Clone, Debug)]
pub struct ScrollIndicator {
    /// Show scrollbar
    pub show_scrollbar: bool,
    /// Show "more above" indicator
    pub show_more_above: bool,
    /// Show "more below" indicator  
    pub show_more_below: bool,
    /// Scrollbar track color
    pub track_color: Color,
    /// Scrollbar thumb color
    pub thumb_color: Color,
}

impl Default for ScrollIndicator {
    fn default() -> Self {
        Self {
            show_scrollbar: true,
            show_more_above: true,
            show_more_below: true,
            track_color: Color::DarkGray,
            thumb_color: Color::Gray,
        }
    }
}

/// Props for ScrollView component
pub struct ScrollViewProps<F>
where
    F: Fn(Rect, &mut Buffer, usize, usize),
{
    /// Total number of items/lines in the content
    pub content_height: usize,
    /// Render function that receives (area, buffer, start_index, visible_count)
    pub render_content: F,
    /// Optional block wrapper
    pub block: Option<Block<'static>>,
    /// Scroll indicator configuration
    pub indicators: ScrollIndicator,
    /// Enable keyboard navigation (j/k, arrows, etc.)
    pub keyboard_nav: bool,
    /// Lines to scroll per key press
    pub scroll_step: usize,
}

impl<F> ScrollViewProps<F>
where
    F: Fn(Rect, &mut Buffer, usize, usize),
{
    /// Create new ScrollViewProps with a render function
    pub fn new(content_height: usize, render_content: F) -> Self {
        Self {
            content_height,
            render_content,
            block: None,
            indicators: ScrollIndicator::default(),
            keyboard_nav: true,
            scroll_step: 1,
        }
    }

    /// Add a block wrapper
    pub fn block(mut self, block: Block<'static>) -> Self {
        self.block = Some(block);
        self
    }

    /// Configure scroll indicators
    pub fn indicators(mut self, indicators: ScrollIndicator) -> Self {
        self.indicators = indicators;
        self
    }

    /// Enable/disable keyboard navigation
    pub fn keyboard_nav(mut self, enabled: bool) -> Self {
        self.keyboard_nav = enabled;
        self
    }

    /// Set scroll step (lines per key press)
    pub fn scroll_step(mut self, step: usize) -> Self {
        self.scroll_step = step;
        self
    }
}

/// A scrollable container component.
///
/// ScrollView wraps content and provides scrolling when the content
/// exceeds the viewport height. It supports:
///
/// - Keyboard navigation (j/k, arrows, page up/down, home/end)
/// - Visual scrollbar
/// - "More above/below" indicators
/// - Customizable styling
///
/// # Example
///
/// ```rust,ignore
/// use reratui::prelude::*;
/// use reratui::components::{ScrollView, ScrollViewProps};
///
/// struct MyList {
///     items: Vec<String>,
/// }
///
/// impl Component for MyList {
///     fn render(&self, area: Rect, buffer: &mut Buffer) {
///         let props = ScrollViewProps::new(
///             self.items.len(),
///             |area, buf, start, count| {
///                 // Render visible items
///                 for (i, item) in self.items.iter()
///                     .skip(start)
///                     .take(count)
///                     .enumerate()
///                 {
///                     let y = area.y + i as u16;
///                     buf.set_string(area.x, y, item, Style::default());
///                 }
///             },
///         )
///         .block(Block::default().borders(Borders::ALL).title("Items"));
///
///         ScrollView::new(props).render(area, buffer);
///     }
/// }
/// ```
pub struct ScrollView<F>
where
    F: Fn(Rect, &mut Buffer, usize, usize),
{
    props: ScrollViewProps<F>,
}

impl<F> ScrollView<F>
where
    F: Fn(Rect, &mut Buffer, usize, usize),
{
    /// Create a new ScrollView with the given props
    pub fn new(props: ScrollViewProps<F>) -> Self {
        Self { props }
    }
}

impl<F> Component for ScrollView<F>
where
    F: Fn(Rect, &mut Buffer, usize, usize) + 'static,
{
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        // Calculate inner area (accounting for block borders)
        let inner_area = if let Some(ref block) = self.props.block {
            block.clone().render(area, buffer);
            block.inner(area)
        } else {
            area
        };

        // Reserve space for scrollbar if enabled
        let content_area = if self.props.indicators.show_scrollbar {
            Rect {
                width: inner_area.width.saturating_sub(1),
                ..inner_area
            }
        } else {
            inner_area
        };

        let viewport_height = content_area.height as usize;
        let content_height = self.props.content_height;

        // Scroll state
        let (offset, set_offset) = use_state(|| 0usize);

        // Calculate max offset
        let max_offset = content_height.saturating_sub(viewport_height);

        // Clamp offset if content shrinks
        let clamped_offset = offset.min(max_offset);
        if clamped_offset != offset {
            set_offset.set(clamped_offset);
        }

        // Keyboard navigation
        if self.props.keyboard_nav {
            let step = self.props.scroll_step;

            use_keyboard(move |key| {
                let current = offset;
                let new_offset = match key.code {
                    KeyCode::Char('j') | KeyCode::Down => (current + step).min(max_offset),
                    KeyCode::Char('k') | KeyCode::Up => current.saturating_sub(step),
                    KeyCode::Char('g') | KeyCode::Home => 0,
                    KeyCode::Char('G') | KeyCode::End => max_offset,
                    KeyCode::PageDown => {
                        (current + viewport_height.saturating_sub(1)).min(max_offset)
                    }
                    KeyCode::PageUp => current.saturating_sub(viewport_height.saturating_sub(1)),
                    KeyCode::Char('d')
                        if key
                            .modifiers
                            .contains(crossterm::event::KeyModifiers::CONTROL) =>
                    {
                        (current + viewport_height / 2).min(max_offset)
                    }
                    KeyCode::Char('u')
                        if key
                            .modifiers
                            .contains(crossterm::event::KeyModifiers::CONTROL) =>
                    {
                        current.saturating_sub(viewport_height / 2)
                    }
                    _ => return,
                };
                set_offset.set(new_offset);
            });
        }

        // Render content
        let visible_count = viewport_height.min(content_height.saturating_sub(clamped_offset));
        (self.props.render_content)(content_area, buffer, clamped_offset, visible_count);

        // Render scrollbar
        if self.props.indicators.show_scrollbar && content_height > viewport_height {
            let scrollbar_area = Rect {
                x: inner_area.x + inner_area.width.saturating_sub(1),
                y: inner_area.y,
                width: 1,
                height: inner_area.height,
            };

            let mut scrollbar_state = ScrollbarState::new(content_height)
                .position(clamped_offset)
                .viewport_content_length(viewport_height);

            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .track_style(Style::default().fg(self.props.indicators.track_color))
                .thumb_style(Style::default().fg(self.props.indicators.thumb_color));

            StatefulWidget::render(scrollbar, scrollbar_area, buffer, &mut scrollbar_state);
        }

        // Render "more above" indicator
        if self.props.indicators.show_more_above && clamped_offset > 0 {
            let indicator = "▲ more";
            let x = content_area.x
                + content_area
                    .width
                    .saturating_sub(indicator.len() as u16 + 1);
            buffer.set_string(
                x,
                content_area.y,
                indicator,
                Style::default().fg(Color::DarkGray),
            );
        }

        // Render "more below" indicator
        if self.props.indicators.show_more_below && clamped_offset < max_offset {
            let indicator = "▼ more";
            let y = content_area.y + content_area.height.saturating_sub(1);
            let x = content_area.x
                + content_area
                    .width
                    .saturating_sub(indicator.len() as u16 + 1);
            buffer.set_string(x, y, indicator, Style::default().fg(Color::DarkGray));
        }
    }
}
