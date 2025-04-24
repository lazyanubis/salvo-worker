use std::borrow::Cow;

use salvo_core::async_trait;
use salvo_core::http::header::{AUTHORIZATION, HeaderName, PROXY_AUTHORIZATION};
use salvo_core::http::{Method, Request};

use super::ALL_METHODS;

/// Trait for extracting JWT tokens from HTTP requests.
///
/// Implementors of this trait provide different strategies for locating JWT tokens
/// in various parts of an HTTP request (headers, query string, cookies, etc.).
/// The `JwtAuth` middleware tries each configured finder in sequence until one
/// returns a token.
#[async_trait]
pub trait JwtTokenFinder: Send + Sync {
    /// Attempts to extract a JWT token from the request.
    ///
    /// Returns `Some(String)` containing the token if found, or `None` if no token
    /// could be extracted using this finder's strategy.
    async fn find_token(&self, req: &mut Request) -> Option<String>;
}

/// Extracts JWT tokens from HTTP request headers.
///
/// By default, this finder looks for Bearer tokens in the `Authorization`
/// and `Proxy-Authorization` headers for all HTTP methods.
///
#[derive(Eq, PartialEq, Clone, Default)]
#[non_exhaustive]
pub struct HeaderFinder {
    /// List of HTTP methods for which this finder should extract tokens.
    /// If the request's method is not in this list, the finder will not attempt extraction.
    pub cared_methods: Vec<Method>,

    /// List of headers names to check for Bearer tokens.
    pub header_names: Vec<HeaderName>,
}
impl HeaderFinder {
    /// Create new `HeaderFinder`.
    #[inline]
    pub fn new() -> Self {
        Self {
            cared_methods: ALL_METHODS.to_vec(),
            header_names: vec![AUTHORIZATION, PROXY_AUTHORIZATION],
        }
    }

    /// Get header names mutable reference.
    #[inline]
    pub fn header_names_mut(&mut self) -> &mut Vec<HeaderName> {
        &mut self.header_names
    }

    /// Sets header names and returns `Self`.
    #[inline]
    pub fn header_names(mut self, header_names: impl Into<Vec<HeaderName>>) -> Self {
        self.header_names = header_names.into();
        self
    }

    /// Get cared methods list mutable reference.
    #[inline]
    pub fn cared_methods_mut(&mut self) -> &mut Vec<Method> {
        &mut self.cared_methods
    }
    /// Sets cared methods list and returns `Self`.
    #[inline]
    pub fn cared_methods(mut self, methods: Vec<Method>) -> Self {
        self.cared_methods = methods;
        self
    }
}
#[async_trait]
impl JwtTokenFinder for HeaderFinder {
    #[inline]
    async fn find_token(&self, req: &mut Request) -> Option<String> {
        if self.cared_methods.contains(req.method()) {
            for header_name in &self.header_names {
                if let Some(Ok(auth)) = req.headers().get(header_name).map(|auth| auth.to_str()) {
                    if auth.starts_with("Bearer") {
                        return auth.split_once(' ').map(|(_, token)| token.to_owned());
                    }
                }
            }
        }
        None
    }
}

/// Extracts JWT tokens from request form data.
///
/// This finder looks for a token in the request's form data using a specified field name.
///
#[derive(Eq, PartialEq, Clone, Default)]
#[non_exhaustive]
pub struct FormFinder {
    /// List of HTTP methods for which this finder should extract tokens.
    pub cared_methods: Vec<Method>,

    /// Name of the form field containing the token.
    pub field_name: Cow<'static, str>,
}
impl FormFinder {
    /// Create new `FormFinder`.
    #[inline]
    pub fn new<T: Into<Cow<'static, str>>>(field_name: T) -> Self {
        Self {
            field_name: field_name.into(),
            cared_methods: ALL_METHODS.to_vec(),
        }
    }
    /// Get cared methods list mutable reference.
    #[inline]
    pub fn cared_methods_mut(&mut self) -> &mut Vec<Method> {
        &mut self.cared_methods
    }
    /// Sets cared methods list and returns Self.
    #[inline]
    pub fn cared_methods(mut self, methods: Vec<Method>) -> Self {
        self.cared_methods = methods;
        self
    }
}
#[async_trait]
impl JwtTokenFinder for FormFinder {
    #[inline]
    async fn find_token(&self, req: &mut Request) -> Option<String> {
        if self.cared_methods.contains(req.method()) {
            req.form(&self.field_name).await
        } else {
            None
        }
    }
}

/// Extracts JWT tokens from URL query parameters.
///
/// This finder looks for a token in the request's query string using a specified parameter name.
///
#[derive(Eq, PartialEq, Clone, Default)]
#[non_exhaustive]
pub struct QueryFinder {
    /// List of HTTP methods for which this finder should extract tokens.
    pub cared_methods: Vec<Method>,

    /// Name of the query parameter containing the token.
    pub query_name: Cow<'static, str>,
}
impl QueryFinder {
    /// Create new `QueryFinder`.
    #[inline]
    pub fn new<T: Into<Cow<'static, str>>>(query_name: T) -> Self {
        Self {
            query_name: query_name.into(),
            cared_methods: ALL_METHODS.to_vec(),
        }
    }
    /// Get cared methods list mutable reference.
    #[inline]
    pub fn cared_methods_mut(&mut self) -> &mut Vec<Method> {
        &mut self.cared_methods
    }
    /// Sets cared methods list and returns Self.
    #[inline]
    pub fn cared_methods(mut self, methods: Vec<Method>) -> Self {
        self.cared_methods = methods;
        self
    }
}

#[async_trait]
impl JwtTokenFinder for QueryFinder {
    #[inline]
    async fn find_token(&self, req: &mut Request) -> Option<String> {
        if self.cared_methods.contains(req.method()) {
            req.query(&self.query_name)
        } else {
            None
        }
    }
}

/// Extracts JWT tokens from cookies.
///
/// This finder looks for a token in the request's cookies using a specified cookie name.
///
#[derive(Eq, PartialEq, Clone, Default)]
#[non_exhaustive]
pub struct CookieFinder {
    /// List of HTTP methods for which this finder should extract tokens.
    pub cared_methods: Vec<Method>,

    /// Name of the cookie containing the token.
    pub cookie_name: Cow<'static, str>,
}
impl CookieFinder {
    /// Create new `CookieFinder`.
    #[inline]
    pub fn new<T: Into<Cow<'static, str>>>(cookie_name: T) -> Self {
        Self {
            cookie_name: cookie_name.into(),
            cared_methods: ALL_METHODS.to_vec(),
        }
    }
    /// Get cared methods list mutable reference.
    #[inline]
    pub fn cared_methods_mut(&mut self) -> &mut Vec<Method> {
        &mut self.cared_methods
    }
    /// Sets cared methods list and returns Self.
    #[inline]
    pub fn cared_methods(mut self, methods: Vec<Method>) -> Self {
        self.cared_methods = methods;
        self
    }
}
#[async_trait]
impl JwtTokenFinder for CookieFinder {
    #[inline]
    async fn find_token(&self, req: &mut Request) -> Option<String> {
        if self.cared_methods.contains(req.method()) {
            req.cookie(&self.cookie_name).map(|c| c.value().to_owned())
        } else {
            None
        }
    }
}
