use quote::ToTokens;
use syn::{
    braced,
    parse::{self, Parse},
    punctuated::Punctuated,
    Expr, Ident, Token, Type, Visibility,
};

pub struct Struct {
    pub vis: Visibility,
    pub struct_token: Token![struct],
    pub name: Ident,
    pub brace_token: syn::token::Brace,
    pub fields: Punctuated<Field, Token![,]>,
}

pub struct Field {
    pub vis: Visibility,
    pub ident: Ident,
    pub colon_token: Token![:],
    pub ty: Type,
    pub default_value: DefaultValue,
}

pub enum DefaultValue {
    None,
    Present(DefaultValueExpr),
}

pub struct DefaultValueExpr {
    pub eq_token: Token![=],
    pub expr: Expr,
}

impl Parse for Struct {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let content;

        Ok(Struct {
            vis: input.parse()?,
            struct_token: input.parse::<Token![struct]>()?,
            name: input.parse()?,
            brace_token: braced!(content in input),
            fields: content.parse_terminated(Field::parse)?,
        })
    }
}

impl ToTokens for Struct {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.vis.to_tokens(tokens);
        self.struct_token.to_tokens(tokens);
        self.name.to_tokens(tokens);
        self.brace_token.surround(tokens, |tokens| {
            self.fields.to_tokens(tokens);
        });
    }
}


impl Parse for Field {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        Ok(Field {
            vis: input.parse()?,
            ident: input.parse()?,
            colon_token: input.parse()?,
            ty: input.parse()?,
            default_value: input.parse()?,
        })
    }
}

impl ToTokens for Field {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.vis.to_tokens(tokens);
        self.ident.to_tokens(tokens);
        self.colon_token.to_tokens(tokens);
        self.ty.to_tokens(tokens);
        self.default_value.to_tokens(tokens);
    }
}

impl Parse for DefaultValue {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![=]) {
            Ok(DefaultValue::Present(input.parse()?))
        } else {
            Ok(DefaultValue::None)
        }
    }
}

impl ToTokens for DefaultValue {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            DefaultValue::None => {}
            DefaultValue::Present(default_value_expr) => default_value_expr.to_tokens(tokens),
        }
    }
}

impl Parse for DefaultValueExpr {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        Ok(DefaultValueExpr{
            eq_token: input.parse()?,
            expr: input.parse()?,
        })
    }
}

impl ToTokens for DefaultValueExpr {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.eq_token.to_tokens(tokens);
        self.expr.to_tokens(tokens);
    }
}
