use proc_macro::TokenStream;

use quote::quote;
use syn::spanned::Spanned;
use syn::visit_mut::{visit_expr_try_mut, VisitMut};
use syn::{parse_macro_input, parse_quote_spanned, ExprTry, ItemFn};

#[proc_macro_attribute]
pub fn conerror(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut func = parse_macro_input!(input as ItemFn);
    MapErr(&func.sig.ident.to_string()).visit_item_fn_mut(&mut func);
    quote!(#func).into()
}

struct MapErr<'a>(&'a str);

impl<'a> VisitMut for MapErr<'a> {
    fn visit_expr_try_mut(&mut self, i: &mut ExprTry) {
        let expr = i.expr.clone();
        let func = self.0;
        *i.expr = parse_quote_spanned! {expr.span() =>
            #expr.map_err(|err| conerror::Error::chain(err, file!(), line!(), #func))
        };
        visit_expr_try_mut(self, i);
    }
}
