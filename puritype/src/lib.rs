use syn::{parse, visit::{self, Visit}};
use syn::punctuated::Punctuated;
use proc_macro2::TokenStream;
use quote::quote;

struct ListOfItems(Vec<syn::Item>);

impl parse::Parse for ListOfItems {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        let mut items = Vec::new();

        while !input.cursor().eof() {
            let item = input.parse()?;
            items.push(item);
        }

        Ok(Self(items))
    }
}

#[derive(Default)]
struct SyntaxTree<'ast> {
    enums: Vec<&'ast syn::ItemEnum>
}

impl<'ast> visit::Visit<'ast> for SyntaxTree<'ast> {
    fn visit_item_enum(&mut self, item_enum: &'ast syn::ItemEnum) {
        self.enums.push(item_enum)
    }
}

#[proc_macro]
pub fn puritype(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ListOfItems(items) = syn::parse_macro_input!(ts as ListOfItems);

    let mut syntax = SyntaxTree::default();

    for item in &items {
        syntax.visit_item(item);
    }

    let mut output = TokenStream::new();

    for e in syntax.enums {
        let vis = &e.vis;
        let trait_ident = &e.ident;
        
        output.extend(quote! {
            #vis trait #trait_ident {}
        });

        for variant in &e.variants {
            let attrs = &variant.attrs;
            let ident = &variant.ident;

            let (impl_types, applied_types, generics_and_fields) =  match &variant.fields {
                syn::Fields::Named(fields) => todo!("add named field support"),
                syn::Fields::Unit => (TokenStream::new(), TokenStream::new(), quote!(;)),
                syn::Fields::Unnamed(fields) => {
                    let mut impl_types = Punctuated::<TokenStream, syn::Token![,]>::new();
                    let mut applied_types = Punctuated::<syn::Ident, syn::Token![,]>::new();

                    for (i, field) in fields.unnamed.iter().enumerate() {
                        let ident = syn::Ident::new(&format!("__TypeParam__{}", i), proc_macro2::Span::call_site());
                        let field_ty = &field.ty;

                        let mut has_pushed = false;

                        if let syn::Type::Path(ty) = field_ty {
                            if let Some(ty_ident) = ty.path.get_ident() {
                                if ty_ident == "Type" {
                                    impl_types.push(quote!{#ident});
                                    has_pushed = true;
                                }
                            }
                        }

                        if !has_pushed {
                            impl_types.push(quote!{#ident: #field_ty})
                        }

                        applied_types.push(ident);
                    }

                    (
                        quote! {
                            <#impl_types>
                        },
                        quote! {
                            <#applied_types>
                        },
                        quote! {
                            <#applied_types>(#applied_types);
                        }
                    )
                }
            };

            output.extend(quote! {
                #(#attrs)*
                #vis struct #ident #generics_and_fields
                impl #impl_types #trait_ident for #ident #applied_types {}
            })
        }
    }

    proc_macro::TokenStream::from(output)
}