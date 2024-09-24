use std::collections::HashSet;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse::Parse, parse_macro_input, punctuated::Punctuated, DeriveInput, Ident, ItemImpl, LitStr,
    Token,
};

struct PluginAttr {
    plugin_type: Ident,
    dependencies: Punctuated<LitStr, Token![,]>,
}

impl Parse for PluginAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let plugin_type = input.parse()?;
        let dependencies = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            Punctuated::parse_terminated(input)?
        } else {
            Punctuated::new()
        };
        Ok(PluginAttr {
            plugin_type,
            dependencies,
        })
    }
}

// note: maybe we should seperate the type/name definitions from the dependencies listing
// also, either we implictly need to state the dependencies to bring them in scope, either we
// resolve them automatically whenever they are used, this is TBD
#[proc_macro_attribute]
pub fn plugin(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let attr = parse_macro_input!(attr as PluginAttr);

    let name = &input.ident;
    let plugin_type = &attr.plugin_type;

    let impl_block = find_impl_block(&input);

    // let dependencies = find_dependencies(impl_block);

    let dependencies = &attr.dependencies;
    let dependency_vec = generate_dependency_vec(dependencies);
    //

    let expanded = quote! {
        #[derive(Default)]
        #input

        impl malbox_abi_common::AnalysisPlugin for #name {
            extern "C" fn get_info(&self) -> malbox_abi_common::PluginInfo {
                malbox_abi_common::PluginInfo {
                    name: stringify!(#name).into(),
                    version: env!("CARGO_PKG_VERSION").into(),
                    _type: malbox_abi_common::PluginType::#plugin_type,
                    dependencies: #dependency_vec
                }
            }

            extern "C" fn analyze(&self) -> stabby::result::Result<malbox_abi_common::AnalysisResult, stabby::string::String> {
                self._analyze()
            }
        }

        impl<StabbyTransitiveDerefN> malbox_abi_common::AnalysisPluginDyn<StabbyTransitiveDerefN> for #name {
            extern "C" fn get_info(&self) -> malbox_abi_common::PluginInfo {
                <Self as malbox_abi_common::AnalysisPlugin>::get_info(self)
            }

            extern "C" fn analyze(&self) -> stabby::result::Result<malbox_abi_common::AnalysisResult, stabby::string::String> {
                <Self as malbox_abi_common::AnalysisPlugin>::analyze(self)
            }
        }

        #[stabby::export]
        pub extern "C" fn init_plugin() -> stabby::result::Result<malbox_abi_common::Plugin, stabby::string::String> {
            stabby::result::Result::Ok(stabby::boxed::Box::new(#name::default()).into())
        }
    };

    TokenStream::from(expanded)
}

fn find_impl_block(derive_input: &DeriveInput) -> Option<&ItemImpl> {
    dbg!("{:#?}", derive_input);
    None
}
fn find_dependencies(impl_block: Option<&ItemImpl>) -> HashSet<String> {
    todo!()
}

fn generate_dependency_vec(dependencies: &Punctuated<LitStr, Token![,]>) -> TokenStream2 {
    if dependencies.is_empty() {
        quote! { stabby::vec::Vec::new() }
    } else {
        let deps = dependencies.iter().map(|dep| {
            quote! {
                malbox_abi_common::PluginDependency {
                    name: #dep.into(),
                    version_requirement: stabby::string::String::from("*"),
                }
            }
        });
        quote! {
            stabby::vec::Vec::from_iter(vec![#(#deps),*])
        }
    }
}

// note: maybe we could create a better macro for analysis, this seems kind of redundant since
// you'd still need to write down the function signature
#[proc_macro_attribute]
pub fn analysis(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::ItemFn);
    let name = &input.sig.ident;
    let expanded = quote! {
        #input

        fn _analyze(&self) -> stabby::result::Result<malbox_abi_common::AnalysisResult, stabby::string::String> {
            self.#name()
        }
    };
    TokenStream::from(expanded)
}
