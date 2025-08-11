Leverage compiler-provided functionality to emit custom error messages in procedural macros.

# Example

```rust,ignore
use proc_diag::Error;

Error::new("custom error message")
    .label("inline label")
    .note("some context")
    .note("additional context")
    .span(some_span) // or .span((start_span, end_span))
    .to_tokens(&mut output);
```

```text
error[E0277]: custom error message
 --> src/main.rs:L:11
  |
L |     demo!("macro input");
  |           ^^^^^^^^^^^^^ inline label
  |
  = help: the trait `DiagnosticHack` is not implemented for `*const ()`
  = note: some context
  = note: additional context
note: required by a bound in `diagnostic_hack`
 --> src/main.rs:L:5
  |
L |     demo!("macro input");
  |     ^^^^^^^^^^^^^^^^^^^^ required by this bound in `diagnostic_hack`
  = note: this error originates in the macro `demo` (in Nightly builds, run with -Z macro-backtrace for more info)
```

# Feature flags

This crate has three feature flags:

- `quote` enables an implementation of `quote::ToTokens`. This also enables the `proc-macro2`
  feature.
- `proc-macro2` enables passing `proc_macro2::Span` and `(proc_macro2::Span, proc_macro2::Span)` to
  the `span` method.
- `msrv-1-78`, which lowers the minimum supported Rust version from 1.85 to 1.78. The side
  effect of enabling this is an additional (irrelevant) line in the diagnostic output.
