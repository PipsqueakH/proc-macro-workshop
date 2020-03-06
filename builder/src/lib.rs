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

    // let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

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



        // impl #builder_name {
        //     pub fn build(&mut self) -> Result<Command, Box<dyn Error>> {
        //         let ex = self.executable.clone().ok_or_else::<Box<dyn Error>, _>(||::std::convert::From::from("has no executable"))?;
        //         let ar = self.args.clone().ok_or_else::<Box<dyn Error>, _>(||::std::convert::From::from("has no args"))?;
        //         let en = self.env.clone().ok_or_else::<Box<dyn Error>, _>(||::std::convert::From::from("has no enc"))?;
        //         let cd = self.current_dir.clone().ok_or_else::<Box<dyn Error>, _>(||::std::convert::From::from("has no current_dir"))?;

        //         Ok(Command {
        //             executable: ex,
        //             args: ar,
        //             env: en,
        //             current_dir: cd,
        //         })
        //     }
        // }

    };

    eprintln!("TOKENS: {}", expanded);
    // Hand the output tokens back to the compiler.
    proc_macro::TokenStream::from(expanded)
}
