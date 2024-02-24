use proc_macro::TokenStream;

use quote::quote;
use syn::spanned::Spanned;
use syn::visit_mut::{visit_expr_try_mut, visit_generic_argument_mut, VisitMut};
use syn::{
    parse_macro_input, parse_quote, parse_quote_spanned, parse_str, ExprTry, GenericArgument,
    ItemFn, ReturnType, Stmt, Type,
};

#[proc_macro_attribute]
pub fn conerror(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut func = parse_macro_input!(input as ItemFn);
    MapErr(&func.sig.ident.to_string()).visit_item_fn_mut(&mut func);
    func.block.stmts.insert(0, return_type_assert(&func));
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

fn return_type_assert(func: &ItemFn) -> Stmt {
    match func.sig.output {
        ReturnType::Type(_, ref ty) => {
            let mut ty = ty.clone();
            SubstitueImplTrait.visit_type_mut(&mut ty);
            parse_quote_spanned! {ty.span()=>
                { <#ty as conerror::ConerrorResult>::ASSERT; }
            }
        }
        ReturnType::Default => {
            let e = syn::Error::new(
                func.sig.paren_token.span.close(),
                "conerror: expected return type",
            )
            .to_compile_error();
            parse_quote!(#e)
        }
    }
}

struct SubstitueImplTrait;

impl VisitMut for SubstitueImplTrait {
    fn visit_generic_argument_mut(&mut self, i: &mut GenericArgument) {
        match i {
            GenericArgument::Type(Type::ImplTrait(_)) => {
                *i = parse_str("conerror::SubstitutedImplTrait").unwrap()
            }
            _ => (),
        }
        visit_generic_argument_mut(self, i)
    }
}
