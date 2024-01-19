use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_error::abort;
use proc_macro_error::proc_macro_error;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::*;
use template_quote::quote;

fn to_tstr(krate: &Path, s: &str, span: Span) -> TypeTuple {
    let mut elems = Punctuated::new();
    for c in s.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '_' => {
                let ident = Ident::new(&format!("_{}", c), span);
                elems.push(parse_quote!(#krate::_tstr::#ident));
            }
            _ => abort!(span, "Bad char '{}'", c),
        }
    }
    TypeTuple {
        paren_token: Default::default(),
        elems,
    }
}

fn emit_offsets(krate: &Path, ident: &Ident, fields: &[&Field]) -> TokenStream2 {
    quote! {
        (
            [
            #(for (n, field) in fields.iter().enumerate()) {
                ptr_cast(#krate::addr_of_enum::addr_of_enum!(
                        ptr, #{&ident},
                        #(if let Some(id) = &field.ident) { #id }
                        #(else) { #{LitInt::new(&format!("{}", n), field.span())} }
                )) - ptr_cast(ptr),
            }
            ] as [::core::primitive::usize; #{fields.len()}]
        )
    }
}

fn sort_fields(fields: &Fields) -> Vec<&Field> {
    let mut out: Vec<_> = fields.iter().collect();
    if let Fields::Named(_) = fields {
        out.sort_by(|l, r| l.ident.cmp(&r.ident));
    }
    out
}

fn impl_conversion_target(krate: &Path, input: &ItemEnum) -> TokenStream2 {
    let (impl_generics, arg_generics, whclause) = input.generics.split_for_impl();
    let self_ty = quote! { #{ &input.ident } #arg_generics };
    let mut out = quote! {};
    for variant in &input.variants {
        let fields = sort_fields(&variant.fields);
        out.extend(quote! {
            unsafe impl #impl_generics #krate::HasVariant<
                #{to_tstr(krate, &variant.ident.to_string(), variant.ident.span())}
            > for #self_ty #whclause {
                type Fields = (#(for (n, field) in fields.iter().enumerate()) {
                    #(if let Some(id) = &field.ident) {
                        #{to_tstr(krate, &id.to_string(), field.ident.span())}
                    }
                    #(else) {
                        #{to_tstr(krate, &format!("{}", n), field.span())}
                    },
                });
                type Offsets = [::core::primitive::usize; #{variant.fields.len()}];
                fn discriminant() -> ::core::mem::Discriminant<Self> {
                    #krate::addr_of_enum::get_discriminant!(Self, #{&variant.ident})
                }
                fn offsets() -> Self::Offsets {
                    let ptr: *const Self = unsafe{::core::mem::MaybeUninit::uninit().as_ptr()};
                    fn ptr_cast<T>(ptr: *const T) -> ::core::primitive::usize {
                        ptr as ::core::primitive::usize
                    }
                    #{emit_offsets(krate, &variant.ident, fields.as_slice())}
                }
            }
        });
    }
    out
}

fn variant_to_matcher(variant: &Variant) -> (&Ident, TokenStream2) {
    let matcher = match &variant.fields {
        Fields::Named(_) => {
            quote! { { #(for field in &variant.fields) { #{&field.ident}, } } }
        }
        Fields::Unnamed(_) => {
            quote! {(
                #(for (n, field) in variant.fields.iter().enumerate()) { #{Ident::new(&format!("a{}", n), field.span())}, }
            )}
        }
        Fields::Unit => quote! {},
    };
    (&variant.ident, matcher)
}

fn path_to_pat(mut path: Path) -> Option<(Path, PathArguments)> {
    let len = path.segments.len();
    assert!(len > 0);
    for seg in path.segments.iter().take(len - 1) {
        if seg.arguments != PathArguments::None {
            return None;
        }
    }
    let arg = path.segments[len - 1].arguments.clone();
    path.segments[len - 1].arguments = PathArguments::None;
    Some((path, arg))
}

struct ConvertToArgs {
    targets: Punctuated<Path, Token![,]>,
    krate: Option<Path>,
}

impl syn::parse::Parse for ConvertToArgs {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let mut targets = Punctuated::new();
        while !input.is_empty() {
            if let Ok(_at) = input.parse::<Token![@]>() {
                let krate = Some(input.parse()?);
                if !input.is_empty() {
                    return Err(input.error("Unexpected token"));
                }
                return Ok(Self { targets, krate });
            } else {
                targets.push_value(input.parse()?);
                if let Ok(punct) = input.parse() {
                    targets.push_punct(punct);
                }
            }
        }
        Ok(Self {
            targets,
            krate: None,
        })
    }
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn convert_to(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as ConvertToArgs);
    let input = parse_macro_input!(input as ItemEnum);
    let (impl_generics, arg_generics, whclause) = input.generics.split_for_impl();
    let krate = args.krate.unwrap_or(parse_quote! {::fast_enum_conversion});
    let mut out = quote! {
        #[derive(#krate::addr_of_enum::AddrOfEnum)]
        #input
    };
    out.extend(impl_conversion_target(&krate, &input));
    for target in args.targets.iter() {
        let whclause = whclause.map(|w| quote! {#w,}).unwrap_or(quote! {where});
        let where_clause = input.variants.iter().fold(whclause, |acc, variant| {
            let name = to_tstr(&krate, &variant.ident.to_string(), variant.ident.span());
            let mut fields: Punctuated<_, Token![,]> = sort_fields(&variant
                .fields)
                .iter()
                .enumerate()
                .map(|(n, field)| {
                    let field_name = field
                        .ident
                        .as_ref()
                        .map(|id| id.to_string())
                        .unwrap_or(format!("{}", n));
                    to_tstr(&krate, &field_name, field.ident.span())
                })
                .collect();
            if !fields.is_empty() {
                fields.push_punct(Default::default());
            }
            quote! {
                #acc
                #target: #krate::HasVariant<#name, Fields = (#fields), Offsets = [::core::primitive::usize; #{fields.len()}]>,
            }
        });
        let (target_pat, _target_arg) = path_to_pat(target.clone())
            .unwrap_or_else(|| abort!(target.span(), "Unsupported path"));

        out.extend(
            quote! {
                impl #impl_generics #krate::ConvertTo<#target> for #{&input.ident} #arg_generics
                    #where_clause
                {
                    fn convert_to(self) -> #target {
                        use ::core::mem::{MaybeUninit, replace, transmute};
                        if <Self as #krate::ConvertTo<#target>>::is_zerocost() {
                            let mut src = MaybeUninit::new(self);
                            // SAFETY: checked in `is_zerocost()`
                            unsafe {
                                replace(transmute(&mut *(src.as_mut_ptr())), MaybeUninit::uninit()).assume_init()
                            }
                        } else {
                            self.convert_to_slow()
                        }
                    }
                    fn convert_to_slow(self) -> #target {
                        match self {
                            #(for (id, matcher) in input.variants.iter().map(variant_to_matcher)) {
                                Self::#id #matcher => #target_pat::#id #matcher,
                            }
                        }
                    }
                    fn try_convert_from(this: #target) -> ::core::result::Result<Self, #target> {
                        Self::try_convert_from_slow(this)
                    }
                    fn try_convert_from_slow(this: #target) -> ::core::result::Result<Self, #target> {
                        match this {
                            #(for (id, matcher) in input.variants.iter().map(variant_to_matcher)) {
                                #target_pat::#id #matcher => ::core::result::Result::Ok(Self::#id #matcher),
                            }
                            #[allow(unreachable_patterns)]
                            o => ::core::result::Result::Err(o)
                        }
                    }
                    fn is_zerocost() -> ::core::primitive::bool {
                        use ::core::mem::{size_of, align_of, Discriminant, MaybeUninit};
                        let ptr: *const Self = MaybeUninit::uninit().as_ptr();
                        fn ptr_cast<T>(ptr: *const T) -> ::core::primitive::usize {
                            ptr as ::core::primitive::usize
                        }
                        ::core::mem::align_of::<Self>() >= ::core::mem::align_of::<#target>() &&
                            ::core::mem::size_of::<Self>() == ::core::mem::size_of::<#target>() &&
                            #(for variant in &input.variants) {
                                #(let tsname = to_tstr(&krate, &variant.ident.to_string(), variant.ident.span())) {
                                    size_of::<Discriminant<Self>>() == size_of::<Discriminant<#target>>() &&
                                        align_of::<Discriminant<Self>>() == align_of::<Discriminant<#target>>() &&
                                        unsafe{::core::mem::transmute::<::core::mem::Discriminant<Self>, ::core::mem::Discriminant<#target>>(#krate::addr_of_enum::get_discriminant!(Self, #{&variant.ident}))} ==
                                            <#target as #krate::HasVariant<#tsname>>::discriminant() &&
                                                #{ emit_offsets(&krate, &variant.ident, sort_fields(&variant.fields).as_slice()) }
                                                == <#target as #krate::HasVariant<#tsname>>::offsets() &&
                                }
                        }
                        true
                    }
                }
            }
        );
    }
    out.into()
}
