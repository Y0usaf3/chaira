use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{Ident, ItemFn, Token, parse_macro_input};

/// checks the permissions from the state, first param is the enum type
/// followed by the specific permission variants
///
/// Usage:
/// #[requires(BasePermission, Edit, View)]
/// async fn my_function(&self, ..) -> Result<(), Irror> { ... }
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

    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = input;

    let stmts = &block.stmts;

    let expanded = quote! {
        #(#attrs)*
        #vis #sig {
            let state = self.load_state().await?;

            let is_authorized = state.is_owner #( || state.permissions.can(#perm_type::#variants) )*;

            if !is_authorized {
                return Err(Irror::Table(TableError::Unauthorized));
            }

            #(#stmts)*
        }
    };

    TokenStream::from(expanded)
}
