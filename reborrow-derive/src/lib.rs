use quote::quote;
use syn::{DeriveInput, GenericParam, Lifetime, LifetimeDef};

#[proc_macro_derive(ReborrowCopyTraits)]
pub fn derive_reborrow_copy(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let reborrowed_lifetime = &LifetimeDef::new(Lifetime::new(
        "'__reborrow_lifetime",
        proc_macro2::Span::call_site(),
    ));

    let mut target_ty_generics = input.generics.clone();
    for lt in target_ty_generics.lifetimes_mut() {
        *lt = reborrowed_lifetime.clone();
    }
    let target_ty_generics = target_ty_generics.split_for_impl().1;
    let mut impl_generics = input.generics.clone();
    impl_generics
        .params
        .insert(0, GenericParam::Lifetime(reborrowed_lifetime.clone()));
    let impl_generics = impl_generics.split_for_impl().0;

    let (orig_impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #orig_impl_generics ::core::marker::Copy for #name #ty_generics
            #where_clause {}

        impl #orig_impl_generics ::core::clone::Clone for #name #ty_generics
            #where_clause
        {
            #[inline]
            fn clone(&self) -> Self {
                *self
            }
        }

        impl #orig_impl_generics ::reborrow::IntoConst for #name #ty_generics
            #where_clause
        {
            type Target = #name #ty_generics;

            #[inline]
            fn into_const(self) -> <Self as ::reborrow::IntoConst>::Target {
                self
            }
        }

        impl #impl_generics ::reborrow::ReborrowMut<'__reborrow_lifetime> for #name #ty_generics
            #where_clause
        {
            type Target = #name #target_ty_generics;

            #[inline]
            fn rb_mut(&'__reborrow_lifetime mut self) -> <Self as ::reborrow::ReborrowMut>::Target {
                *self
            }
        }

        impl #impl_generics ::reborrow::Reborrow<'__reborrow_lifetime> for #name #ty_generics
            #where_clause
        {
            type Target = #name #target_ty_generics;

            #[inline]
            fn rb(&'__reborrow_lifetime self) -> <Self as ::reborrow::Reborrow>::Target {
                *self
            }
        }

        impl #impl_generics ::reborrow::AsGeneralizedMut<
            '__reborrow_lifetime,
            <Self as ::reborrow::ReborrowMut<'__reborrow_lifetime>>::Target,
        > for #name #ty_generics
            #where_clause
        {
            #[inline]
            fn as_generalized_mut(&'__reborrow_lifetime mut self) -> <Self as ::reborrow::ReborrowMut<'__reborrow_lifetime>>::Target {
                *self
            }
        }

        impl #impl_generics ::reborrow::AsGeneralizedRef<
            '__reborrow_lifetime,
            <Self as ::reborrow::Reborrow<'__reborrow_lifetime>>::Target,
        > for #name #ty_generics
            #where_clause
        {
            #[inline]
            fn as_generalized_ref(&'__reborrow_lifetime self) -> <Self as ::reborrow::Reborrow<'__reborrow_lifetime>>::Target {
                *self
            }
        }
    };

    expanded.into()
}

#[proc_macro_derive(ReborrowTraits, attributes(reborrow, Const))]
pub fn derive_reborrow(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    let const_name = input
        .attrs
        .iter()
        .find(|&attr| {
            let segments = &attr.path.segments;
            if let Some(syn::PathSegment {
                ident,
                arguments: syn::PathArguments::None,
            }) = segments.first()
            {
                ident.to_string() == "Const"
            } else {
                false
            }
        })
        .unwrap_or_else(|| panic!("Const reborrowed type must be specified."));

    let const_name = const_name.tokens.clone();
    let const_name = *syn::parse2::<syn::TypeParen>(const_name).unwrap().elem;

    let name = &input.ident;

    let reborrowed_lifetime = &LifetimeDef::new(Lifetime::new(
        "'__reborrow_lifetime",
        proc_macro2::Span::call_site(),
    ));

    let mut target_ty_generics = input.generics.clone();
    for lt in target_ty_generics.lifetimes_mut() {
        *lt = reborrowed_lifetime.clone();
    }
    let target_ty_generics = target_ty_generics.split_for_impl().1;

    let mut impl_generics = input.generics.clone();
    impl_generics
        .params
        .insert(0, GenericParam::Lifetime(reborrowed_lifetime.clone()));
    let impl_generics = impl_generics.split_for_impl().0;

    let (orig_impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let (rb_mut, rb, into_const) = {
        let data = input.data;

        match data {
            syn::Data::Struct(s) => match s.fields {
                syn::Fields::Named(f) => {
                    let names: Vec<_> = f.named.iter().map(|f| &f.ident).collect();
                    let (f0, f1, f2) = unzip3(
                        f.named
                            .iter()
                            .enumerate()
                            .map(|(i, f)| reborrow_exprs(i, f.clone())),
                    );

                    (
                        quote! { #name:: #target_ty_generics { #(#names: #f0,)* } },
                        quote! { #const_name:: #target_ty_generics { #(#names: #f1,)* } },
                        quote! { #const_name:: #ty_generics { #(#names: #f2,)* } },
                    )
                }
                syn::Fields::Unnamed(f) => {
                    let (f0, f1, f2) = unzip3(
                        f.unnamed
                            .iter()
                            .enumerate()
                            .map(|(i, f)| reborrow_exprs(i, f.clone())),
                    );

                    (
                        quote! { #name:: #target_ty_generics ( #(#f0,)* ) },
                        quote! { #const_name:: #target_ty_generics ( #(#f1,)* ) },
                        quote! { #const_name:: #ty_generics ( #(#f2,)* ) },
                    )
                }
                syn::Fields::Unit => (
                    quote! { #name:: #target_ty_generics },
                    quote! { #const_name:: #target_ty_generics },
                    quote! { #const_name:: #target_ty_generics },
                ),
            },
            syn::Data::Enum(_) => panic!("reborrow-derive does not support enums."),
            syn::Data::Union(_) => panic!("reborrow-derive does not support unions."),
        }
    };

    let expanded = quote! {
        impl #orig_impl_generics ::reborrow::IntoConst for #name #ty_generics
            #where_clause
        {
            type Target = #const_name #ty_generics;

            #[inline]
            fn into_const(self) -> <Self as ::reborrow::IntoConst>::Target {
                #into_const
            }
        }

        impl #impl_generics ::reborrow::ReborrowMut<'__reborrow_lifetime> for #name #ty_generics
            #where_clause
        {
            type Target = #name #target_ty_generics;

            #[inline]
            fn rb_mut(&'__reborrow_lifetime mut self) -> <Self as ::reborrow::ReborrowMut>::Target {
                #rb_mut
            }
        }

        impl #impl_generics ::reborrow::Reborrow<'__reborrow_lifetime> for #name #ty_generics
            #where_clause
        {
            type Target = #const_name #target_ty_generics;

            #[inline]
            fn rb(&'__reborrow_lifetime self) -> <Self as ::reborrow::Reborrow>::Target {
                #rb
            }
        }

        impl #impl_generics ::reborrow::AsGeneralizedMut<
            '__reborrow_lifetime,
            <Self as ::reborrow::ReborrowMut<'__reborrow_lifetime>>::Target,
        > for #name #ty_generics
            #where_clause
        {
            #[inline]
            fn as_generalized_mut(&'__reborrow_lifetime mut self) -> <Self as ::reborrow::ReborrowMut<'__reborrow_lifetime>>::Target {
                <Self as ::reborrow::ReborrowMut>::rb_mut(self)
            }
        }

        impl #impl_generics ::reborrow::AsGeneralizedRef<
            '__reborrow_lifetime,
            <Self as ::reborrow::Reborrow<'__reborrow_lifetime>>::Target,
        > for #name #ty_generics
            #where_clause
        {
            #[inline]
            fn as_generalized_ref(&'__reborrow_lifetime self) -> <Self as ::reborrow::Reborrow<'__reborrow_lifetime>>::Target {
                <Self as ::reborrow::Reborrow>::rb(self)
            }
        }
    };

    expanded.into()
}

fn unzip3<A, B, C, I: Iterator<Item = (A, B, C)>>(iter: I) -> (Vec<A>, Vec<B>, Vec<C>) {
    let mut v0 = Vec::new();
    let mut v1 = Vec::new();
    let mut v2 = Vec::new();
    for (a, b, c) in iter {
        v0.push(a);
        v1.push(b);
        v2.push(c);
    }
    (v0, v1, v2)
}

fn reborrow_exprs(
    idx: usize,
    f: syn::Field,
) -> (
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
) {
    let is_reborrowable = f
        .attrs
        .iter()
        .find(|&attr| {
            let segments = &attr.path.segments;
            if let Some(syn::PathSegment {
                ident,
                arguments: syn::PathArguments::None,
            }) = segments.first()
            {
                ident.to_string() == "reborrow"
            } else {
                false
            }
        })
        .is_some();

    let idx = syn::Index::from(idx);

    let expr = f
        .ident
        .map(|ident| quote! { self.#ident })
        .unwrap_or(quote! { self.#idx });

    if !is_reborrowable {
        (quote! {#expr}, quote! {#expr}, quote! {#expr})
    } else {
        let ty = f.ty;
        (
            quote! { <#ty as ::reborrow::ReborrowMut>::rb_mut(&mut #expr) },
            quote! { <#ty as ::reborrow::Reborrow>::rb(&#expr) },
            quote! { <#ty as ::reborrow::IntoConst>::into_const(#expr) },
        )
    }
}
