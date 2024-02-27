use proc_macro::TokenStream;

use quote::quote;
use syn::spanned::Spanned;
use syn::visit_mut::{visit_generic_argument_mut, visit_impl_item_fn_mut, VisitMut};
use syn::{
    parse, parse_quote, parse_quote_spanned, parse_str, ExprTry, GenericArgument, ImplItemFn,
    ItemFn, ItemImpl, ReturnType, Signature, Stmt, Type,
};

#[proc_macro_attribute]
pub fn conerror(_: TokenStream, input: TokenStream) -> TokenStream {
    match parse::<ItemFn>(input.clone()) {
        Ok(mut f) => {
            MapErr::new(None, Some(f.sig.ident.to_string())).visit_item_fn_mut(&mut f);
            f.block.stmts.insert(0, return_type_assert(&f.sig));
            quote!(#f).into()
        }
        Err(e) => match parse::<ItemImpl>(input) {
            Ok(mut item) => {
                MapErr::new(Some(item.self_ty.clone()), None).visit_item_impl_mut(&mut item);
                quote!(#item).into()
            }
            Err(mut err) => {
                err.combine(e);
                err.to_compile_error().into()
            }
        },
    }
}

struct MapErr {
    self_ty: Option<Box<Type>>,
    ident: Option<String>,
}

impl MapErr {
    fn new(self_ty: Option<Box<Type>>, ident: Option<String>) -> Self {
        Self { self_ty, ident }
    }
}

impl VisitMut for MapErr {
    fn visit_expr_try_mut(&mut self, i: &mut ExprTry) {
        let ident = self.ident.as_ref().unwrap();
        let module = match self.self_ty {
            Some(ref v) => quote!(std::any::type_name::<#v>()),
            None => quote!(module_path!()),
        };
        let expr = &i.expr;
        *i.expr = parse_quote_spanned! {expr.span() =>
            #expr.map_err(|err| conerror::Error::chain(err, file!(), line!(), #ident, #module))
        };
    }

    fn visit_impl_item_fn_mut(&mut self, i: &mut ImplItemFn) {
        let mut indices = vec![];
        for (i, attr) in i.attrs.iter().enumerate() {
            if attr.path().is_ident("conerror") {
                indices.push(i);
            }
        }
        if indices.is_empty() {
            return;
        }

        for idx in indices {
            i.attrs.remove(idx);
        }
        self.ident = Some(i.sig.ident.to_string());
        visit_impl_item_fn_mut(self, i);
        i.block.stmts.insert(0, return_type_assert(&i.sig));
    }
}

fn return_type_assert(sig: &Signature) -> Stmt {
    match sig.output {
        ReturnType::Type(_, ref ty) => {
            let mut ty = ty.clone();
            SubstitueImplTrait.visit_type_mut(&mut ty);
            parse_quote_spanned! {ty.span()=>
                { let _ = <#ty as conerror::ConerrorResult>::ASSERT; }
            }
        }
        ReturnType::Default => {
            let e = syn::Error::new(
                sig.paren_token.span.close(),
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
