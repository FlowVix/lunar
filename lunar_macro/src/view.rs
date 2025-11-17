use either::Either::{self, Left, Right};
use heck::ToUpperCamelCase;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    AngleBracketedGenericArguments, Expr, Ident, Pat, Token, Type, braced, bracketed,
    parenthesized, parse::Parse, parse_quote, punctuated::Punctuated, token,
};

use crate::util::take_until_semicolon;

mod kw {
    syn::custom_keyword!(when);
    syn::custom_keyword!(state);
    syn::custom_keyword!(quiet);
    syn::custom_keyword!(build);
    syn::custom_keyword!(memo);
}

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
        kw: token::For,
        pattern: Pat,
        iter: Expr,
        key: Expr,
        body: ViewBody,
    },
    If(IfView),
    Dyn(ViewBody),
    State {
        kw: kw::state,
        quiet: Option<kw::quiet>,
        name: Ident,
        typ: Type,
        init: TokenStream,
        body: ViewBody,
    },
    When {
        kw: kw::when,
        expr: Expr,
        body: ViewBody,
    },
    Memo {
        kw: kw::memo,
        expr: Expr,
        body: ViewBody,
    },
    Let {
        pat: Pat,
        typ: Type,
        value: TokenStream,
        body: ViewBody,
    },
}

pub enum ElemModifier {
    Attr(Ident, Expr, Option<kw::build>),
    OnSignal(Ident, Expr),
    ThemeOverride {
        typ: Ident,
        name: Ident,
        value: Expr,
    },
    NodeRef(Expr),
}

impl Parse for ElemModifier {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![@]) {
            input.parse::<Token![@]>()?;
            let name = input.parse()?;
            input.parse::<Token![=]>()?;
            let value = input.parse()?;
            Ok(ElemModifier::OnSignal(name, value))
        } else if input.peek(Token![#]) {
            input.parse::<Token![#]>()?;
            let typ = input.parse()?;
            input.parse::<Token![:]>()?;
            let name = input.parse()?;
            input.parse::<Token![=]>()?;
            let value = input.parse()?;
            Ok(ElemModifier::ThemeOverride { typ, name, value })
        } else if input.peek(Token![ref]) {
            input.parse::<Token![ref]>()?;
            let inner;
            parenthesized!(inner in input);
            let expr = inner.parse()?;
            Ok(ElemModifier::NodeRef(expr))
        } else {
            let build = if input.peek(kw::build) {
                Some(input.parse::<kw::build>()?)
            } else {
                None
            };
            let name = input.parse()?;
            input.parse::<Token![=]>()?;
            let value = input.parse()?;
            Ok(ElemModifier::Attr(name, value, build))
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
            let kw = input.parse::<Token![for]>()?;
            let pattern = Pat::parse_single(input)?;
            input.parse::<Token![in]>()?;
            let iter = input.parse()?;
            input.parse::<Token![=>]>()?;
            let key = Expr::parse_without_eager_brace(input)?;
            let inner;
            braced!(inner in input);
            let body = inner.parse()?;

            Ok(ViewType::For {
                kw,
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
        } else if input.peek(kw::state) {
            let kw = input.parse::<kw::state>()?;
            let quiet = if input.peek(kw::quiet) {
                Some(input.parse::<kw::quiet>()?)
            } else {
                None
            };
            let name = input.parse()?;
            let typ = if input.peek(Token![:]) {
                input.parse::<Token![:]>()?;
                input.parse()?
            } else {
                parse_quote! { _ }
            };
            input.parse::<Token![=]>()?;
            let init = take_until_semicolon(input)?;
            let body = input.parse()?;
            Ok(ViewType::State {
                kw,
                quiet,
                name,
                typ,
                init,
                body,
            })
        } else if input.peek(Token![let]) {
            input.parse::<Token![let]>()?;
            let pat = Pat::parse_single(input)?;
            let typ = if input.peek(Token![:]) {
                input.parse::<Token![:]>()?;
                input.parse()?
            } else {
                parse_quote! { _ }
            };
            input.parse::<Token![=]>()?;
            let value = take_until_semicolon(input)?;
            let body = input.parse()?;
            Ok(ViewType::Let {
                pat,
                typ,
                value,
                body,
            })
        } else if input.peek(kw::when) {
            let kw = input.parse::<kw::when>()?;
            let expr = Expr::parse_without_eager_brace(input)?;
            let inner;
            braced!(inner in input);
            let body = inner.parse()?;
            Ok(ViewType::When { kw, expr, body })
        } else if input.peek(kw::memo) {
            let kw = input.parse::<kw::memo>()?;
            let expr = Expr::parse_without_eager_brace(input)?;
            let inner;
            braced!(inner in input);
            let body = inner.parse()?;
            Ok(ViewType::Memo { kw, expr, body })
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
                        ElemModifier::Attr(ident, expr, build) => {
                            let build = build.map(|v| Ident::new("try", v.span));
                            if let Some(build) = build {
                                out.extend(
                                    quote! { .attr_build({ stringify!(#build); stringify!(#ident) }, #expr) },
                                );
                            } else {
                                out.extend(quote! { .attr(stringify!(#ident), #expr) });
                            }
                        }
                        ElemModifier::OnSignal(name, expr) => {
                            out.extend(quote! { .on_signal(stringify!(#name), #expr) });
                        }
                        ElemModifier::ThemeOverride { typ, name, value } => {
                            let typ = Ident::new(
                                &format!("ThemeOverride{}", typ.to_string().to_upper_camel_case()),
                                typ.span(),
                            );
                            out.extend(
                                quote! { .theme_override::<::lunar::#typ, _>(stringify!(#name), #value) },
                            );
                        }
                        ElemModifier::NodeRef(expr) => {
                            out.extend(quote! { .node_ref(#expr) });
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
                kw,
                pattern,
                iter,
                key,
                body,
            } => {
                let body = body.gen_rust();
                quote! {
                    {
                        #kw _ in [()] {};
                        (#iter).into_iter().map(move |#pattern| (#key, #body)).collect::<Vec<_>>()
                    }
                }
            }
            ViewType::If(if_view) => if_view.gen_rust(),
            ViewType::Dyn(view_body) => {
                let body = view_body.gen_rust();
                quote! { { #[allow(clippy::double_parens)] ( Box::new(#body) as Box<dyn ::lunar::AnyView> ) } }
            }
            ViewType::State {
                kw,
                quiet,
                name,
                typ,
                init,
                body,
            } => {
                let body = body.gen_rust();
                let kw = Ident::new("try", kw.span);
                let quiet = quiet.map(|v| Ident::new("try", v.span));
                if let Some(quiet) = quiet {
                    quote! {
                        {
                            stringify!(#kw);
                            stringify!(#quiet);
                            ::lunar::stateful_quiet::<#typ, _, _, _>(move || #init, move |#name| #body)
                        }
                    }
                } else {
                    quote! {
                        {
                            stringify!(#kw);
                            ::lunar::stateful::<#typ, _, _, _>(move || #init, move |#name| #body)
                        }
                    }
                }
            }
            ViewType::When { kw, expr, body } => {
                let body = body.gen_rust();
                let kw = Ident::new("yield", kw.span);
                quote! {
                    {
                        stringify!(#kw);
                        ::lunar::when(#expr, || #body)
                    }
                }
            }
            ViewType::Memo { kw, expr, body } => {
                let body = body.gen_rust();
                let kw = Ident::new("yield", kw.span);
                quote! {
                    {
                        stringify!(#kw);
                        ::lunar::memo(#expr, || #body)
                    }
                }
            }
            ViewType::Let {
                pat,
                typ,
                value,
                body,
            } => {
                let body = body.gen_rust();
                quote! {
                    {
                        let #pat: #typ = #value;
                        #body
                    }
                }
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
