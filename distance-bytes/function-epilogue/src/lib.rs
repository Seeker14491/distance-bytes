use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::FnArg;

#[proc_macro_attribute]
pub fn epilogue(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_ast = syn::parse(attr).unwrap();
    let item_ast = syn::parse(item).unwrap();

    impl_epilogue(&attr_ast, &item_ast)
}

fn impl_epilogue(attr: &syn::ExprPath, item: &syn::ItemFn) -> TokenStream {
    let mut item = item.clone();

    let inputs_1 = item.sig.inputs.clone();
    let inputs_2 = item.sig.inputs.iter().map(|fn_arg| {
        let pat_type = match fn_arg {
            FnArg::Receiver(_) => panic!("epilogue macro does not support \"self\" parameters"),
            FnArg::Typed(x) => x
        };

        pat_type.pat.clone()
    });
    let block = item.block;
    let fn_block = quote! {
        {
            let original_fn = |#inputs_1| #block;
            #attr(original_fn(#(#inputs_2),*))
        }
    };

    item.block = syn::parse(fn_block.into()).unwrap();

    item.into_token_stream().into()
}
