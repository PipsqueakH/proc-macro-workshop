extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);

    // Used in the quasi-quotation below as `#name`.
    let name = input.ident;
    let builder_name = format_ident!("{}Builder", name);

    let (name_vec, type_vec): (Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>) =
        match input.data {
            Data::Struct(ref data) => match data.fields {
                Fields::Named(ref fields) => fields
                    .named
                    .iter()
                    .map(|f| {
                        let n = f.ident.as_ref();
                        let t = &f.ty;
                        (quote! {#n}, quote! {#t})
                    })
                    .unzip(),
                Fields::Unnamed(_) | Fields::Unit => unimplemented!(),
            },
            Data::Enum(_) | Data::Union(_) => unimplemented!(),
        };

    // let field_name = name_iter.collect();
    // eprintln!("TOKENS: {}", name_iter[0]);
    // eprintln!("TOKENS: {}", field_type);

    let expanded = quote! {

        use std::error::Error;

        pub struct #builder_name  { #( #name_vec : Option< #type_vec>,)*}

        // The generated impl.
        impl  #name  {
            pub fn builder() -> #builder_name {
                #builder_name { #( #name_vec : None,)*}
            }
        }

        impl #builder_name {
            #(fn #name_vec(&mut self, #name_vec: #type_vec) -> &mut Self {
                self.#name_vec = Some(#name_vec);
                self
            })*

        }



        impl #builder_name {
            pub fn build(&mut self) -> Result<#name, Box<dyn Error>> {
                #(let #name_vec = self.#name_vec.clone().ok_or_else::<Box<dyn Error>, _>(||::std::convert::From::from("there is missing field"))?;)*
                Ok(Command {#(#name_vec ,)*})
            }
        }

    };

    // eprintln!("TOKENS: {}", expanded);
    // Hand the output tokens back to the compiler.
    proc_macro::TokenStream::from(expanded)
}
