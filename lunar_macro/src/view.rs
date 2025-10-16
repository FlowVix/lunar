use either::Either::{self, Left, Right};
use heck::ToUpperCamelCase;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    AngleBracketedGenericArguments, Expr, Ident, Pat, Token, Type, braced, bracketed,
    parenthesized, parse::Parse, parse_quote, punctuated::Punctuated, token,
};

pub struct ViewBody {
    pub views: Vec<ViewType>,
}

pub struct IfView {
    cond: Expr,
    body: ViewBody,
    else_expr: Option<Either<Box<IfView>, ViewBody>>,
}

#[allow(clippy::large_enum_variant)]
pub enum ViewType {
    Element {
        name: Ident,
        modifiers: Option<Punctuated<ElemModifier, Token![,]>>,
        children: Option<ViewBody>,
    },
    Component {
        name: Ident,
        args: Punctuated<Expr, Token![,]>,
        children: Option<ViewBody>,
    },
    Expr(Expr),
    For {
        pattern: Pat,
        iter: Expr,
        key: Expr,
        body: ViewBody,
    },
    If(IfView),
    Dyn(ViewBody),
    State {
        name: Ident,
        typ: Type,
        init: Expr,
        body: ViewBody,
    },
}

pub struct Event {
    typ: Ident,
    arg: Option<Ident>,
}

pub enum ElemModifier {
    Attr(Ident, Expr, bool),
    Event(Event, Expr),
    ThemeOverride {
        typ: Ident,
        name: Ident,
        value: Expr,
    },
}

impl Parse for Event {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let typ = input.parse()?;
        let arg = if input.peek(Token![:]) {
            input.parse::<Token![:]>()?;
            Some(input.parse()?)
        } else {
            None
        };
        Ok(Self { typ, arg })
    }
}

impl Parse for ElemModifier {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![@]) {
            input.parse::<Token![@]>()?;
            let typ = input.parse()?;
            input.parse::<Token![=]>()?;
            let value = input.parse()?;
            Ok(ElemModifier::Event(typ, value))
        } else if input.peek(Token![#]) {
            input.parse::<Token![#]>()?;
            let typ = input.parse()?;
            input.parse::<Token![:]>()?;
            let name = input.parse()?;
            input.parse::<Token![=]>()?;
            let value = input.parse()?;
            Ok(ElemModifier::ThemeOverride { typ, name, value })
        } else {
            let name = input.parse()?;
            let build_only = input.peek(Token![@]);
            if build_only {
                input.parse::<Token![@]>()?;
            }
            input.parse::<Token![=]>()?;
            let value = input.parse()?;
            Ok(ElemModifier::Attr(name, value, build_only))
        }
    }
}

impl Parse for IfView {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![if]>()?;
        let cond = Expr::parse_without_eager_brace(input)?;
        let inner;
        braced!(inner in input);
        let body = inner.parse()?;
        let else_expr = if input.peek(Token![else]) {
            input.parse::<Token![else]>()?;
            Some(if input.peek(Token![if]) {
                Left(Box::new(input.parse()?))
            } else {
                let inner;
                braced!(inner in input);
                Right(inner.parse()?)
            })
        } else {
            None
        };

        Ok(Self {
            cond,
            body,
            else_expr,
        })
    }
}

impl Parse for ViewType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(token::Paren) {
            let inner;
            parenthesized!(inner in input);
            let expr = inner.parse()?;
            Ok(ViewType::Expr(expr))
        } else if input.peek(Token![for]) {
            input.parse::<Token![for]>()?;
            let pattern = Pat::parse_single(input)?;
            input.parse::<Token![in]>()?;
            let iter = input.parse()?;
            input.parse::<Token![=>]>()?;
            let key = Expr::parse_without_eager_brace(input)?;
            let inner;
            braced!(inner in input);
            let body = inner.parse()?;

            Ok(ViewType::For {
                pattern,
                iter,
                key,
                body,
            })
        } else if input.peek(Token![if]) {
            Ok(ViewType::If(input.parse()?))
        } else if input.peek(Token![dyn]) {
            input.parse::<Token![dyn]>()?;
            let inner;
            braced!(inner in input);
            let body = inner.parse()?;
            Ok(ViewType::Dyn(body))
        } else if input.peek(Token![use]) {
            input.parse::<Token![use]>()?;
            let name = input.parse()?;
            let typ = if input.peek(Token![:]) {
                input.parse::<Token![:]>()?;
                input.parse()?
            } else {
                parse_quote! { _ }
            };
            input.parse::<Token![=]>()?;
            let init = input.parse()?;
            input.parse::<Token![;]>()?;
            let body = input.parse()?;
            Ok(ViewType::State {
                name,
                typ,
                init,
                body,
            })
        } else {
            let name = input.parse()?;

            if input.peek(token::Paren) {
                let inner;
                parenthesized!(inner in input);
                let args = Punctuated::parse_terminated(&inner)?;
                let children = if input.peek(token::Brace) {
                    let inner;
                    braced!(inner in input);
                    Some(inner.parse()?)
                } else {
                    None
                };
                Ok(ViewType::Component {
                    name,
                    args,
                    children,
                })
            } else {
                let modifiers = if input.peek(token::Bracket) {
                    let inner;
                    bracketed!(inner in input);
                    Some(Punctuated::parse_terminated(&inner)?)
                } else {
                    None
                };
                let children = if input.peek(token::Brace) {
                    let inner;
                    braced!(inner in input);
                    Some(inner.parse()?)
                } else {
                    None
                };
                Ok(ViewType::Element {
                    name,
                    modifiers,
                    children,
                })
            }
        }
    }
}

impl Parse for ViewBody {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut views = vec![];
        while !input.is_empty() {
            views.push(input.parse()?);
        }
        Ok(ViewBody { views })
    }
}

impl IfView {
    pub fn gen_rust(&self) -> TokenStream {
        let body = self.body.gen_rust();
        let else_expr = match &self.else_expr {
            Some(Left(v)) => v.gen_rust(),
            Some(Right(v)) => v.gen_rust(),
            None => quote! { () },
        };
        let cond = &self.cond;
        quote! { if #cond { ::lunar::either::Either::Left(#body) } else { ::lunar::either::Either::Right(#else_expr) } }
    }
}

impl ViewType {
    pub fn gen_rust(&self) -> TokenStream {
        match self {
            ViewType::Element {
                name: typ,
                modifiers,
                children,
            } => {
                let mut out = quote! { ::lunar::el::<#typ>() };

                if let Some(children) = children {
                    let inner = children.gen_rust();
                    out.extend(quote! { .children(#inner) });
                }
                for m in modifiers.iter().flatten() {
                    match m {
                        ElemModifier::Attr(ident, expr, build_only) => out.extend(
                            quote! { .attr::<_, _, #build_only>(stringify!(#ident), #expr) },
                        ),
                        ElemModifier::Event(event, expr) => {
                            let func_name =
                                Ident::new(&format!("on_{}", event.typ), event.typ.span());
                            let arg = event.arg.as_ref().map(|v| quote! { stringify!(#v), });
                            out.extend(quote! { .#func_name(#arg #expr) })
                        }
                        ElemModifier::ThemeOverride { typ, name, value } => {
                            let typ = Ident::new(
                                &format!("ThemeOverride{}", typ.to_string().to_upper_camel_case()),
                                typ.span(),
                            );
                            out.extend(
                                quote! { .theme_override::<::lunar::#typ, _>(stringify!(#name), #value) },
                            )
                        }
                    }
                }
                out
            }
            ViewType::Component {
                name,
                args,
                children,
            } => {
                let children = children.as_ref().map(|v| v.gen_rust());
                let children = if let Some(children) = children {
                    quote! { , #children }
                } else {
                    quote! {}
                };
                quote! { #name(#args #children) }
            }
            ViewType::Expr(expr) => quote! { #expr },
            ViewType::For {
                pattern,
                iter,
                key,
                body,
            } => {
                let body = body.gen_rust();
                quote! { (#iter).into_iter().map(|#pattern| (#key, #body) ).collect::<Vec<_>>() }
            }
            ViewType::If(if_view) => if_view.gen_rust(),
            ViewType::Dyn(view_body) => {
                let body = view_body.gen_rust();
                quote! { ( Box::new(#body) as Box<dyn ::lunar::AnyView<_>> ) }
            }
            ViewType::State {
                name,
                typ,
                init,
                body,
            } => {
                let body = body.gen_rust();
                quote! { ::lunar::stateful::<#typ, _, _, _>(move || #init, move |#name| #body) }
            }
        }
    }
}

impl ViewBody {
    pub fn gen_rust(&self) -> TokenStream {
        let views = self.views.iter().map(|v| v.gen_rust());
        if self.views.len() == 1 {
            quote! { #(#views),* }
        } else {
            quote! { ( #(#views),* ) }
        }
    }
}
