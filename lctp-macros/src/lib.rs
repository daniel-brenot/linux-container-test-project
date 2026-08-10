//! Attribute macros for registering linux-container-test cases.

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Ident, ItemFn, Result, Token};

struct TestArgs {
    suite: Ident,
    full: bool,
}

impl Parse for TestArgs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut suite: Option<Ident> = None;
        let mut full = false;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            if key == "suite" {
                input.parse::<Token![=]>()?;
                suite = Some(input.parse()?);
            } else if key == "full" {
                full = true;
            } else {
                return Err(syn::Error::new(
                    key.span(),
                    "expected `suite = <name>` or `full`",
                ));
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        let suite = suite.ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "missing required argument `suite = bootstrap|syscall|posix|fs`",
            )
        })?;

        Ok(Self { suite, full })
    }
}

/// Annotate a test function to register it with the harness.
///
/// ```ignore
/// #[lctp_test(suite = fs)]
/// fn chmod_file_644() -> TestResult { ... }
///
/// #[lctp_test(suite = fs, full)]
/// fn chmod_file_777() -> TestResult { ... }
/// ```
#[proc_macro_attribute]
pub fn lctp_test(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as TestArgs);
    let func = parse_macro_input!(item as ItemFn);

    let fn_name = &func.sig.ident;
    let fn_name_str = fn_name.to_string();
    let static_name = Ident::new(
        &format!("__LCTP_TEST_{fn_name_str}"),
        fn_name.span(),
    );

    let suite_variant = match args.suite.to_string().as_str() {
        "bootstrap" => quote!(crate::harness::Suite::Bootstrap),
        "syscall" => quote!(crate::harness::Suite::Syscall),
        "posix" => quote!(crate::harness::Suite::Posix),
        "fs" => quote!(crate::harness::Suite::Fs),
        other => {
            return syn::Error::new(
                args.suite.span(),
                format!("unknown suite `{other}`; expected bootstrap, syscall, posix, or fs"),
            )
            .to_compile_error()
            .into();
        }
    };

    let full_only = args.full;

    let expanded = quote! {
        #func

        #[::linkme::distributed_slice(crate::harness::ALL_TESTS)]
        #[linkme(crate = ::linkme)]
        static #static_name: crate::harness::TestCase = crate::harness::TestCase {
            name: #fn_name_str,
            suite: #suite_variant,
            full_only: #full_only,
            func: #fn_name,
        };
    };

    expanded.into()
}
