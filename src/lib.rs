#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

extern crate proc_macro;

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

macro_rules! tts {
    ($($item:expr),* $(,)?) => {
        [$(TokenTree::from($item)),*]
    };
}

macro_rules! punct {
    ($token:tt) => {
        Punct::new($token, Spacing::Joint)
    };
}

#[derive(Debug, Clone, Copy)]
enum SpanPair {
    Native {
        start: proc_macro::Span,
        end: proc_macro::Span,
    },
    #[cfg(feature = "proc-macro2")]
    ProcMacro2 {
        start: proc_macro2::Span,
        end: proc_macro2::Span,
    },
}

impl SpanPair {
    fn start_native(&self) -> Option<proc_macro::Span> {
        match *self {
            Self::Native { start, .. } => Some(start),
            #[cfg(feature = "proc-macro2")]
            Self::ProcMacro2 { start, .. } => proc_macro::is_available().then(|| start.unwrap()),
        }
    }

    fn end_native(&self) -> Option<proc_macro::Span> {
        match *self {
            Self::Native { end, .. } => Some(end),
            #[cfg(feature = "proc-macro2")]
            Self::ProcMacro2 { end, .. } => proc_macro::is_available().then(|| end.unwrap()),
        }
    }
}

impl From<proc_macro::Span> for SpanPair {
    fn from(span: proc_macro::Span) -> Self {
        SpanPair::Native {
            start: span,
            end: span,
        }
    }
}

impl From<(proc_macro::Span, proc_macro::Span)> for SpanPair {
    fn from(spans: (proc_macro::Span, proc_macro::Span)) -> Self {
        SpanPair::Native {
            start: spans.0,
            end: spans.1,
        }
    }
}

#[cfg(feature = "proc-macro2")]
impl From<proc_macro2::Span> for SpanPair {
    fn from(span: proc_macro2::Span) -> Self {
        SpanPair::ProcMacro2 {
            start: span,
            end: span,
        }
    }
}

#[cfg(feature = "proc-macro2")]
impl From<(proc_macro2::Span, proc_macro2::Span)> for SpanPair {
    fn from(spans: (proc_macro2::Span, proc_macro2::Span)) -> Self {
        SpanPair::ProcMacro2 {
            start: spans.0,
            end: spans.1,
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
    span: Option<SpanPair>,
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
    /// This method accepts a [`proc_macro::Span`], [`proc_macro2::Span`], `(proc_macro::Span,
    /// proc_macro::Span)`, or `(proc_macro2::Span, proc_macro2::Span)`. If a tuple is provided,
    /// the error message will encompass both spans. Note that the `proc-macro2` feature must be
    /// enabled to pass a `proc_macro2::Span`.
    ///
    /// If this method is called multiple times, the final call takes precedence.
    #[allow(private_bounds)] // deliberately not exposing inner type
    pub fn span(mut self, span: impl Into<SpanPair>) -> Self {
        self.span = Some(span.into());
        self
    }

    /// Append the error to the provided `TokenStream`.
    pub fn to_tokens(&self, tokens: &mut TokenStream) {
        let call_site = Span::call_site();
        let start_span = self
            .span
            .and_then(|pair| pair.start_native())
            .unwrap_or(call_site);
        let end_span = self
            .span
            .and_then(|pair| pair.end_native())
            .unwrap_or(call_site);

        macro_rules! ident {
            ($name:ident) => {
                Ident::new(stringify!($name), call_site)
            };
        }

        let customization = {
            let mut ts = TokenStream::from_iter(tts![
                ident!(message),
                punct!('='),
                Literal::string(&self.message),
            ]);
            if let Some(label) = &self.label {
                ts.extend(tts![
                    punct!(','),
                    ident!(label),
                    punct!('='),
                    Literal::string(label),
                ]);
            }
            for note in &self.notes {
                ts.extend(tts![
                    punct!(','),
                    ident!(note),
                    punct!('='),
                    Literal::string(note),
                ]);
            }
            ts
        };

        let mut inner_ts = TokenStream::from_iter(tts![
            punct!('#'),
            Group::new(
                Delimiter::Bracket,
                TokenStream::from_iter(tts![
                    ident!(diagnostic),
                    punct!(':'),
                    punct!(':'),
                    ident!(on_unimplemented),
                    Group::new(Delimiter::Parenthesis, customization),
                ]),
            ),
            ident!(trait),
            ident!(DiagnosticHack),
            Group::new(Delimiter::Brace, TokenStream::new()),
        ]);
        #[cfg(not(feature = "msrv-1-78"))]
        inner_ts.extend(tts![
            punct!('#'),
            Group::new(
                Delimiter::Bracket,
                TokenStream::from_iter(tts![
                    ident!(diagnostic),
                    punct!(':'),
                    punct!(':'),
                    ident!(do_not_recommend),
                ]),
            ),
        ]);

        inner_ts.extend(tts![
            ident!(impl),
            ident!(DiagnosticHack),
            ident!(for),
            punct!(':'),
            punct!(':'),
            ident!(core),
            punct!(':'),
            punct!(':'),
            ident!(convert),
            punct!(':'),
            punct!(':'),
            ident!(Infallible),
            Group::new(Delimiter::Brace, TokenStream::new()),
            ident!(fn),
            ident!(diagnostic_hack),
            punct!('<'),
            ident!(T),
            punct!(':'),
            ident!(DiagnosticHack),
            punct!('>'),
            Group::new(Delimiter::Parenthesis, TokenStream::new()),
            Group::new(Delimiter::Brace, TokenStream::new()),
            ident!(diagnostic_hack),
            punct!(':'),
            punct!(':'),
            punct!('<'),
            {
                let mut tt = punct!('*');
                tt.set_span(start_span);
                tt
            },
            ident!(const),
            {
                let mut tt = Group::new(Delimiter::Parenthesis, TokenStream::new());
                tt.set_span(end_span);
                tt
            },
            punct!('>'),
            Group::new(Delimiter::Parenthesis, TokenStream::new()),
            punct!(';'),
        ]);

        tokens.extend(tts![Group::new(Delimiter::Brace, inner_ts)]);
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for Error {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mut ts = TokenStream::new();
        self.to_tokens(&mut ts);
        tokens.extend(proc_macro2::TokenStream::from(ts));
    }
}
