use reratui::prelude::*;

#[derive(Props)]
pub struct FormProps {
    pub form: FormHandle,
    pub children: Vec<Element>,
}

/// Field registry for tracking field order
#[derive(Clone)]
pub struct FieldRegistry {
    fields: StateHandle<Vec<String>>,
    set_fields: StateSetter<Vec<String>>,
    focused_index: StateHandle<usize>,
    set_focused_index: StateSetter<usize>,
}

impl FieldRegistry {
    pub fn new() -> Self {
        let (fields, set_fields) = use_state(Vec::new);
        let (focused_index, set_focused_index) = use_state(|| 0);

        Self {
            fields,
            set_fields,
            focused_index,
            set_focused_index,
        }
    }

    pub fn register_field(&self, field_name: &str) -> usize {
        let mut fields = self.fields.get().clone();
        if let Some(index) = fields.iter().position(|f| f == field_name) {
            index
        } else {
            fields.push(field_name.to_string());
            self.set_fields.set(fields.clone());
            fields.len() - 1
        }
    }

    pub fn get_focused_index(&self) -> usize {
        self.focused_index.get()
    }

    pub fn set_focused_index(&self, index: usize) {
        self.set_focused_index.set(index);
    }

    pub fn field_count(&self) -> usize {
        self.fields.get().len()
    }
}

#[component]
pub fn Form(props: &FormProps) -> Element {
    // Provide form handle to children via context
    use_context_provider(|| props.form.clone());

    // Create and provide field registry
    let registry = FieldRegistry::new();
    use_context_provider(|| registry.clone());

    // Keyboard navigation
    use_keyboard_shortcut(KeyCode::Tab, KeyModifiers::NONE, {
        let registry = registry.clone();
        move || {
            let current = registry.get_focused_index();
            let count = registry.field_count();
            if count > 0 {
                registry.set_focused_index((current + 1) % count);
            }
        }
    });

    use_keyboard_shortcut(KeyCode::BackTab, KeyModifiers::SHIFT, {
        let registry = registry.clone();
        move || {
            let current = registry.get_focused_index();
            let count = registry.field_count();
            if count > 0 {
                registry.set_focused_index(if current == 0 { count - 1 } else { current - 1 });
            }
        }
    });

    // Create constraints dynamically based on number of children
    // Each field needs: 1 line for label + 3 lines for input + 1 line for description (optional) + 1 line for message
    // For now, allocate 6 lines per field to accommodate optional description
    let constraints = props
        .children
        .iter()
        .map(|_| Constraint::Length(6))
        .collect::<Vec<_>>();

    rsx! {
        <Layout
            direction={Direction::Vertical}
            constraints={constraints}
        >
            {props.children.clone()}
        </Layout>
    }
}
