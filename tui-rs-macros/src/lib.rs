extern crate proc_macro;
extern crate proc_macro2;
extern crate proc_macro_crate;
extern crate quote;
extern crate syn;

#[cfg(test)]
extern crate pretty_assertions;
#[cfg(test)]
extern crate prettyplease;

use proc_macro::TokenStream;

mod def_struct;
mod interactive_form_impl;

#[proc_macro_attribute]
pub fn interactive_form(_args: TokenStream, input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    interactive_form_impl::process(ast).into()
}
