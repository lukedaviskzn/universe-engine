extern crate proc_macro;

use preproc_core::MapEntry;
use proc_macro2::Literal;
use syn::{parse_macro_input, LitStr};

#[proc_macro]
pub fn preprocess(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as LitStr).value();

    let (output, map) = preproc_core::preprocess(&input).expect(&format!("failed to preprocess file '{input}'"));
    
    let source_map_entries = map.0.into_iter().map(|MapEntry { filename, source_start, dest_start, length }| {
        let filename = Literal::string(filename.to_str().expect("failed to parse OsStr to str"));
        let source_start = Literal::usize_unsuffixed(source_start);
        let dest_start = Literal::usize_unsuffixed(dest_start);
        let length = Literal::usize_unsuffixed(length);
        quote::quote! {
            ::wgsl_preprocessor::MapEntry {
                filename: ::std::ffi::OsStr::new(#filename).into(),
                source_start: #source_start,
                dest_start: #dest_start,
                length: #length,
            }
        }
    });
    
    quote::quote! {
        (String::from(#output), ::wgsl_preprocessor::SourceMap(vec![#(#source_map_entries,)*]))
    }.into()
}
