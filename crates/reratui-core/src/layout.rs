//! Layout wrapper components for RSX macro support
//!
//! This module provides wrapper components that enable ratatui's Layout and Block
//! to work with nested children in the RSX macro system.

use crate::vnode::Element;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, List, Paragraph, Widget},
};

/// An enum that can hold different types of widgets
#[derive(Clone)]
pub enum AnyWidget {
    /// A layout wrapper widget
    Layout(LayoutWrapper),
    /// A block wrapper widget
    Block(BlockWrapper),
    /// A paragraph widget
    Paragraph(Paragraph<'static>),
    /// A list widget
    List(List<'static>),
    /// A VNode widget
    VNode(Element),
}

impl Widget for AnyWidget {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        match self {
            AnyWidget::Layout(layout) => layout.render(area, buffer),
            AnyWidget::Block(block) => block.render(area, buffer),
            AnyWidget::Paragraph(paragraph) => paragraph.render(area, buffer),
            AnyWidget::List(list) => list.render(area, buffer),
            AnyWidget::VNode(vnode) => vnode.render(area, buffer),
        }
    }
}

impl From<LayoutWrapper> for AnyWidget {
    fn from(layout: LayoutWrapper) -> Self {
        AnyWidget::Layout(layout)
    }
}

impl From<BlockWrapper> for AnyWidget {
    fn from(block: BlockWrapper) -> Self {
        AnyWidget::Block(block)
    }
}

impl From<Paragraph<'static>> for AnyWidget {
    fn from(paragraph: Paragraph<'static>) -> Self {
        AnyWidget::Paragraph(paragraph)
    }
}

impl From<List<'static>> for AnyWidget {
    fn from(list: List<'static>) -> Self {
        AnyWidget::List(list)
    }
}

impl From<Element> for AnyWidget {
    fn from(vnode: Element) -> Self {
        AnyWidget::VNode(vnode)
    }
}

impl From<Block<'static>> for AnyWidget {
    fn from(block: Block<'static>) -> Self {
        AnyWidget::Block(BlockWrapper::new(block, vec![]))
    }
}

impl From<String> for AnyWidget {
    fn from(text: String) -> Self {
        AnyWidget::VNode(Element::text(text))
    }
}

impl From<&str> for AnyWidget {
    fn from(text: &str) -> Self {
        AnyWidget::VNode(Element::text(text.to_string()))
    }
}

impl From<&String> for AnyWidget {
    fn from(text: &String) -> Self {
        AnyWidget::VNode(Element::text(text.clone()))
    }
}

impl From<ratatui::text::Span<'static>> for AnyWidget {
    fn from(span: ratatui::text::Span<'static>) -> Self {
        // Wrap the span in a paragraph to make it renderable
        use ratatui::text::Line;
        let line = Line::from(vec![span]);
        let paragraph = Paragraph::new(vec![line]);
        AnyWidget::Paragraph(paragraph)
    }
}

impl From<ratatui::text::Line<'static>> for AnyWidget {
    fn from(line: ratatui::text::Line<'static>) -> Self {
        let paragraph = Paragraph::new(vec![line]);
        AnyWidget::Paragraph(paragraph)
    }
}

/// A wrapper around ratatui's Layout that can render children in split areas
#[derive(Clone)]
pub struct LayoutWrapper {
    layout: Layout,
    children: Vec<AnyWidget>,
    constraints: Option<Vec<Constraint>>,
}

impl LayoutWrapper {
    /// Creates a new LayoutWrapper
    pub fn new(layout: Layout, children: Vec<AnyWidget>) -> Self {
        Self {
            layout,
            children,
            constraints: None,
        }
    }

    /// Creates a new LayoutWrapper with custom constraints
    pub fn with_constraints(
        layout: Layout,
        children: Vec<AnyWidget>,
        constraints: Vec<Constraint>,
    ) -> Self {
        Self {
            layout,
            children,
            constraints: Some(constraints),
        }
    }

    /// Creates a new LayoutWrapper from Elements
    pub fn from_elements(layout: Layout, children: Vec<Element>) -> Self {
        Self {
            layout,
            children: children.into_iter().map(AnyWidget::from).collect(),
            constraints: None,
        }
    }

    /// Creates a new LayoutWrapper from Elements with custom constraints
    pub fn from_elements_with_constraints(
        layout: Layout,
        children: Vec<Element>,
        constraints: Vec<Constraint>,
    ) -> Self {
        Self {
            layout,
            children: children.into_iter().map(AnyWidget::from).collect(),
            constraints: Some(constraints),
        }
    }
}

impl Widget for LayoutWrapper {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        // Use custom constraints if provided, otherwise create default constraints
        let constraints = if let Some(custom_constraints) = self.constraints {
            custom_constraints
        } else {
            // Create better default constraints based on the number of children
            match self.children.len() {
                0 => vec![],
                1 => vec![Constraint::Percentage(100)],
                2 => vec![Constraint::Percentage(50), Constraint::Percentage(50)],
                3 => vec![
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                    Constraint::Percentage(34),
                ],
                4 => vec![Constraint::Percentage(25); 4],
                5 => vec![Constraint::Percentage(20); 5],
                _ => {
                    // For more than 5 children, use equal distribution
                    let percentage = 100 / self.children.len() as u16;
                    let remainder = 100 % self.children.len() as u16;
                    let mut constraints =
                        vec![Constraint::Percentage(percentage); self.children.len()];
                    if remainder > 0 && !constraints.is_empty() {
                        // Add the remainder to the last constraint
                        if let Some(last) = constraints.last_mut() {
                            *last = Constraint::Percentage(percentage + remainder);
                        }
                    }
                    constraints
                }
            }
        };

        // Create a new layout with the constraints
        let layout = self.layout.constraints(constraints);
        let chunks = layout.split(area);

        // Render each child in its corresponding chunk
        for (i, child) in self.children.into_iter().enumerate() {
            if i < chunks.len() {
                child.render(chunks[i], buffer);
            }
        }
    }
}

/// A wrapper around ratatui's Block that can render children inside the block
#[derive(Clone)]
pub struct BlockWrapper {
    block: Block<'static>,
    children: Vec<AnyWidget>,
}

impl BlockWrapper {
    /// Creates a new BlockWrapper
    pub fn new(block: Block<'static>, children: Vec<AnyWidget>) -> Self {
        Self { block, children }
    }
}

impl Widget for BlockWrapper {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        // Calculate the inner area before rendering the block
        let inner_area = self.block.inner(area);

        // Render the block border
        self.block.render(area, buffer);

        // Render children in the inner area
        if !self.children.is_empty() {
            if self.children.len() == 1 {
                // Single child - render directly in the inner area
                self.children
                    .into_iter()
                    .next()
                    .unwrap()
                    .render(inner_area, buffer);
            } else {
                // Multiple children - create a vertical layout
                let constraints: Vec<Constraint> = (0..self.children.len())
                    .map(|_| Constraint::Min(0))
                    .collect();

                let layout = Layout::default()
                    .direction(ratatui::layout::Direction::Vertical)
                    .constraints(constraints);

                let chunks = layout.split(inner_area);

                for (i, child) in self.children.into_iter().enumerate() {
                    if i < chunks.len() {
                        child.render(chunks[i], buffer);
                    }
                }
            }
        }
    }
}
