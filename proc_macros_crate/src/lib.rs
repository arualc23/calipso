extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::{TokenStream};

#[proc_macro_derive(ArrayVariants)]
pub fn derive_array_variants(input: TokenStream) -> TokenStream {
    let syn_item: syn::DeriveInput = syn::parse(input).unwrap();

    let variants = match syn_item.data {
    syn::Data::Enum(enum_item) => {
        enum_item
            .variants
            .into_iter()
            .map(|v| v.ident)
    }
    _ => unimplemented!("ArrayVariants only works on enums"),
    };


    let enum_name = syn_item.ident;
    let count = variants.len();

    quote! {
        impl #enum_name {
            pub const VARIANTS: [#enum_name; #count] = 
                [ #(#enum_name::#variants),* ];
        }
    }.into()
    
}