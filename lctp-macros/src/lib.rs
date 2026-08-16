//! Attribute macros for registering linux-container-test cases.

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Ident, ItemFn, Result, Token};

struct TestArgs {
    suite: Ident,
    full: bool,
    expect: Ident,
    case: syn::Expr,
}

impl Parse for TestArgs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut suite: Option<Ident> = None;
        let mut full = false;
        let mut expect: Option<Ident> = None;
        let mut case: Option<syn::Expr> = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            if key == "suite" {
                input.parse::<Token![=]>()?;
                suite = Some(input.parse()?);
            } else if key == "full" {
                full = true;
            } else if key == "expect" {
                input.parse::<Token![=]>()?;
                expect = Some(input.parse()?);
            } else if key == "case" {
                input.parse::<Token![=]>()?;
                case = Some(input.parse()?);
            } else {
                return Err(syn::Error::new(
                    key.span(),
                    "expected `suite`, `expect`, `case`, or `full`",
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
        let expect = expect.ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "missing required argument `expect = success|failure|soft`",
            )
        })?;
        let case = case.ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "missing required argument `case = \"...\"`",
            )
        })?;

        Ok(Self {
            suite,
            full,
            expect,
            case,
        })
    }
}

/// Annotate a test function to register it with the harness.
///
/// `expect` is the outcome under test:
/// - `success` — the operation or property is required to succeed / hold
/// - `failure` — the operation is required to fail (named errno in `case`)
/// - `soft` — success if the interface is available; unsupported rejection is accepted
///
/// ```ignore
/// #[lctp_test(suite = fs, expect = success, case = "chmod on a regular file sets mode 0644")]
/// fn chmod_file_644() -> TestResult { ... }
///
/// #[lctp_test(suite = fs, full, expect = failure, case = "open without write permission returns EACCES")]
/// fn open_write_denied() -> TestResult { ... }
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

    let expect_variant = match args.expect.to_string().as_str() {
        "success" => quote!(crate::harness::Expect::Success),
        "failure" => quote!(crate::harness::Expect::Failure),
        "soft" => quote!(crate::harness::Expect::Soft),
        other => {
            return syn::Error::new(
                args.expect.span(),
                format!("unknown expect `{other}`; expected success, failure, or soft"),
            )
            .to_compile_error()
            .into();
        }
    };

    let full_only = args.full;
    let case = args.case;

    let expanded = quote! {
        #func

        #[::linkme::distributed_slice(crate::harness::ALL_TESTS)]
        #[linkme(crate = ::linkme)]
        static #static_name: crate::harness::TestCase = crate::harness::TestCase {
            name: #fn_name_str,
            suite: #suite_variant,
            full_only: #full_only,
            expect: #expect_variant,
            case: #case,
            func: #fn_name,
        };
    };

    expanded.into()
}
