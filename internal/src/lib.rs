mod util;

use crate::util::IdentList;
use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::__private::{Span, TokenStream2};
use syn::spanned::Spanned;
use syn::{
    Data, DeriveInput, GenericArgument, Ident, Path, PathArguments, PathSegment, Type,
    parse_macro_input,
};

#[derive(FromDeriveInput)]
#[darling(attributes(request))]
struct DeriveRequestOpts {
    executor: Option<IdentList>,
}

#[proc_macro_derive(Request, attributes(request))]
pub fn derive_request(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    if !matches!(derive_input.data, Data::Struct(_)) && !matches!(derive_input.data, Data::Enum(_))
    {
        return syn::Error::new(derive_input.span(), "Only allowed on structs and enums")
            .to_compile_error()
            .into();
    }

    let DeriveInput {
        ident,
        generics,
        data,
        ..
    } = &derive_input;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let request_opts = DeriveRequestOpts::from_derive_input(&derive_input).unwrap();
    let mut executor_fields = request_opts.executor.as_ref().map(|f| f.to_vec());

    let mut impl_executor = vec![];

    match data {
        Data::Struct(data_struct) => {
            for field in data_struct.fields.iter() {
                let Some(ident) = &field.ident else {
                    continue;
                };

                for (i, executor_ident) in executor_fields.iter().flatten().enumerate() {
                    if ident != executor_ident {
                        continue;
                    }

                    let Type::Path(ty) = field.ty.clone() else {
                        unreachable!()
                    };
                    impl_executor.push(derive_request_check(quote! { self.#ident }, &ty.path));

                    executor_fields.as_mut().unwrap().remove(i);
                    break;
                }

                if let Type::Path(ty) = field.clone().ty {
                    let segment = ty.path.segments.last().unwrap();
                    if segment.ident == "Arc" && segment_types(segment)[0].is_ident("Executor") {
                        impl_executor.push(quote! {
                            self.#ident = executor.clone();
                        })
                    }
                }
            }
        }
        Data::Enum(_) if request_opts.executor.is_some() => {
            return syn::Error::new(
                request_opts.executor.unwrap().span(),
                "Executor fields aren't allowed on enums",
            )
            .to_compile_error()
            .into();
        }
        _ => (),
    }

    if let Some(first_field) = executor_fields.iter().flatten().next() {
        return syn::Error::new(
            first_field.span(),
            format!("Executor field not found: {first_field}"),
        )
        .to_compile_error()
        .into();
    }

    let expanded = quote! {
        impl #impl_generics crate::Request for #ident #ty_generics # where_clause {
            async fn __set_executor(&mut self, executor: std::sync::Arc<crate::Executor>) {
                #(#impl_executor)*
            }
        }
    };
    expanded.into()
}

fn derive_request_check(set_path: TokenStream2, path: &Path) -> TokenStream2 {
    let segment = path.segments.last().unwrap();

    let _deep_set_path = set_path.to_string();
    let deep_set_path = _deep_set_path.split('.').next_back().unwrap();

    if segment.ident == "Option" {
        let options_set_path = Ident::new(
            format!("{}{}", "option_", deep_set_path).as_str(),
            Span::call_site(),
        );
        let ty = &segment_types(segment)[0];
        let check = derive_request_check(options_set_path.to_token_stream(), ty);
        quote! {
            if let Some(#options_set_path) = &mut #set_path {
                #check
            }
        }
    } else if segment.ident == "Vec" {
        let vec_set_path = Ident::new(
            format!("{}{}", "vec_", deep_set_path).as_str(),
            Span::call_site(),
        );
        let ty = &segment_types(segment)[0];
        let check = derive_request_check(vec_set_path.to_token_stream(), ty);
        quote! {
            for #vec_set_path in #set_path.iter_mut() {
                #check
            }
        }
    } else if segment.ident == "HashMap" {
        let hash_map_set_path = Ident::new(
            format!("{}{}", "hash_map_", deep_set_path).as_str(),
            Span::call_site(),
        );
        let ty = &segment_types(segment)[1];
        let check = derive_request_check(hash_map_set_path.to_token_stream(), ty);
        quote! {
            for #hash_map_set_path in #set_path.values_mut() {
                #check
            }
        }
    } else {
        quote! {
            #set_path.__set_executor(executor.clone()).await;
        }
    }
}

fn segment_types(segment: &PathSegment) -> Vec<Path> {
    let args = if let PathArguments::AngleBracketed(args) = &segment.arguments {
        &args.args
    } else {
        unreachable!()
    };
    args.iter()
        .map(|a| {
            if let GenericArgument::Type(t) = a {
                t
            } else {
                unreachable!()
            }
        })
        .map(|t| {
            if let Type::Path(ty) = t {
                ty.path.clone()
            } else {
                unreachable!()
            }
        })
        .collect()
}
