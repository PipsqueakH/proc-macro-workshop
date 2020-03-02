extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);

    // Used in the quasi-quotation below as `#name`.
    let name = input.ident;

    // let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let builder_name = format_ident!("{}Builder", name);

    let expanded = quote! {

        pub struct #builder_name  {
            executable: Option<String>,
            args: Option<Vec<String>>,
            env: Option<Vec<String>>,
            current_dir: Option<String>,
        }

        // The generated impl.
        impl  #name  {
            pub fn builder() -> CommandBuilder {
                CommandBuilder {
                    executable: None,
                    args: None,
                    env: None,
                    current_dir: None,
                }
            }
        }

        impl #builder_name {
            fn executable(&mut self, executable: String) -> &mut Self {
                self.executable = Some(executable);
                self
            }
        }

        impl #builder_name {
            fn args(&mut self, args: Vec<String>) -> &mut Self {
                self.args = Some(args);
                self
            }
        }

        impl #builder_name {
            fn env(&mut self, env: Vec<String>) -> &mut Self {
                self.env = Some(env);
                self
            }
        }

        impl #builder_name {
            fn current_dir(&mut self, current_dir: String) -> &mut Self {
                self.current_dir = Some(current_dir);
                self
            }
        }
    };

    // Hand the output tokens back to the compiler.
    proc_macro::TokenStream::from(expanded)
}
