use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{FnArg, Ident, ItemFn, Pat, Token, Type, parse_macro_input};

fn find_field_id_param(inputs: &Punctuated<FnArg, syn::token::Comma>) -> Option<Ident> {
    inputs.iter().find_map(|arg| {
        if let FnArg::Typed(pat_type) = arg {
            if let Type::Path(type_path) = pat_type.ty.as_ref() {
                let is_field_id = type_path
                    .path
                    .segments
                    .last()
                    .map(|s| s.ident == "FieldId")
                    .unwrap_or(false);
                if is_field_id {
                    if let Pat::Ident(pat_ident) = pat_type.pat.as_ref() {
                        return Some(pat_ident.ident.clone());
                    }
                }
            }
        }
        None
    })
}

/// checks the permissions from the state, first param is the enum type
/// followed by the specific permission variants
///
/// Usage:
/// #[requires(TablePermission, Edit, View)]
/// async fn my_function(&self, ..) -> Result<(), Irror> { ... }
///
/// For field-level permissions, use `FieldPermission` + a `FieldId` param:
/// #[requires(FieldPermission, Edit)]
/// async fn update_field(&mut self, field_id: FieldId, ...) -> ... { ... }
#[proc_macro_attribute]
pub fn requires(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let parser = Punctuated::<Ident, Token![,]>::parse_terminated;
    let idents = match parser.parse(attr) {
        Ok(idents) => idents,
        Err(err) => return err.to_compile_error().into(),
    };
    let mut iter = idents.into_iter();
    let perm_type = match iter.next() {
        Some(id) => id,
        _ => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                "[<PERMISSION>, <edit, view, etc...>, ...]",
            )
            .to_compile_error()
            .into();
        }
    };

    let variants: Vec<Ident> = iter.collect();
    if variants.is_empty() {
        return syn::Error::new(
            perm_type.span(),
            "[<PERMISSION>, <edit, view, etc...>, ...]",
        )
        .to_compile_error()
        .into();
    }

    let is_field_perm = perm_type == Ident::new("FieldPermission", Span::call_site());
    let field_id_ident = if is_field_perm {
        find_field_id_param(&input.sig.inputs)
    } else {
        None
    };

    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = input;

    let stmts = &block.stmts;

    let expanded = if is_field_perm {
        if let Some(field_id) = field_id_ident {
            quote! {
                #(#attrs)*
                #vis #sig {
                    let state = self.load_state().await?;
                    let field_perms = self.load_field_perms().await?;
                    let fp = field_perms.get(&#field_id).copied().unwrap_or(FieldPermissions::from(0));

                    let is_authorized = state.is_owner #( || fp.can(FieldPermission::#variants) )*;

                    if !is_authorized {
                        return Err(Irror::Table(TableError::Unauthorized));
                    }

                    #(#stmts)*
                }
            }
        } else {
            // Collect all field IDs with the first permission variant
            let perm = &variants[0];
            quote! {
                #(#attrs)*
                #vis #sig {
                    let state = self.load_state().await?;
                    let field_perms = self.load_field_perms().await?;
                    let __visible_fields: Vec<FieldId> = field_perms.into_iter()
                        .filter(|(_, fp)| fp.can(FieldPermission::#perm))
                        .map(|(fid, _)| fid)
                        .collect();

                    let is_authorized = state.is_owner || !__visible_fields.is_empty();

                    if !is_authorized {
                        return Err(Irror::Table(TableError::Unauthorized));
                    }

                    #(#stmts)*
                }
            }
        }
    } else {
        quote! {
            #(#attrs)*
            #vis #sig {
                let state = self.load_state().await?;

                let is_authorized = state.is_owner #( || state.permissions.can(#perm_type::#variants) )*;

                if !is_authorized {
                    return Err(Irror::Table(TableError::Unauthorized));
                }

                #(#stmts)*
            }
        }
    };

    TokenStream::from(expanded)
}
