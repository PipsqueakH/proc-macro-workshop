extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, spanned::Spanned, AngleBracketedGenericArguments, Attribute, Data,
    DataStruct, DeriveInput, Error, Fields, FieldsNamed, GenericArgument, Lit, Meta, MetaList,
    MetaNameValue, NestedMeta, PathArguments, Type, TypePath,
};

#[proc_macro_derive(Builder, attributes(builder))]
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

    // let mut err_code = proc_macro2::TokenStream::new();

    // eprintln! {"{:?}", err_code};

    let builder_fields = origin_fields.iter().map(|x| {
        let n = x.ident.as_ref();
        let ty = &x.ty;
        if x.attrs.len() != 0 {
            // if let Err(e) = setter_name(&x.attrs[0]) {
            //     err_code = e.to_compile_error();
            //     println!("err code is{:?}", err_code);
            // }
            let inner_ty = type_in_container("Vec", ty);
            if inner_ty.is_some() {
                return quote! { #n:  #ty};
            }
        }
        let optioned_ty = type_in_container("Option", &x.ty);
        let t = optioned_ty.map_or(&x.ty, |x| x);
        quote! { #n: std::option::Option< #t>}
    });

    let builder_fields_name = origin_fields.iter().map(|x| {
        let n = x.ident.as_ref();
        let ty = &x.ty;
        if x.attrs.len() != 0 {
            let inner_ty = type_in_container("Vec", ty);
            if inner_ty.is_some() {
                return quote! { #n:  Vec::new()};
            }
        }
        quote! { #n: std::option::Option::None}
    });

    let setter_method = origin_fields.iter().map(|x| {
        let n = x.ident.as_ref();

        println! {" attts: {:?}", x.attrs};
        if x.attrs.len() != 0 {
            let setter_name = setter_name(&x.attrs[0]);
            println! {" got attr: {:?}", setter_name};
            let veced_ty = type_in_container("Vec", &x.ty);
            if setter_name.is_ok() && veced_ty.is_some() {
                let sn = setter_name.ok().unwrap();
                // eprintln!("setter name is {:?}", sn);

                return quote! {
                    fn #sn(&mut self, #sn: #veced_ty) -> &mut Self {
                        self.#n.push(#sn);
                        self
                    }
                };
            } else {
                if let Err(e) = setter_name {
                    return e.to_compile_error();
                }
            }
        }

        let optioned_ty = type_in_container("Option", &x.ty);
        let t = optioned_ty.map_or(&x.ty, |x| x);
        quote! {
            fn #n(&mut self, #n: #t) -> &mut Self {
                self.#n = std::option::Option::Some(#n);
                self
            }
        }
    });

    let built_fields = origin_fields.iter().map(|x| {
        let n = x.ident.as_ref();
        let err_msg = concat!(stringify!(n));
        if type_in_container("Option", &x.ty).is_some() || x.attrs.len() != 0 {
            quote!{
                #n: self.#n.clone() 
            }
        } else {
            quote! {
            #n: self.#n.clone().ok_or_else::<std::boxed::Box<dyn std::error::Error>, _>(||::std::convert::From::from(#err_msg))?
            }
        }
        
    });

    // let field_name = name_iter.collect();
    // eprintln!("TOKENS: {}", name_iter[0]);
    // eprintln!("TOKENS: {:?}", builder_fields_name);

    let expanded = quote! {


        // use std::error::Error;

        pub struct #builder_name  { #( #builder_fields ,)*}

        // #err_code;

        // The generated impl.
        impl  #name  {
            pub fn builder() -> #builder_name {
                #builder_name { #( #builder_fields_name,)*}
            }
        }

        impl #builder_name {
            #(#setter_method)*

        }



        impl #builder_name {
            pub fn build(&mut self) -> std::result::Result<#name, std::boxed::Box<dyn std::error::Error>> {

                Ok(Command {#(#built_fields ,)*})
            }
        }

    };

    // eprintln!("TOKENS: {}", expanded);
    // Hand the output tokens back to the compiler.
    proc_macro::TokenStream::from(expanded)
}

fn type_in_container<'a>(container: &str, ty: &'a Type) -> std::option::Option<&'a Type> {
    if let Type::Path(TypePath { ref path, .. }) = ty {
        let first_path = path.segments.first().unwrap();
        if first_path.ident == container {
            if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                ref args, ..
            }) = first_path.arguments
            {
                let inner_ty = args.first().unwrap();
                if let GenericArgument::Type(ref ty) = inner_ty {
                    return std::option::Option::Some(ty);
                }
            }
        }
    }
    std::option::Option::None
}

fn setter_name(attr: &Attribute) -> syn::Result<Ident> {
    match attr.parse_meta() {
        Ok(ref meta) => {
            // println!("meta: {:?}", meta);
            if let Meta::List(MetaList { ref nested, .. }) = meta {
                if nested.len() == 1 {
                    if let NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                        ref path,
                        ref lit,
                        ..
                    })) = nested.first().unwrap()
                    {
                        // println!("path is {:?}, lit is {:?}", path, lit);
                        let attr_name = path.segments.first().unwrap();
                        if attr_name.ident == "each" {
                            if let Lit::Str(ref s) = lit {
                                return Ok(Ident::new(&s.value(), s.span()));
                            }
                        } else {
                            // compile_error!("aaa");
                            // println!(" ident is {:?}", attr_name.ident);

                            return Err(Error::new(
                                meta.span(),
                                "expected `builder(each = \"...\")`",
                            ));
                        }
                    }
                }
            }
        }
        Err(err) => {
            println!("parse setter name err: {:?}", err);
            return Err(Error::new(Span::call_site(), "parse attribute error"));
        }
    }

    return Err(Error::new(Span::call_site(), "parse attribute error"));
}
