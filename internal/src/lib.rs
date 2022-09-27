use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::__private::{Span, TokenStream2};
use syn::{
    parse_macro_input, Data, DeriveInput, GenericArgument, Ident, Path, PathArguments, PathSegment,
    Type,
};

#[derive(FromDeriveInput)]
#[darling(attributes(request))]
struct DeriveRequestOpts {
    executor: Option<darling::util::PathList>,
}

#[proc_macro_derive(Request, attributes(request))]
pub fn derive_request(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let DeriveInput {
        ident,
        generics,
        data,
        ..
    } = &derive_input;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let request_opts: DeriveRequestOpts =
        DeriveRequestOpts::from_derive_input(&derive_input).unwrap();
    let executor_fields = request_opts.executor.unwrap_or_default();

    let mut impl_executor = vec![];

    if let Data::Struct(data_struct) = data {
        for field in data_struct.fields.iter() {
            if let Some(ident) = &field.ident {
                for path in executor_fields.iter() {
                    if path.is_ident(ident) {
                        let ty = if let Type::Path(ty) = field.clone().ty {
                            ty
                        } else {
                            panic!("shouldn't happen")
                        };
                        impl_executor.push(derive_request_check(quote! { self.#ident }, &ty.path));
                        continue;
                    }
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
    };

    let expanded = quote! {
        impl #impl_generics crate::Request for #ident #ty_generics # where_clause {
            fn __set_executor(&mut self, executor: std::sync::Arc<crate::Executor>) {
                #(#impl_executor)*
            }
        }
    };
    expanded.into()
}

fn derive_request_check(set_path: TokenStream2, path: &Path) -> TokenStream2 {
    let segment = path.segments.last().unwrap();

    let _deep_set_path = set_path.to_string();
    let deep_set_path = _deep_set_path.split('.').last().unwrap();

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
            #set_path.__set_executor(executor.clone());
        }
    }
}

fn segment_types(segment: &PathSegment) -> Vec<Path> {
    let args = if let PathArguments::AngleBracketed(args) = &segment.arguments {
        &args.args
    } else {
        panic!("shouldn't happen")
    };
    args.iter()
        .map(|a| {
            if let GenericArgument::Type(t) = a {
                t
            } else {
                panic!("shouldn't happen")
            }
        })
        .map(|t| {
            if let Type::Path(ty) = t {
                ty.path.clone()
            } else {
                panic!("shouldn't happen")
            }
        })
        .collect()
}
