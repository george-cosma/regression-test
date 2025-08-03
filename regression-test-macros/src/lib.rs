//! # regression-test-macros
//!
//! This crate provides procedural macros to support regression testing in Rust projects.
//! The main macro, `regtest`, is an attribute macro designed to be applied to test functions.
//! It automatically generates boilerplate code for managing regression test data files,
//! ensuring that each test has a dedicated file for storing and comparing results.
//!
//! ## Features
//! - Automatically determines and creates the appropriate file path for regression data
//!   based on the test's source location (unit or integration test).
//! - Handles compatibility with tools like rust-analyzer.
//!
//! ## Usage
//!
//! ```rust
//! use regression_test_macros::regtest;
//! use regression_test::RegTest;
//!
//! #[regtest]
//! fn my_regression_test(rt: RegTest) {
//!     // Test logic here
//!     rt.regtest("some output");
//!     rt.regtest_dbg(vec![1, 2, 3]);
//! }
//! ```
//!
//! The macro will ensure that a regression data file is created and passed to the test
//! via the `RegTest` argument.
use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

/// Attribute macro for regression tests.
///
/// This macro should be applied to test functions whose first argument is of type `RegTest`.
/// It generates a `#[test]` function that automatically manages the file path for regression
/// test data, creating the necessary directories and passing a `RegTest` instance to the test.
///
/// # Requirements
/// - The first argument of the function must be of type `RegTest`.
///
/// # Example
/// ```rust
/// use regression_test::RegTest;
/// use regression_test_macros::regtest;
///
/// #[regtest]
/// fn my_test(rt: RegTest) {
///     // Test logic
///     rt.regtest("some output");
///     rt.regtest_dbg(vec![1, 2, 3]);
/// }
/// ```
///
/// The macro will inject code to determine the appropriate file path for the regression data,
/// create the file if necessary, and pass a `RegTest` instance to the test function.
#[proc_macro_attribute]
pub fn regtest(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let fn_attrs = &input_fn.attrs;
    let fn_vis = &input_fn.vis;
    let fn_block = &input_fn.block;
    let fn_inputs = &input_fn.sig.inputs;
    let fn_async = &input_fn.sig.asyncness;

    // Check if there is at least one argument
    let first_arg = match fn_inputs.iter().next() {
        Some(arg) => arg,
        None => {
            return syn::Error::new_spanned(
                &input_fn.sig,
                "Expected at least one argument of type 'RegTest', but found none.",
            )
            .to_compile_error()
            .into();
        }
    };

    // Check if the first argument is a typed argument and of type RegTest (by last segment)
    let arg_pat = if let syn::FnArg::Typed(pat_type) = first_arg {
        if let syn::Type::Path(type_path) = &*pat_type.ty {
            if let Some(last_segment) = type_path.path.segments.last() {
                if last_segment.ident == "RegTest" {
                    &pat_type.pat
                } else {
                    return syn::Error::new_spanned(
                        &pat_type.ty,
                        format!(
                            "Expected the first argument to be of type RegTest, but found type '{}'.",
                            last_segment.ident
                        )
                    ).to_compile_error().into();
                }
            } else {
                return syn::Error::new_spanned(
                    &pat_type.ty,
                    "Expected the first argument to be of type RegTest, but found an empty type path."
                ).to_compile_error().into();
            }
        } else {
            return syn::Error::new_spanned(
                &pat_type.ty,
                format!(
                    "Expected the first argument to be of type RegTest, but found a different type: {}.",
                    quote!(#pat_type.ty).to_string()
                )
            ).to_compile_error().into();
        }
    } else {
        return syn::Error::new_spanned(
            first_arg,
            format!(
                "Expected the first argument to be a typed argument (e.g., arg: RegTest), but found: `{}`.",
                quote!(#first_arg).to_string()
            )
        ).to_compile_error().into();
    };

    // Try to get the local file path, but handle rust-analyzer bug where local_file() returns None
    let file_path_opt = proc_macro::Span::call_site().local_file();

    let regtest_path_quote = if let Some(full_file_path_buf) = file_path_opt {
        let full_file_path_buf = full_file_path_buf
            .canonicalize()
            .expect("Failed to canonicalize the file path");

        let full_file_path = full_file_path_buf
            .to_str()
            .expect("Failed to convert the file path to a string")
            .to_string();

        // Path computation quote
        quote! {
            // Determine the file path for the regression test data
            let __regtest_file_path = {
                use std::path::{Path, PathBuf};

                let file = #full_file_path;
                let test_name = stringify!(#fn_name);
                let path = Path::new(file);

                // Helper to get the relative path after "src" or "tests"
                fn relative_mod_path(path: &std::path::Path) -> std::path::PathBuf {
                    let mut components = path.components().peekable();
                    let mut found = false;
                    let mut rel = PathBuf::new();
                    while let Some(comp) = components.next() {
                        if found {
                            rel.push(comp.as_os_str());
                        }
                        if comp.as_os_str() == "src" || comp.as_os_str() == "tests" {
                            found = true;
                        }
                    }
                    rel
                }

                let mut base = {
                    // Check if this is an integration test (in "tests" folder)
                    if path.components().any(|c| c.as_os_str() == "tests") {
                        // Place the file next to the test file, preserving subfolders after "tests"
                        let ancestor = path.ancestors().find(|a| a.ends_with("tests")).unwrap_or_else(|| Path::new(""));
                        let rel = relative_mod_path(path);
                        let mut p = ancestor.parent().unwrap_or_else(|| Path::new("")).to_path_buf();
                        p.push("regtest_data");
                        p.push("tests");
                        if let Some(parent) = rel.parent() {
                            p.push(parent);
                        }
                        p
                    } else {
                        // Place the file in "unit_tests" at the same level as "src"
                        let ancestor = path.ancestors().find(|a| a.ends_with("src")).unwrap_or_else(|| Path::new(""));
                        let rel = relative_mod_path(path);
                        let mut p = ancestor.parent().unwrap_or_else(|| Path::new("")).to_path_buf();
                        p.push("regtest_data");
                        p.push("src");
                        if let Some(parent) = rel.parent() {
                            p.push(parent);
                        }
                        p
                    }
                };

                // Add the file stem as a directory
                if let Some(file_stem) = path.file_stem() {
                    base.push(file_stem);
                }

                // Create the directory if it doesn't exist
                std::fs::create_dir_all(&base).ok();

                // Add the test name as the file
                base.push(format!("{}.json", test_name));
                base
            };
        }
    } else {
        // rust-analyzer fallback
        quote! {
            let __regtest_file_path = "./rust-analyzer-dummy.json".to_string();
        }
    };

    let fn_quote = quote! {
        #[test]
        #(#fn_attrs)*
        #fn_vis #fn_async fn #fn_name() {
            #regtest_path_quote
            let #arg_pat = RegTest::new(__regtest_file_path).expect("Failed to create or open regression test file");
            #fn_block
        }
    };

    TokenStream::from(fn_quote)
}
