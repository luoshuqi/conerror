use proc_macro::TokenStream;

use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::visit_mut::{visit_expr_try_mut, VisitMut};
use syn::{parse_macro_input, parse_quote_spanned, ExprTry, ItemFn, ReturnType};

#[proc_macro_attribute]
pub fn conerror(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut func = parse_macro_input!(input as ItemFn);
    MapErr(&func.sig.ident.to_string()).visit_item_fn_mut(&mut func);
    let assert = return_type_assert(&func);
    quote!(#assert #func).into()
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

fn return_type_assert(func: &ItemFn) -> proc_macro2::TokenStream {
    let ty = match func.sig.output {
        ReturnType::Default => {
            return syn::Error::new(
                func.sig.paren_token.span.close(),
                "#[conerror]: expected return type",
            )
            .to_compile_error()
        }
        ReturnType::Type(_, ref ty) => &**ty,
    };
    let st = format_ident!("_conerror_assert_{}", func.sig.ident);
    quote_spanned! {ty.span()=>
        #[allow(non_camel_case_types)]
        struct #st where #ty: conerror::ConerrorResult;
    }
}
