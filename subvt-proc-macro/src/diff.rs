//! A derive macro that generates a diff struct for a struct, which contains `Option`s
//! for each field of the marked struct. Diff struct's name is the original struct's name
//! suffixed with `Diff`.
//!
//! A `diff`d struct can use the `get_diff` function on itself to calculate
//! the diff between itself and another instance, and the `apply_diff` function to
//! apply a diff struct to itself.
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Fields};

/// See module documentation for details.
pub fn derive_diff(input: DeriveInput) -> TokenStream {
    const KEY_ATTR_NAME: &str = "diff_key";

    let fields = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => fields.named,
        _ => panic!("This derive macro only works with named fields."),
    };
    let get_diff_method_statements = fields.iter().map(|field| {
        let field_ident = &field.ident;
        let field_is_key = field
            .attrs
            .iter()
            .any(|attribute| attribute.path.is_ident(KEY_ATTR_NAME));
        if field_is_key {
            quote! {
                diff.#field_ident = other.#field_ident.clone();
            }
        } else {
            quote! {
                if self.#field_ident != other.#field_ident {
                    diff.#field_ident = Some(other.#field_ident.clone());
                }
            }
        }
    });
    let apply_diff_method_statements = fields.iter().map(|field| {
        let field_ident = &field.ident;
        let field_is_key = field
            .attrs
            .iter()
            .any(|attribute| attribute.path.is_ident(KEY_ATTR_NAME));
        if field_is_key {
            quote! {
                self.#field_ident = diff.#field_ident.clone();
            }
        } else {
            quote! {
                if let Some(#field_ident) = &diff.#field_ident {
                    self.#field_ident = #field_ident.clone();
                }
            }
        }
    });
    let diff_fields = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_ty = &field.ty;
        let field_is_key = field
            .attrs
            .iter()
            .any(|attribute| attribute.path.is_ident(KEY_ATTR_NAME));
        if field_is_key {
            quote! {
                pub #field_name: #field_ty,
            }
        } else {
            quote! {
                #[serde(skip_serializing_if = "Option::is_none")]
                pub #field_name: ::std::option::Option<#field_ty>,
            }
        }
    });
    let ident = input.ident;
    let diff_ident = syn::Ident::new(&format!("{}Diff", ident), ident.span());
    let diff_struct = quote! {
        #[automatically_derived]
        #[derive(Clone, Debug, Default, Serialize)]
        pub struct #diff_ident {
            #(#diff_fields)*
        }
    };
    let get_diff_impl = quote! {
        #[automatically_derived]
        impl #ident {
            pub fn get_diff(&self, other:&#ident) -> #diff_ident {
                let mut diff = #diff_ident::default();
                #(#get_diff_method_statements)*
                diff
            }
        }
    };
    let apply_diff_impl = quote! {
        #[automatically_derived]
        impl #ident {
            pub fn apply_diff(&mut self, diff: &#diff_ident) {
                #(#apply_diff_method_statements)*
            }
        }
    };
    quote! {
        #diff_struct
        #get_diff_impl
        #apply_diff_impl
    }
}
