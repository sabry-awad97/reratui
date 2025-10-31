//! Virtual DOM node types

use crate::component::Component;
use crate::layout::{AnyWidget, LayoutWrapper};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use std::{
    any::{Any, TypeId},
    rc::Rc,
};

impl Default for Element {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Component + 'static> From<T> for Element {
    fn from(component: T) -> Self {
        Element::component(component)
    }
}

/// Type alias for the render function
type RenderFn = Rc<dyn Fn(&dyn Any, Rect, &mut Buffer)>;

/// Represents a virtual node in the virtual DOM tree.
#[derive(Clone)]
pub enum Element {
    /// Represents a component in the virtual DOM tree.
    Component {
        /// The type ID of the component.
        type_id: TypeId,
        /// The props of the component.
        props: Rc<dyn Any>,
        /// The children of the component.
        children: Vec<Element>,
        /// The key of the component.
        key: Option<String>,
        /// The actual component instance.
        component: Rc<dyn Component>,
    },
    /// Represents a primitive widget in the virtual DOM tree.
    Widget {
        /// The widget instance.
        widget: Rc<dyn Any>,
        /// Render function that knows how to render this specific widget
        render_fn: RenderFn,
        /// The key of the widget.
        key: Option<String>,
    },
    /// Represents a text node in the virtual DOM tree.
    Text(String),
}

impl Element {
    /// Creates a new empty element (placeholder for compatibility)
    pub fn new() -> Self {
        Element::Text(String::new())
    }

    /// Creates a new component node.
    pub fn component<C: Component + 'static>(component: C) -> Self {
        Element::Component {
            type_id: TypeId::of::<C>(),
            props: Rc::new(()),
            children: Vec::new(),
            key: None,
            component: Rc::new(component),
        }
    }

    /// Creates a new widget node.
    pub fn widget<W: Widget + Clone + 'static>(widget: W) -> Self {
        let widget_box = Rc::new(widget.clone());
        let render_fn = Rc::new(move |any: &dyn Any, area: Rect, buffer: &mut Buffer| {
            if let Some(w) = any.downcast_ref::<W>() {
                w.clone().render(area, buffer);
            }
        });

        Element::Widget {
            widget: widget_box,
            render_fn,
            key: None,
        }
    }

    /// Creates a new text node.
    pub fn text<S: Into<String>>(text: S) -> Self {
        Element::Text(text.into())
    }

    /// Creates a fragment containing multiple elements.
    /// This creates a container that can hold and render multiple child elements.
    pub fn fragment(elements: Vec<Element>) -> Self {
        if elements.is_empty() {
            Element::text("")
        } else if elements.len() == 1 {
            elements.into_iter().next().unwrap()
        } else {
            // Create a fragment container that holds all elements

            // Convert all elements to AnyWidget
            let children: Vec<AnyWidget> = elements.into_iter().map(AnyWidget::VNode).collect();

            // Create a vertical layout with equal constraints for each child
            let constraints: Vec<Constraint> =
                (0..children.len()).map(|_| Constraint::Min(0)).collect();

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints);

            let layout_wrapper = LayoutWrapper::new(layout, children);

            Element::Widget {
                widget: Rc::new(layout_wrapper),
                render_fn: Rc::new(|widget, area, buffer| {
                    if let Some(layout_wrapper) = widget.downcast_ref::<LayoutWrapper>() {
                        layout_wrapper.clone().render(area, buffer);
                    }
                }),
                key: None,
            }
        }
    }

    /// Sets the key for this node.
    pub fn with_key<S: Into<String>>(mut self, key: S) -> Self {
        match &mut self {
            Element::Component { key: k, .. } => *k = Some(key.into()),
            Element::Widget { key: k, .. } => *k = Some(key.into()),
            Element::Text(_) => {} // Text nodes don't have keys
        }
        self
    }

    /// Renders this node to the buffer.
    pub fn render(&self, area: Rect, buffer: &mut Buffer) {
        match self {
            Element::Component { component, .. } => {
                // Render with lifecycle hooks (on_mount/on_unmount)
                crate::component::render_component_with_lifecycle(component, area, buffer);
            }
            Element::Widget {
                widget, render_fn, ..
            } => {
                render_fn(widget.as_ref(), area, buffer);
            }
            Element::Text(_) => {
                // Text nodes are usually rendered as part of a widget
            }
        }
    }
}

/// Represents a property value in the virtual DOM tree.
#[derive(Clone)]
pub enum PropValue {
    /// Represents a string property value.
    String(String),
    /// Represents an integer property value.
    Int(i64),
    /// Represents a number property value.
    Number(f64),
    /// Represents a boolean property value.
    Bool(bool),
    /// Represents an object property value.
    Object(Rc<dyn Any>),
}
