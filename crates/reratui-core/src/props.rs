use crate::Element;

/// Trait for component props that support children
pub trait ComponentProps {
    /// Get the children VNodes (returns a clone for simplicity)
    fn get_children(&self) -> Vec<Element>;

    /// Set the children VNodes
    fn set_children(&mut self, children: Vec<Element>);
}
