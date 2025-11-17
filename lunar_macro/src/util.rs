use proc_macro2::TokenStream;
use syn::{Result, Token, parse::ParseStream};

pub fn take_until_semicolon(input: ParseStream) -> Result<TokenStream> {
    let mut depth = 0usize;
    let mut out = TokenStream::new();

    while !input.is_empty() {
        let look = input.fork();
        let tt: proc_macro2::TokenTree = look.parse()?;

        match &tt {
            proc_macro2::TokenTree::Punct(p) if p.as_char() == ';' && depth == 0 => {
                input.parse::<Token![;]>()?;
                break;
            }

            proc_macro2::TokenTree::Punct(p) => {
                match p.as_char() {
                    '(' | '[' | '{' => depth += 1,
                    ')' | ']' | '}' => depth -= 1,
                    _ => {}
                }
                let tt: proc_macro2::TokenTree = input.parse()?;
                out.extend([tt]);
            }

            _ => {
                let tt: proc_macro2::TokenTree = input.parse()?;
                out.extend([tt]);
            }
        }
    }

    Ok(out)
}
