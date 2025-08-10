#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

use proc_macro2::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

use crate::sealed::Spanned;

mod sealed {
    use proc_macro2::Span;

    pub trait Spanned {
        fn into_pair(self) -> (Span, Span);
    }

    impl Spanned for Span {
        fn into_pair(self) -> (Span, Span) {
            (self, self)
        }
    }

    impl Spanned for (Span, Span) {
        fn into_pair(self) -> (Span, Span) {
            self
        }
    }
}

/// A structure representing an error message.
///
/// **Note**: The output of this structure is only valid in expression position.
#[must_use = "this struct does nothing unless explicitly appended to a `TokenStream`"]
#[derive(Debug, Clone)]
pub struct Error {
    message: Box<str>,
    label: Option<Box<str>>,
    notes: Vec<Box<str>>,
    span: Option<(Span, Span)>,
}

impl Error {
    /// Create a new `Error` with the given top-level error message.
    pub fn new(message: impl ToString) -> Self {
        Self {
            message: message.to_string().into_boxed_str(),
            label: None,
            notes: Vec::new(),
            span: None,
        }
    }

    /// Set the label shown inline in the broken code.
    ///
    /// If this method is called multiple times, the final call takes precedence.
    pub fn label(mut self, label: impl ToString) -> Self {
        self.label = Some(label.to_string().into_boxed_str());
        self
    }

    /// Add a note to the error. This method may be called multiple times to add multiple notes.
    pub fn note(mut self, note: impl ToString) -> Self {
        self.notes.push(note.to_string().into_boxed_str());
        self
    }

    /// Set the span of the error.
    ///
    /// This method accepts either a [`Span`] or `(Span, Span)`. If the tuple is provided, the error
    /// message will encompass both spans.
    pub fn span(mut self, span: impl Spanned) -> Self {
        self.span = Some(span.into_pair());
        self
    }

    /// Append the error to the provided `TokenStream`.
    pub fn to_tokens(&self, tokens: &mut TokenStream) {
        let call_site = Span::call_site();
        let start_span = self.span.map_or(call_site, |(s, _)| s);
        let end_span = self.span.map_or(call_site, |(_, e)| e);

        let mut inner_ts = TokenStream::new();

        let mut customization = TokenStream::from_iter([
            TokenTree::Ident(Ident::new("message", call_site)),
            TokenTree::Punct(Punct::new('=', Spacing::Alone)),
            TokenTree::Literal(Literal::string(&self.message)),
        ]);
        if let Some(label) = &self.label {
            customization.extend([
                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                TokenTree::Ident(Ident::new("label", call_site)),
                TokenTree::Punct(Punct::new('=', Spacing::Alone)),
                TokenTree::Literal(Literal::string(label)),
            ]);
        }
        for note in &self.notes {
            customization.extend([
                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                TokenTree::Ident(Ident::new("note", call_site)),
                TokenTree::Punct(Punct::new('=', Spacing::Alone)),
                TokenTree::Literal(Literal::string(note)),
            ]);
        }

        inner_ts.extend([
            TokenTree::Punct(Punct::new('#', Spacing::Alone)),
            TokenTree::Group(Group::new(
                Delimiter::Bracket,
                TokenStream::from_iter([
                    TokenTree::Ident(Ident::new("diagnostic", call_site)),
                    TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                    TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                    TokenTree::Ident(Ident::new("on_unimplemented", call_site)),
                    TokenTree::Group(Group::new(Delimiter::Parenthesis, customization)),
                ]),
            )),
            TokenTree::Ident(Ident::new("trait", call_site)),
            TokenTree::Ident(Ident::new("DiagnosticHack", call_site)),
            TokenTree::Group(Group::new(Delimiter::Brace, TokenStream::new())),
        ]);

        #[cfg(not(feature = "msrv-1-78"))]
        inner_ts.extend([
            TokenTree::Punct(Punct::new('#', Spacing::Alone)),
            TokenTree::Group(Group::new(
                Delimiter::Bracket,
                TokenStream::from_iter([
                    TokenTree::Ident(Ident::new("diagnostic", call_site)),
                    TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                    TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                    TokenTree::Ident(Ident::new("do_not_recommend", call_site)),
                ]),
            )),
        ]);

        inner_ts.extend([
            TokenTree::Ident(Ident::new("impl", call_site)),
            TokenTree::Ident(Ident::new("DiagnosticHack", call_site)),
            TokenTree::Ident(Ident::new("for", call_site)),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("core", call_site)),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("convert", call_site)),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("Infallible", call_site)),
            TokenTree::Group(Group::new(Delimiter::Brace, TokenStream::new())),
            TokenTree::Ident(Ident::new("fn", call_site)),
            TokenTree::Ident(Ident::new("diagnostic_hack", call_site)),
            TokenTree::Punct(Punct::new('<', Spacing::Alone)),
            TokenTree::Ident(Ident::new("T", call_site)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("DiagnosticHack", call_site)),
            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::new())),
            TokenTree::Group(Group::new(Delimiter::Brace, TokenStream::new())),
            TokenTree::Ident(Ident::new("diagnostic_hack", call_site)),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Punct(Punct::new('<', Spacing::Alone)),
            {
                let mut tt = TokenTree::Punct(Punct::new('*', Spacing::Alone));
                tt.set_span(start_span);
                tt
            },
            TokenTree::Ident(Ident::new("const", start_span)),
            {
                let mut tt =
                    TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::new()));
                tt.set_span(end_span);
                tt
            },
            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::new())),
            TokenTree::Punct(Punct::new(';', Spacing::Alone)),
        ]);

        tokens.extend([TokenTree::Group(Group::new(Delimiter::Brace, inner_ts))]);
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for Error {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.to_tokens(tokens);
    }
}
