use std::{fs::OpenOptions, sync::OnceLock};

use convert_case::ccase;
use proc_macro::TokenStream;
use proc_macro2::{Punct, TokenTree};
use quote::{ToTokens, TokenStreamExt, quote};
use syn::{
    ExprMatch, Generics, Ident, ImplItemType, ItemEnum, ItemImpl, ItemTrait, Path, Token,
    TraitBound, TraitItem, TraitItemFn, Type, TypeParamBound, TypePath, Variant,
    parse::{self, Parse, ParseBuffer, ParseStream},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    token::{Comma, Plus, Pub, Trait},
};

use std::io::Write;

fn debugfile(path: impl ToString) -> anyhow::Result<std::fs::File> {
    Ok(OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path.to_string())?)
}

struct IoTypes(Punctuated<Type, Comma>);
impl Parse for IoTypes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(Punctuated::<Type, Comma>::parse_terminated(input)?))
    }
}

struct CommaSeparatedTT(Punctuated<TokenTree, Comma>);
impl Parse for CommaSeparatedTT {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(Punctuated::<TokenTree, Comma>::parse_terminated(
            input,
        )?))
    }
}

#[derive(Clone, Debug)]
struct GnComponents {
    ty: Ident,
    variant: Ident,
    full_path: TypePath,
}
fn mkgn_derive_names(ident: &Ident, types: &[Type]) -> Vec<GnComponents> {
    let mut names = Vec::new();
    let enum_ident: Ident = syn::parse_str(&format!("{}Type", ident.to_string())).unwrap();
    for ty in types {
        let mut ty_str = ty.to_token_stream().to_string();
        ty_str.make_ascii_uppercase();
        // panic!("{ty_str} {ident}");
        let ty_str: Ident = syn::parse_str(&ty_str).unwrap();
        let variant_name: TypePath = parse_quote! {#enum_ident :: #ty_str};
        names.push(GnComponents {
            ty: syn::parse_quote! {#ty},
            variant: syn::parse_quote! {#ty_str},
            full_path: variant_name,
        });
    }

    names
}

// const NEWLINE = syn::parse_quote! { \n };
// const NEWLINE: proc_macro2::TokenTree =
//     proc_macro2::TokenTree::Punct(Punct {});

fn mkgn_num_cast() -> proc_macro2::TokenStream {
    syn::parse_quote! { T::_from(*v) }
}
fn mkgn_num_cast_vec() -> proc_macro2::TokenStream {
    syn::parse_quote! { v.iter().map(|v| T::_from(*v)).collect() }
}
fn mkgn_num_len_vec() -> proc_macro2::TokenStream {
    syn::parse_quote! { v.len() }
}
fn mkgn_num_set_idx_vec() -> proc_macro2::TokenStream {
    syn::parse_quote! { v[index] = value.num_cast() }
}
fn mkgn_num_get_idx_vec(ident: &Ident) -> proc_macro2::TokenStream {
    let fn_name: Ident =
        syn::parse_str(&format!("as_{}", ccase!(snake, ident.to_string()))).unwrap();
    syn::parse_quote! { v.get(index)?.#fn_name() }
}

fn mkgn_build_ty_match(
    ident: &Ident,
    // type_enum_ident: &Ident,
    cmp: &[GnComponents],
) -> ExprMatch {
    let mut mat: ExprMatch = syn::parse_quote! {match &self {}};
    for cmp in cmp {
        let variant_fullpath = &cmp.full_path;
        let variant = &cmp.variant;
        // let ty = &cmp.ty;
        let mut arm: syn::Arm = syn::parse_quote! { #ident :: #variant (_) => {#variant_fullpath} };
        mat.arms.push(arm);
    }
    mat
}

fn mkgn_build_match_block(
    ident: &Ident,
    arm: &proc_macro2::TokenStream,
    // type_enum_ident: &Ident,
    cmp: &[GnComponents],
) -> ExprMatch {
    let mut mat: ExprMatch = syn::parse_quote! {match self {}};
    for cmp in cmp {
        let variant_fullpath = &cmp.full_path;
        let variant = &cmp.variant;
        // let ty = &cmp.ty;
        let mut arm: syn::Arm = syn::parse_quote! { #ident :: #variant (v) => {#arm} };
        mat.arms.push(arm);
    }
    mat
}

fn mkgn_build_type_enum(params: &MkgnParams) -> ItemEnum {
    // let enum_ident: Ident = syn::parse_str(&format!("{}Type", ident.to_string())).unwrap();
    let enum_ident = &params.ty_name;
    let cmp = &params.cmp;
    let mut en: ItemEnum = syn::parse_quote! {pub enum #enum_ident {} };
    for cmp in cmp {
        en.variants.push(Variant {
            attrs: Vec::new(),
            ident: cmp.variant.clone(),
            fields: syn::Fields::Unit,
            discriminant: None,
        });
    }

    en
}
fn mkgn_build_val_enum(
    params: &MkgnParams,
    // ident: &Ident,
    // type_enum_ident: &Ident,
    // trait_ident: &Ident,
    // cmp: &[GnComponents],
) -> (ItemEnum, ItemImpl) {
    // let enum_ident: Ident = syn::parse_str(&format!("{}", ident.to_string())).unwrap();
    // let enum_ident = ident.clone();
    let enum_ident = &params.name;
    let type_enum_ident = &params.ty_name;
    let trait_ident = &params.tr_name;
    let cmp = &params.cmp;
    // let derive = &params.derive_directive;
    let mut en: ItemEnum = syn::parse_quote! {pub enum #enum_ident {} };
    let match_block = mkgn_build_ty_match(&enum_ident, &cmp);
    let cast_match_block = mkgn_build_match_block(&enum_ident, &mkgn_num_cast(), &cmp);
    let mut impls: ItemImpl = syn::parse_quote!(impl #enum_ident {
        pub fn num_type(&self) -> #type_enum_ident { #match_block }
        pub fn num_cast<T: #trait_ident>(&self) -> T {#cast_match_block}
    });
    for cmp in cmp {
        let ty = &cmp.ty;
        en.variants.push(Variant {
            attrs: Vec::new(),
            ident: cmp.variant.clone(),
            fields: syn::Fields::Unnamed(syn::parse_quote! {
                (#ty)
            }),
            discriminant: None,
        });
    }

    (en, impls)
}
fn mkgn_build_vec_enum(
    params: &MkgnParams,
    // ident: &Ident,
    // type_enum_ident: &Ident,
    // trait_ident: &Ident,
    // cmp: &[GnComponents],
) -> (ItemEnum, ItemImpl) {
    let ident = &params.name;
    let enum_ident = &params.vec_name;
    let type_enum_ident = &params.ty_name;
    let trait_ident = &params.tr_name;
    let cmp = &params.cmp;
    // let derive = &params.derive_directive;
    // let enum_ident: Ident = syn::parse_str(&format!("{}Vec", ident.to_string())).unwrap();
    let mut en: ItemEnum = syn::parse_quote! {pub enum #enum_ident {} };
    let match_block = mkgn_build_ty_match(&enum_ident, &cmp);
    let cast_match_block = mkgn_build_match_block(&enum_ident, &mkgn_num_cast_vec(), &cmp);
    let len_match_block = mkgn_build_match_block(&enum_ident, &mkgn_num_len_vec(), &cmp);
    let set_match_block = mkgn_build_match_block(&enum_ident, &mkgn_num_set_idx_vec(), &cmp);
    let get_match_block = mkgn_build_match_block(&enum_ident, &mkgn_num_get_idx_vec(&ident), &cmp);
    let mut impls: ItemImpl = syn::parse_quote!(impl #enum_ident {
        pub fn num_type(&self) -> #type_enum_ident { #match_block }
        pub fn num_cast<T: #trait_ident>(&self) -> Vec<T> {#cast_match_block}
        pub fn len(&self) -> usize {#len_match_block}
        pub fn get(&self, index: usize) -> Option<#ident> {Some(#get_match_block)}
        /// Panics if index OOB
        pub fn set(&mut self, index: usize, value: #ident) {#set_match_block}
    });
    for cmp in cmp {
        let ty = &cmp.ty;
        en.variants.push(Variant {
            attrs: Vec::new(),
            ident: cmp.variant.clone(),
            fields: syn::Fields::Unnamed(syn::parse_quote! {
                (Vec<#ty>)
            }),
            discriminant: None,
        });
    }

    (en, impls)
}
fn mkgn_build_trait(
    params: &MkgnParams,
    // ident: &Ident, type_enum_ident: &Ident, cmp: &[GnComponents]
) -> ItemTrait {
    // let trait_ident: Ident = syn::parse_str(&format!("{}Trait", ident.to_string())).unwrap();
    let ident = &params.name;
    let trait_ident = &params.tr_name;
    let vec_enum_ident = &params.vec_name;
    let type_enum_ident = &params.ty_name;
    let conv_fn_ident: Ident =
        syn::parse_str(&format!("as_{}", ccase!(snake, ident.to_string()))).unwrap();
    let conv_vec_fn_ident: Ident = syn::parse_str(&format!("{conv_fn_ident}_vec")).unwrap();

    let mut supertraits = &params.trait_bounds;
    // let vec_enum_ident: Ident = syn::parse_str(&format!("{}Vec", ident.to_string())).unwrap();
    // let mut supertraits = Punctuated::<TypeParamBound, Token![+]>::new();
    // for cmp in cmp {
    // let ty = cmp.ty.clone();
    // supertraits.push(syn::parse_quote! {Num});
    // }
    let mut tr: ItemTrait = syn::parse_quote! {pub trait #trait_ident : #supertraits {}};

    /* fn */
    let mut item = syn::parse_quote! {fn num_type() -> #type_enum_ident; };
    tr.items.push(item);
    tr.items
        .push(syn::parse_quote! {fn #conv_fn_ident (&self) -> #ident; });
    tr.items
        .push(syn::parse_quote! {fn #conv_vec_fn_ident (vec: Vec<Self>) -> #vec_enum_ident;});

    /* */

    tr
}
fn mkgn_build_trait_impls(
    params: &MkgnParams,
    // ident: &Ident,
    // trait_ident: &Ident,
    // type_enum_ident: &Ident,
    // cmp: &[GnComponents],
) -> Vec<ItemImpl> {
    let ident = &params.name;
    let trait_ident = &params.tr_name;
    let vec_enum_ident = &params.vec_name;
    let type_enum_ident = &params.ty_name;
    let cmp = &params.cmp;
    let conv_fn_ident: Ident =
        syn::parse_str(&format!("as_{}", ccase!(snake, ident.to_string()))).unwrap();
    let conv_vec_fn_ident: Ident = syn::parse_str(&format!("{conv_fn_ident}_vec")).unwrap();

    let vec_enum_ident: Ident = syn::parse_str(&format!("{}Vec", ident.to_string())).unwrap();

    let mut impls = Vec::new();
    for cmp in cmp {
        let ty = &cmp.ty;
        let mut impl_block: ItemImpl = syn::parse_quote! { impl #trait_ident for #ty {}};
        let variant = &cmp.variant;
        let variant_path = cmp.full_path.clone();
        // let mut item = syn::parse_quote! {fn num_type() -> #type_enum_ident { #variant_path } };
        impl_block
            .items
            .push(syn::parse_quote! {fn num_type() -> #type_enum_ident { #variant_path } });
        impl_block.items.push(
            syn::parse_quote! {fn #conv_fn_ident (&self) -> #ident { #ident :: #variant (*self) } },
        );
        impl_block
            .items
            .push(syn::parse_quote! {fn #conv_vec_fn_ident (vec: Vec<#ty>) -> #vec_enum_ident { #vec_enum_ident :: #variant (vec) } });
        impls.push(impl_block);
    }
    impls
}

#[derive(Debug)]
struct MkgnParams {
    name: Ident,
    ty_name: Ident,
    tr_name: Ident,
    vec_name: Ident,
    derive_directive: proc_macro2::TokenStream,
    // derive_directive_vec: proc_macro2::TokenStream,
    trait_bounds: proc_macro2::TokenStream,
    cmp: Vec<GnComponents>,
}
impl MkgnParams {
    fn parse_input(item: &TokenStream) -> MkgnParams {
        let tt: CommaSeparatedTT = syn::parse(item.clone()).unwrap();
        let tt: Vec<&TokenTree> = tt.0.iter().collect();
        let ident: Ident = syn::parse2(tt[0].into_token_stream()).unwrap();

        let TokenTree::Group(types) = tt[1] else {
            panic!()
        };
        let types: Vec<Type> = syn::parse2::<IoTypes>(types.stream())
            .unwrap()
            .0
            .iter()
            .cloned()
            .collect();

        let trait_bounds = if let Some(TokenTree::Group(ts)) = tt.get(2) {
            ts.stream()
        } else {
            panic!()
        };
        let derive_directive: proc_macro2::TokenStream = if let Some(ts) = tt.get(3) {
            let derive: proc_macro2::TokenStream =
                syn::parse_str(&format!("#[derive{}]", ts.to_string())).unwrap();
            derive
        } else {
            proc_macro2::TokenStream::new()
        };

        let cmp = mkgn_derive_names(&ident, &types);
        let name = ident.to_string();
        MkgnParams {
            name: ident.clone(),
            ty_name: syn::parse_str(&format!("{name}Type")).unwrap(),
            tr_name: syn::parse_str(&format!("{name}Trait")).unwrap(),
            vec_name: syn::parse_str(&format!("{name}Vec")).unwrap(),
            derive_directive,
            trait_bounds,
            cmp,
        }
    }
}

#[proc_macro]
pub fn make_generic_num(item: TokenStream) -> TokenStream {
    // let mut dbg = debugfile("_numutil_pm_debug_1.txt").unwrap();
    // writeln!(dbg, "{}", item.to_string()).unwrap();
    // let tt: CommaSeparatedTT = syn::parse(item).unwrap();
    // let tt: Vec<&TokenTree> = tt.0.iter().collect();
    // let ident: Ident = syn::parse2(tt[0].into_token_stream()).unwrap();
    // writeln!(dbg, "{ident:#?}").unwrap();
    // let TokenTree::Group(types) = tt[1] else {
    //     panic!()
    // };
    // let types: Vec<Type> = syn::parse2::<IoTypes>(types.stream())
    //     .unwrap()
    //     .0
    //     .iter()
    //     .cloned()
    //     .collect();

    // writeln!(dbg, "kk").unwrap();

    // let names = mkgn_derive_names(&ident, &types);

    let params = MkgnParams::parse_input(&item);

    // let en = mkgn_build_type_enum(&ident, &names);
    // let tr = mkgn_build_trait(&ident, &en.ident, &names);
    // let tr_impls = mkgn_build_trait_impls(&ident, &tr.ident, &en.ident, &names);
    // let (val_en, val_impl) = mkgn_build_val_enum(&ident, &en.ident, &tr.ident, &names);
    // let (vec_en, vec_impl) = mkgn_build_vec_enum(&ident, &en.ident, &tr.ident, &names);

    let en = mkgn_build_type_enum(&params);
    let tr = mkgn_build_trait(&params);
    let tr_impls = mkgn_build_trait_impls(&params);
    let (val_en, val_impl) = mkgn_build_val_enum(&params);
    let (vec_en, vec_impl) = mkgn_build_vec_enum(&params);

    // writeln!(dbg, "\nKVX {params:#?}").unwrap();

    let tr_impls = tr_impls.iter().fold(
        /*String::new()*/ proc_macro2::TokenStream::new(),
        |mut acc, v| {
            v.to_tokens(&mut acc);
            acc
        },
    );

    // writeln!(dbg, "M_UNTIL_H").unwrap();
    let derive = &params.derive_directive;

    let derive_partialeq: proc_macro2::TokenStream =
        syn::parse_str("#[derive(PartialEq, Clone, Copy, Debug)]").unwrap();

    // writeln!(dbg, "M_UNTIL_H2 {}", derive.to_string()).unwrap();
    let ts: proc_macro2::TokenStream = syn::parse_quote! {
        #derive_partialeq
        #en
        #tr
        #tr_impls
        #derive
        #val_en
        #val_impl
        #derive
        #vec_en
        #vec_impl
    };
    // let TokenTree::Group(grp) = tt else { panic!() };
    // writeln!(dbg, "\nKVX {}", ts.to_string()).unwrap();
    // let mut dbg2 = debugfile("_numutil_pm_debug_1.rs").unwrap();
    // writeln!(
    //     dbg2,
    //     "\n{}",
    //     prettyplease::unparse(&syn::parse2::<syn::File>(ts.clone()).unwrap())
    // )
    // .unwrap();
    ts.into()
}

#[proc_macro_attribute]
pub fn primitive_io(attr: TokenStream, item: TokenStream) -> TokenStream {
    // let mut dbg = debugfile("_numutil_pm_debug_1.txt").unwrap();

    let mut tt: syn::ItemTrait = syn::parse(item.clone()).unwrap();
    // let ps: ParseBuffer = parse_macro_input!(attr);
    // let k = ps(Punctuated::<Type, Comma>::parse_terminated).unwrap();
    let types: IoTypes = parse_macro_input!(attr);
    // let types = parse_macro_input!(attr as Punctuated<Ident, Comma>);

    // writeln!(dbg, "{:#?}", tt.to_token_stream().to_string()).unwrap();

    for t in types.0 {
        // let (r_le, r_be, w_le, w_be) = (
        //     syn::parse_str::<Ident>(&format!("read_{}_le", t.to_token_stream().to_string()))
        //         .unwrap(),
        //     syn::parse_str::<Ident>(&format!("read_{}_be", t.to_token_stream().to_string()))
        //         .unwrap(),
        //     syn::parse_str::<Ident>(&format!("write_{}_le", t.to_token_stream().to_string()))
        //         .unwrap(),
        //     syn::parse_str::<Ident>(&format!("write_{}_be", t.to_token_stream().to_string()))
        //         .unwrap(),
        // );
        let (r, w) = (
            syn::parse_str::<Ident>(&format!("read_{}", t.to_token_stream().to_string())).unwrap(),
            syn::parse_str::<Ident>(&format!("write_{}", t.to_token_stream().to_string())).unwrap(),
        );
        // writeln!(dbg, "{:#?}", r_le).unwrap();
        // writeln!(dbg, "{:#?}", t.to_token_stream().to_string()).unwrap();
        // tt.items.push(parse_quote! {
        //     fn #r_le(&mut self, pos: u64) -> anyhow::Result<#t> {
        //         let mut bytes = [0; size_of::<#t>()];
        //         self.read_bytes_into(pos, &mut bytes)?;
        //         Ok(#t::from_le_bytes(bytes))
        //     }
        // });
        // tt.items.push(parse_quote! {
        //     fn #r_be(&mut self, pos: u64) -> anyhow::Result<#t> {
        //         let mut bytes = [0; size_of::<#t>()];
        //         self.read_bytes_into(pos, &mut bytes)?;
        //         Ok(#t::from_be_bytes(bytes))
        //     }
        // });

        // tt.items.push(parse_quote! {
        //     fn #w_le(&mut self, pos: u64, value: #t) -> anyhow::Result<()> {
        //         let mut bytes = value.to_le_bytes();
        //         self.write_bytes(pos, &mut bytes)?;
        //         Ok(())
        //     }
        // });

        // tt.items.push(parse_quote! {
        //     fn #w_be(&mut self, pos: u64, value: #t) -> anyhow::Result<()> {
        //         let mut bytes = value.to_be_bytes();
        //         self.write_bytes(pos, &mut bytes)?;
        //         Ok(())
        //     }
        // });

        tt.items.push(parse_quote! {
            fn #r(&self, pos: u64, endianess: Endianess) -> anyhow::Result<#t> {
                let mut bytes = [0; size_of::<#t>()];
                self.read_bytes_into(pos, &mut bytes)?;
                Ok(match endianess {Endianess::LE => #t::from_le_bytes(bytes), Endianess::BE => #t::from_be_bytes(bytes), })
            }
        });

        tt.items.push(parse_quote! {
            fn #w(&self, pos: u64, value: #t, endianess: Endianess) -> anyhow::Result<()> {
                let mut bytes = match endianess {Endianess::LE => value.to_le_bytes(), Endianess::BE => value.to_be_bytes(), };
                self.write_bytes(pos, &mut bytes)?;
                Ok(())
            }
        });
    }

    // writeln!(dbg, "{}", tt.to_token_stream().to_string()).unwrap();

    // writeln!(dbg, "{:#?}", types.0.to_token_stream().to_string()).unwrap();
    tt.into_token_stream().into()
}
