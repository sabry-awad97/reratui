use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Type, parse_macro_input};

/// Implementation of the Props derive macro
pub fn derive_props_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Extract fields from the struct
    let fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => &fields_named.named,
            _ => {
                return syn::Error::new_spanned(
                    &input,
                    "Props can only be derived for structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(&input, "Props can only be derived for structs")
                .to_compile_error()
                .into();
        }
    };

    // Check if there's a children field
    let has_children_field = fields
        .iter()
        .any(|field| field.ident.as_ref().map(|n| n.to_string()) == Some("children".to_string()));

    // Generate builder methods for each field
    let builder_methods = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;

        // Check if this is an Option<Callback<...>> type for special handling
        if is_option_callback_type(field_type) {
            quote! {
                pub fn #field_name<T>(mut self, value: T) -> Self
                where
                    T: reratui::hooks::callback::IntoCallbackProp<#field_type>,
                {
                    self.#field_name = value.into_callback_prop();
                    self
                }
            }
        } else {
            quote! {
                pub fn #field_name<T: Into<#field_type>>(mut self, value: T) -> Self {
                    self.#field_name = value.into();
                    self
                }
            }
        }
    });

    // Always generate with_children method
    let with_children_method = if has_children_field {
        quote! {
            pub fn with_children(mut self, children: Vec<Element>) -> Self {
                self.children = children;
                self
            }
        }
    } else {
        quote! {
            pub fn with_children(self, _children: Vec<Element>) -> Self {
                // This props struct doesn't have a children field, so just return self
                self
            }
        }
    };

    // Generate default implementation
    let default_fields = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;

        // Check if this is the children field
        let is_children =
            field_name.as_ref().map(|n| n.to_string()) == Some("children".to_string());

        if is_children {
            quote! {
                #field_name: Vec::new()
            }
        } else {
            // Check if the type is Option
            if is_option_type(field_type) {
                quote! {
                    #field_name: None
                }
            } else {
                quote! {
                    #field_name: Default::default()
                }
            }
        }
    });

    // Generate ComponentProps implementation based on whether there's a children field
    let component_props_impl = if has_children_field {
        quote! {
            impl #impl_generics ComponentProps for #name #ty_generics #where_clause {
                fn get_children(&self) -> Vec<Element> {
                    self.children.clone()
                }

                fn set_children(&mut self, children: Vec<Element>) {
                    self.children = children;
                }
            }
        }
    } else {
        quote! {
            impl #impl_generics ComponentProps for #name #ty_generics #where_clause {
                fn get_children(&self) -> Vec<Element> {
                    // Return empty children for props without children field
                    Vec::new()
                }

                fn set_children(&mut self, _children: Vec<Element>) {
                    // This props struct doesn't have a children field, so ignore
                }
            }
        }
    };

    // Generate Clone implementation
    let clone_fields = fields.iter().map(|field| {
        let field_name = &field.ident;
        quote! {
            #field_name: self.#field_name.clone()
        }
    });

    let expanded = quote! {
        // Automatically derive Clone for the props struct
        impl #impl_generics Clone for #name #ty_generics #where_clause {
            fn clone(&self) -> Self {
                Self {
                    #(#clone_fields,)*
                }
            }
        }

        impl #impl_generics Default for #name #ty_generics #where_clause {
            fn default() -> Self {
                Self {
                    #(#default_fields,)*
                }
            }
        }

        impl #impl_generics #name #ty_generics #where_clause {
            #(#builder_methods)*

            #with_children_method
        }

        #component_props_impl
    };

    expanded.into()
}

/// Helper function to check if a type is Option<T>
fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident == "Option";
    }
    false
}

/// Helper function to check if a type is Option<Callback<...>>
fn is_option_callback_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
        && segment.ident == "Option"
    {
        // Check if the inner type is Callback
        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments
            && let Some(syn::GenericArgument::Type(Type::Path(inner_path))) = args.args.first()
            && let Some(inner_segment) = inner_path.path.segments.last()
        {
            return inner_segment.ident == "Callback";
        }
    }
    false
}
