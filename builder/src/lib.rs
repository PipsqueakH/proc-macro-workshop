extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, AngleBracketedGenericArguments, Data, DataStruct, DeriveInput, Fields,
    FieldsNamed, GenericArgument, Path, PathArguments, Type, TypePath,
};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);

    // Used in the quasi-quotation below as `#name`.
    let name = input.ident;
    let builder_name = format_ident!("{}Builder", name);

    let origin_fields = if let Data::Struct(DataStruct {
        fields: Fields::Named(FieldsNamed { ref named, .. }, ..),
        ..
    }) = input.data
    {
        named
    } else {
        unimplemented! {}
    };

    let builder_fields = origin_fields.iter().map(|x| {
        let n = x.ident.as_ref();
        let t = &x.ty;
        quote! { #n: Option< #t>}
    });

    let builder_fields_name = origin_fields.iter().map(|x| {
        let n = x.ident.as_ref();
        quote! { #n}
    });

    let setter_method = origin_fields.iter().map(|x| {
        let n = x.ident.as_ref();
        let t = &x.ty;
        quote! {
            fn #n(&mut self, #n: #t) -> &mut Self {
                self.#n = Some(#n);
                self
            }
        }
    });

    let built_fields = origin_fields.iter().map(|x| {
        let n = x.ident.as_ref();
        let err_msg = concat!(stringify!(n));
        quote! {
            #n: self.#n.clone().ok_or_else::<Box<dyn Error>, _>(||::std::convert::From::from(#err_msg))?
        }
    });

    // let field_name = name_iter.collect();
    // eprintln!("TOKENS: {}", name_iter[0]);
    // eprintln!("TOKENS: {:?}", builder_fields_name);

    let expanded = quote! {

        use std::error::Error;

        pub struct #builder_name  { #( #builder_fields ,)*}

        // The generated impl.
        impl  #name  {
            pub fn builder() -> #builder_name {
                #builder_name { #( #builder_fields_name : None,)*}
            }
        }

        impl #builder_name {
            #(#setter_method)*

        }



        impl #builder_name {
            pub fn build(&mut self) -> Result<#name, Box<dyn Error>> {

                Ok(Command {#(#built_fields ,)*})
            }
        }

    };

    // eprintln!("TOKENS: {}", expanded);
    // Hand the output tokens back to the compiler.
    proc_macro::TokenStream::from(expanded)
}
