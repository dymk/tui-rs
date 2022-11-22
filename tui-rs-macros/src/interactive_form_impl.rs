use proc_macro2::{Ident, Literal, Span, TokenStream};
use proc_macro_crate::crate_name;
use quote::quote;
use quote::{ToTokens, TokenStreamExt};
use syn::parse::{Parse, Parser};

use syn::{Expr, Fields};

use crate::def_struct;

const DEFAULT_FIELD_ATTR: &str = "default";

pub(crate) fn process(mut struct_ast: syn::ItemStruct) -> TokenStream {
    let mut stream = TokenStream::new();

    let tui_crate_toks = crate_name("tui")
        .map_or("::tui".to_string(), |crate_name| match crate_name {
            proc_macro_crate::FoundCrate::Itself => "crate".to_string(),
            proc_macro_crate::FoundCrate::Name(name) => name,
        })
        .parse()
        .unwrap();

    let tui_crate_path = syn::Path::parse.parse2(tui_crate_toks).unwrap();

    // before modifying the struct, create impl blocks
    let impl_default_tokstream = make_impl_blocks(&tui_crate_path, &struct_ast).unwrap();

    // remove `default` annotations on the fields so rust doesn't try to
    // expand it
    for field in &mut struct_ast.fields {
        field
            .attrs
            .retain(|attr| !attr.path.is_ident(DEFAULT_FIELD_ATTR))
    }

    // suffix with the focused state field
    push_focused_state_field(&mut struct_ast).unwrap();
    struct_ast.to_tokens(&mut stream);

    // add in impl blocks
    stream.append_all(impl_default_tokstream);
    stream
}

struct ColonJoined<'a>(&'a def_struct::Field, &'a Expr);
impl<'a> ToTokens for ColonJoined<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.0.ident.to_tokens(tokens);
        syn::token::Colon::default().to_tokens(tokens);
        self.1.to_tokens(tokens);
    }
}

fn push_focused_state_field(struct_ast: &mut syn::ItemStruct) -> Result<(), syn::Error> {
    match &mut struct_ast.fields {
        syn::Fields::Named(fields) => {
            let field = syn::Field::parse_named.parse2(quote! {
                _focused_state_idx: Option<usize>
            })?;
            fields.named.push(field);
            Ok(())
        }
        syn::Fields::Unnamed(fields) => {
            let field = syn::Field::parse_unnamed.parse2(quote! {
                Option<usize>
            })?;
            fields.unnamed.push(field);
            Ok(())
        }
        syn::Fields::Unit => Ok(()),
    }
}

fn make_impl_blocks(
    crate_ident: &syn::Path,
    struct_ast: &syn::ItemStruct,
) -> Result<TokenStream, syn::Error> {
    let mut stream = TokenStream::new();
    make_impl_default_block(struct_ast)?.to_tokens(&mut stream);
    make_impl_form_hooks_block(crate_ident, struct_ast)?.to_tokens(&mut stream);
    Ok(stream)
}

fn make_impl_default_block(struct_ast: &syn::ItemStruct) -> Result<TokenStream, syn::Error> {
    let name = &struct_ast.ident;

    let default_fn = match &struct_ast.fields {
        syn::Fields::Named(fields) => make_impl_default_fn(true, fields.named.iter())?,
        syn::Fields::Unnamed(fields) => make_impl_default_fn(false, fields.unnamed.iter())?,
        syn::Fields::Unit => quote! {
            fn default() -> Self { #name }
        },
    };

    Ok(quote! {
        impl ::core::default::Default for #name {
            #default_fn
        }
    })
}

fn get_field_prefix(named: bool, ident: &Ident) -> TokenStream {
    if named {
        quote! { #ident: }
    } else {
        quote! {}
    }
}

fn make_impl_default_fn<'a>(
    named: bool,
    fields: impl Iterator<Item = &'a syn::Field>,
) -> Result<TokenStream, syn::Error> {
    let fields_and_initializers = fields
        .map(|field| {
            let field_default_expr = get_field_default(field)?;
            Ok(match &field.ident {
                Some(ident) => {
                    let ident_pfx = get_field_prefix(named, ident);
                    quote! {
                        #ident_pfx #field_default_expr
                    }
                }
                None => field_default_expr.into_token_stream(),
            })
        })
        .collect::<Result<Vec<_>, syn::Error>>()?;

    let focused_ident_pfx =
        get_field_prefix(named, &Ident::new("_focused_state_idx", Span::call_site()));

    let initializer_list = quote! {
        #(
            #fields_and_initializers,
        )*
        #focused_ident_pfx None
    };

    let self_initializer = if named {
        quote! { Self { #initializer_list } }
    } else {
        quote! { Self( #initializer_list ) }
    };

    Ok(quote! {
        fn default() -> Self {
            #self_initializer
        }
    })
}

fn get_field_default(field: &syn::Field) -> Result<syn::Expr, syn::Error> {
    if let Some(default_attr) = field
        .attrs
        .iter()
        .find(|attr| attr.path.is_ident(DEFAULT_FIELD_ATTR))
    {
        let default_expr = &default_attr.tokens;
        syn::Expr::parse.parse2(quote! {
            #default_expr.into()
        })
    } else {
        syn::Expr::parse.parse2(quote! {
            ::core::default::Default::default()
        })
    }
}

fn make_impl_form_hooks_block(
    crate_ident: &syn::Path,
    struct_ast: &syn::ItemStruct,
) -> Result<TokenStream, syn::Error> {
    let name = &struct_ast.ident;
    let num_fields = Literal::usize_unsuffixed(struct_ast.fields.len()).into_token_stream();
    let named = matches!(struct_ast.fields, Fields::Named(_));
    let unit = matches!(struct_ast.fields, Fields::Unit);

    let fields_and_names: Vec<TokenStream> = struct_ast
        .fields
        .iter()
        .enumerate()
        .map(|(idx, field)| {
            let field_name = &field.ident.as_ref().map_or_else(
                || Literal::usize_unsuffixed(idx).into_token_stream(),
                |ident| ident.clone().into_token_stream(),
            );
            let field_name_str = &field.ident.as_ref().map_or_else(
                || Literal::string(&idx.to_string()).into_token_stream(),
                |ident| Literal::string(&ident.to_string()).into_token_stream(),
            );
            quote! {
                (#field_name_str, &self.#field_name)
            }
        })
        .collect();

    let (input_state_at_non_mut, input_state_at_with_mut): (Vec<_>, Vec<_>) = struct_ast
        .fields
        .iter()
        .enumerate()
        .map(|(idx, field)| {
            let idx_lit = Literal::usize_unsuffixed(idx);
            let field_name = &field.ident.as_ref().map_or_else(
                || Literal::usize_unsuffixed(idx).into_token_stream(),
                |ident| ident.clone().into_token_stream(),
            );

            let non_mut = quote! {
                if idx == #idx_lit {
                    return Some(&self.#field_name);
                }
            };
            let with_mut = quote! {
                if idx == #idx_lit {
                    return Some(&mut self.#field_name);
                }
            };
            (non_mut, with_mut)
        })
        .unzip();

    let idx_param = if unit {
        quote! { _idx }
    } else {
        quote! { idx }
    };

    let focused_idx_field_tok = if named {
        quote! { _focused_state_idx }
    } else {
        num_fields.clone()
    };

    let get_focused_idx_body = if unit {
        quote! { None }
    } else {
        quote! {
            self.#focused_idx_field_tok
        }
    };

    let set_focused_idx_body = if unit {
        quote! {}
    } else {
        quote! {
            self.#focused_idx_field_tok = #idx_param;
        }
    };

    Ok(quote! {
        impl #name {
            pub fn as_list(&self) -> [(&str, &dyn (#crate_ident::widgets::InteractiveWidgetState)); #num_fields] {
                [#(#fields_and_names),*]
            }
        }

        impl #crate_ident::interactive_form::InteractiveFormHooks for #name {
            fn focused_state_idx(&self)
                -> Option<usize>
            {
                #get_focused_idx_body
            }
            fn set_focused_state_idx(&mut self, #idx_param: Option<usize>)
            {
                #set_focused_idx_body
            }

            fn input_states_len(&self)
                -> usize
            {
                #num_fields
            }
            fn input_state_at(&self, #idx_param: usize)
                -> Option<&dyn (#crate_ident::widgets::InteractiveWidgetState)>
            {
                #(#input_state_at_non_mut)*
                return None;
            }
            fn input_state_at_mut(&mut self, #idx_param: usize)
                -> Option<&mut dyn (#crate_ident::widgets::InteractiveWidgetState)>
            {
                #(#input_state_at_with_mut)*
                return None;
            }
        }
    })
}

#[cfg(test)]
mod test {
    use crate::interactive_form_impl::process;
    use pretty_assertions::assert_eq;
    use proc_macro2::TokenStream;
    use quote::quote;
    use syn::parse::{Parse, Parser};

    fn format(s: &TokenStream) -> String {
        let ast = match syn::File::parse.parse2(s.clone()) {
            Ok(ast) => ast,
            Err(err) => {
                return format!("Error parsing tokstream\n=====\n{}\n======\n: {}\n", s, err)
            }
        };
        prettyplease::unparse(&ast)
    }

    #[test]
    fn test_named_works() {
        let input_struct_toks = quote! {
            struct MyForm {
                #[default("FooDefault")]
                foo: TextInputState,
                bar: TextInputState,
            }
        };

        let expected_toks = quote! {
            struct MyForm {
                foo: TextInputState,
                bar: TextInputState,
                _focused_state_idx: Option<usize>,
            }
            impl ::core::default::Default for MyForm {
                fn default() -> Self {
                    Self {
                        foo: ("FooDefault").into(),
                        bar: ::core::default::Default::default(),
                        _focused_state_idx: None,
                    }
                }
            }
            impl MyForm {
                pub fn as_list(&self) -> [(&str, &dyn (::tui::widgets::InteractiveWidgetState)); 2] {
                    [("foo", &self.foo), ("bar", &self.bar)]
                }
            }
            impl ::tui::interactive_form::InteractiveFormHooks for MyForm {
                fn focused_state_idx(&self) -> Option<usize> {
                    self._focused_state_idx
                }
                fn set_focused_state_idx(&mut self, idx: Option<usize>) {
                    self._focused_state_idx = idx;
                }
                fn input_states_len(&self) -> usize {
                    2
                }
                fn input_state_at(
                    &self, idx: usize
                ) -> Option<&dyn (::tui::widgets::InteractiveWidgetState)> {
                    if idx == 0 {
                        return Some(&self.foo);
                    }
                    if idx == 1 {
                        return Some(&self.bar);
                    }
                    return None;
                }
                fn input_state_at_mut(
                    &mut self,
                    idx: usize,
                ) -> Option<&mut dyn (::tui::widgets::InteractiveWidgetState)> {
                    if idx == 0 {
                        return Some(&mut self.foo);
                    }
                    if idx == 1 {
                        return Some(&mut self.bar);
                    }
                    return None;
                }
            }
        };

        compare_processed(input_struct_toks, expected_toks);
    }

    #[test]
    fn test_unnamed_works() {
        let input_struct_toks = quote! {
            struct MyForm(
                #[default("ZeroDefault")]
                TextInputState,
                TextInputState,
            );
        };

        let expected_toks = quote! {
            struct MyForm(TextInputState, TextInputState, Option<usize>);
            impl ::core::default::Default for MyForm {
                fn default() -> Self {
                    Self(
                        ("ZeroDefault").into(),
                        ::core::default::Default::default(),
                        None,
                    )
                }
            }
            impl MyForm {
                pub fn as_list(&self) -> [(&str, &dyn (::tui::widgets::InteractiveWidgetState)); 2] {
                    [("0", &self.0), ("1", &self.1)]
                }
            }
            impl ::tui::interactive_form::InteractiveFormHooks for MyForm {
                fn focused_state_idx(&self) -> Option<usize> {
                    self.2
                }
                fn set_focused_state_idx(&mut self, idx: Option<usize>) {
                    self.2 = idx;
                }
                fn input_states_len(&self) -> usize {
                    2
                }
                fn input_state_at(
                    &self, idx: usize
                ) -> Option<&dyn (::tui::widgets::InteractiveWidgetState)> {
                    if idx == 0 {
                        return Some(&self.0);
                    }
                    if idx == 1 {
                        return Some(&self.1);
                    }
                    return None;
                }
                fn input_state_at_mut(
                    &mut self,
                    idx: usize,
                ) -> Option<&mut dyn (::tui::widgets::InteractiveWidgetState)> {
                    if idx == 0 {
                        return Some(&mut self.0);
                    }
                    if idx == 1 {
                        return Some(&mut self.1);
                    }
                    return None;
                }
            }
        };

        compare_processed(input_struct_toks, expected_toks);
    }

    #[test]
    fn test_empty_works() {
        let input_struct_toks = quote! {
            struct MyForm;
        };
        let expected_toks = quote! {
            struct MyForm;
            impl ::core::default::Default for MyForm {
                fn default() -> Self {
                    MyForm
                }
            }
            impl MyForm {
                pub fn as_list(&self) -> [(&str, &dyn (::tui::widgets::InteractiveWidgetState)); 0] {
                    []
                }
            }
            impl ::tui::interactive_form::InteractiveFormHooks for MyForm {
                fn focused_state_idx(&self) -> Option<usize> {
                    None
                }
                fn set_focused_state_idx(&mut self, _idx: Option<usize>) {}
                fn input_states_len(&self) -> usize {
                    0
                }
                fn input_state_at(
                    &self, _idx: usize
                ) -> Option<&dyn (::tui::widgets::InteractiveWidgetState)> {
                    return None;
                }
                fn input_state_at_mut(
                    &mut self,
                    _idx: usize,
                ) -> Option<&mut dyn (::tui::widgets::InteractiveWidgetState)> {
                    return None;
                }
            }
        };

        compare_processed(input_struct_toks, expected_toks);
    }

    fn compare_processed(input_toks: TokenStream, expected_toks: TokenStream) {
        let processed_toks = process(syn::parse2(input_toks).unwrap());
        let processed_formatted = format(&processed_toks);
        let expected_formatted = format(&expected_toks);
        if processed_formatted != expected_formatted {
            println!("\n==========\n{}==========\n", processed_formatted);
        }
        assert_eq!(processed_formatted, expected_formatted);
    }
}
