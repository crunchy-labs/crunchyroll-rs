use darling::{FromDeriveInput, FromMeta};
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{Data, DeriveInput, parse_macro_input};
use syn::__private::TokenStream2;

#[proc_macro_derive(Request)]
pub fn derive_request(input: TokenStream) -> TokenStream {
    let DeriveInput {ident, generics, data, ..} = parse_macro_input!(input as DeriveInput);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let data = match data {
        Data::Struct(data_struct) => data_struct,
        _ => panic!("must only be applied to structs")
    };

    let mut executor = TokenStream2::new();

    for field in data.fields {
        if let Some(name_ident) = field.ident {
            if field.ty.into_token_stream().to_string().replace(' ', "").ends_with("Arc<Executor>") {
                if !executor.is_empty() {
                    panic!("could not determine correct arc executor")
                }

                executor = quote! {
                    fn __set_executor(&mut self, executor: Arc<Executor>) {
                        self.#name_ident = executor
                    }

                    fn __get_executor(&self) -> Option<Arc<Executor>> {
                        Some(self.#name_ident.clone())
                    }
                }
            }
        }
    }

    let expanded = quote! {
        impl #impl_generics crate::Request for #ident #ty_generics # where_clause {
            #executor
        }
    };
    expanded.into()
}

#[proc_macro_derive(Available)]
pub fn derive_available(input: TokenStream) -> TokenStream {
    let DeriveInput {ident, generics, data, ..} = parse_macro_input!(input as DeriveInput);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let data = match data {
        Data::Struct(data_struct) => data_struct,
        _ => panic!("must only be applied to structs")
    };

    let mut options = vec![quote!{self.__get_executor().unwrap().details.premium}];

    for field in data.fields {
        if let Some(name_ident) = field.ident {
            match name_ident.to_string().as_str() {
                "is_premium_only" => options.push(quote!{!self.is_premium_only}),
                "channel_id" => options.push(quote!{self.channel_id.is_empty()}),
                _ => ()
            }
        }
    }

    let expanded = quote! {
        impl #impl_generics crate::common::Available for #ident #ty_generics #where_clause {
            fn available(&self) -> bool {
                #(#options)||*
            }
        }
    };
    expanded.into()
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(from_id))]
struct DeriveFromIdOpts {
    multiple: Option<darling::util::PathList>
}

/// Derives the [`crunchyroll_rs::common::FromId`] crate.
///
/// It takes a `#[from_id(...)]` helper which currently only has the `multiple` sub-field
/// (`#[from_id(multiple(...))]`. The sub-field can take multiple [`syn::Path`]s as arguments
/// which must be library structs, and for every given argument, a
/// `from_`_`argument_struct_name`_`_id` method in the deriven struct is implemented as well as a
/// _`deriven_struct_name`_`s` method in the struct passed as current argument.
///
/// # Examples
///
/// ```
/// struct Season;
///
/// #[derive(FromId)]
/// #[from_id(multiple(Season))]
/// struct Episode;
/// ```
///
/// This would lead to the following generated implementation:
///
/// ```
/// struct Season;
///
/// struct Episode;
///
/// // generated from `#[derive(FromId)]`
/// #[async_trait::async_trait]
/// impl crate::common::FromId for Episode {
///     async fn from_id(crunchy: &Crunchyroll, id: String) -> Result<Self, CrunchyrollError> {
///         /* ... */
///     }
/// }
///
/// // generated from `#[from_id(multiple(Season))]`
/// impl Episode {
///     pub async fn from_season_id(crunchy: &Crunchyroll, season_id: String) -> Result<BulkResult<Self>, CrunchyrollError> {
///         /* ... */
///     }
/// }
///
/// // generated from `#[from_id(multiple(Season))]`
/// impl Season {
///     pub async fn episodes(&self) -> Result<BulkResult<Self>, CrunchyrollError> {
///         /* ... */
///     }
/// }
/// ```
///
#[proc_macro_derive(FromId, attributes(from_id))]
pub fn derive_from_id(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let DeriveInput {ident, generics, ..} = &derive_input;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // convert the struct name from CamelCase to snake_case
    let mut _path = String::new();
    for (i, char) in ident.to_string().chars().enumerate() {
        if char.is_ascii_uppercase() && i != 0 {
            _path.push('_');
        }
        _path.push(char.to_ascii_lowercase());
    }
    if !_path.ends_with('s') {
        _path.push('s');
    }
    let ident_path = syn::Ident::from_string(_path.as_str()).unwrap();

    let mut opt_impls = vec![];
    let mut other_impls = vec![];

    let from_id_opts: DeriveFromIdOpts = DeriveFromIdOpts::from_derive_input(&derive_input).unwrap();
    // check if the `#[from_id(multiple(...))]` attribute is populated
    if let Some(multiple) = from_id_opts.multiple {
        // iterate over every elemenet in the `#[from_id(multiple(...))]` attribute and create a
        for opt_path in multiple.iter() {
            let name = opt_path.segments.last().unwrap().ident.to_string().to_ascii_lowercase();
            let name_id = syn::Ident::from_string(format!("{}_id", &name).as_str()).unwrap();
            let from_name_id = syn::Ident::from_string(format!("from_{}_id", &name).as_str()).unwrap();

            let opt_impl_desc = format!("Return all {} by their parent {} id.", ident_path, name);
            opt_impls.push(quote! {
                #[doc = #opt_impl_desc]
                pub async fn #from_name_id(crunchy: &crate::crunchyroll::Crunchyroll, #name_id: String) -> crate::Result<crate::common::BulkResult<#ident>> {
                    let endpoint = format!(
                        "https://beta-api.crunchyroll.com/cms/v2/{}/{}",
                        crunchy.executor.details.bucket, stringify!(#ident_path)
                    );
                    let builder = crunchy
                        .executor
                        .client
                        .get(endpoint.clone())
                        .query(&[(stringify!(#name_id), #name_id)])
                        .query(&crunchy.executor.media_query());

                    crunchy.executor.request(builder).await
                }
            });
            let other_impl_desc = format!("Return all {} {}.", name, ident_path);
            other_impls.push(quote! {
                impl #opt_path {
                    #[doc = #other_impl_desc]
                    pub async fn #ident_path(&self) -> crate::Result<crate::common::BulkResult<#ident>> {
                        #ident::#from_name_id(&crate::crunchyroll::Crunchyroll { executor: self.__get_executor().unwrap() }, self.id.clone()).await
                    }
                }
            });
        }
    }

    let expanded = quote! {
        #[async_trait::async_trait]
        impl #impl_generics crate::common::FromId for #ident #ty_generics #where_clause {
            async fn from_id(crunchy: &crate::crunchyroll::Crunchyroll, id: String) -> crate::Result<Self> {
                let endpoint = format!(
                    "https://beta-api.crunchyroll.com/cms/v2/{}/{}/{}",
                    crunchy.executor.details.bucket, stringify!(#ident_path), id
                );
                let builder = crunchy
                    .executor
                    .client
                    .get(endpoint)
                    .query(&crunchy.executor.media_query());

                crunchy.executor.request(builder).await
            }
        }

        impl #ident {
            #(#opt_impls)*
        }

        #(#other_impls)*
    };
    expanded.into()
}

#[proc_macro_derive(Playback)]
pub fn derive_playback(input: TokenStream) -> TokenStream {
    let DeriveInput {ident, generics, data, ..} = parse_macro_input!(input as DeriveInput);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let data = match data {
        Data::Struct(data_struct) => data_struct,
        _ => panic!("must only be applied to structs")
    };

    let mut playback_id = TokenStream2::new();

    for field in data.fields {
        if let Some(name_ident) = field.ident {
            // only if the `playback_id` id field exists, playback can be deriven
            if name_ident == "playback_id" {
                // check if the `playback_id` field has `String` as type or `Option<String>` which
                // must be handled separately
                if (&field.ty).into_token_stream().to_string().replace(' ', "").ends_with("String") {
                    playback_id = quote! {
                        async fn playback(&self) -> crate::Result<crate::media::PlaybackStream> {
                            self.executor
                                .request(self.executor.client.get(&self.playback_id))
                                .await
                        }
                    }
                } else if field.ty.into_token_stream().to_string().replace(' ', "").ends_with("Option<String>") {
                    playback_id = quote! {
                        async fn playback(&self) -> crate::Result<crate::media::PlaybackStream> {
                            if let Some(playback_id) = &self.playback_id {
                                self.executor
                                    .request(self.executor.client.get(playback_id))
                                    .await
                            } else {
                                Err(crate::error::CrunchyrollError::Request(crate::error::CrunchyrollErrorContext::new(
                                    "no playback id available".into(),
                                )))
                            }
                        }
                    }
                }
            }
        }
    }

    let expanded = quote! {
        #[async_trait::async_trait]
        impl #impl_generics crate::media::Playback for #ident #ty_generics # where_clause {
            #playback_id
        }
    };
    expanded.into()
}
