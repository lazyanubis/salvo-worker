//! This crate implements necessary boiler plate code to serve ReDoc via web server. It
//! works as a bridge for serving the OpenAPI documentation created with [`salvo`][salvo] library in the
//! ReDoc.
//!
//! [salvo]: <https://docs.rs/salvo/>
//!
use salvo_core::writing::Text;
use salvo_core::{Depot, FlowCtrl, Handler, Request, Response, Router, async_trait};
use std::borrow::Cow;

const INDEX_TEMPLATE: &str = r#"
<!DOCTYPE html>
<html>
  <head>
    <title>{{title}}</title>
    {{keywords}}
    {{description}}
    {{favicon_url}}
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>
      body {
        margin: 0;
        padding: 0;
      }
    </style>
  </head>

  <body>
    <div id="redoc-container"></div>
    <script src="{{lib_url}}"></script>
    <script>
      Redoc.init(
        "{{spec_url}}",
        {},
        document.getElementById("redoc-container")
      );
    </script>
  </body>
</html>
"#;

/// Implements [`Handler`] for serving ReDoc.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct ReDoc {
    /// The title of the html page. The default title is "Scalar".
    pub title: Cow<'static, str>,
    /// The version of the html page.
    pub keywords: Option<Cow<'static, str>>,
    /// The description of the html page.
    pub description: Option<Cow<'static, str>>,
    /// The favicon url path
    pub favicon_url: Option<Cow<'static, str>>,
    /// The lib url path.
    pub lib_url: Cow<'static, str>,
    /// The spec url path.
    pub spec_url: Cow<'static, str>,
}

impl ReDoc {
    /// Create a new [`ReDoc`] for given path.
    ///
    /// Path argument will expose the ReDoc to the user and should be something that
    /// the underlying application framework / library supports.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use salvo_oapi::redoc::ReDoc;
    /// let doc = ReDoc::new("/openapi.json");
    /// ```
    pub fn new(spec_url: impl Into<Cow<'static, str>>) -> Self {
        Self {
            title: "ReDoc".into(),
            keywords: None,
            description: None,
            favicon_url: None,
            lib_url: "https://cdn.redoc.ly/redoc/latest/bundles/redoc.standalone.js".into(),
            spec_url: spec_url.into(),
        }
    }

    /// Set title of the html page. The default title is "Scalar".
    pub fn title(mut self, title: impl Into<Cow<'static, str>>) -> Self {
        self.title = title.into();
        self
    }

    /// Set keywords of the html page.
    pub fn keywords(mut self, keywords: impl Into<Cow<'static, str>>) -> Self {
        self.keywords = Some(keywords.into());
        self
    }

    /// Set description of the html page.
    pub fn description(mut self, description: impl Into<Cow<'static, str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set favicon of the html page.
    pub fn favicon_url(mut self, favicon_url: impl Into<Cow<'static, str>>) -> Self {
        self.favicon_url = Some(favicon_url.into());
        self
    }

    /// Set the lib url path.
    pub fn lib_url(mut self, lib_url: impl Into<Cow<'static, str>>) -> Self {
        self.lib_url = lib_url.into();
        self
    }

    /// Consumes the [`ReDoc`] and returns [`Router`] with the [`ReDoc`] as handler.
    pub fn into_router(self, path: impl Into<String>) -> Router {
        Router::with_path(path.into()).goal(self)
    }
}

#[async_trait]
impl Handler for ReDoc {
    async fn handle(&self, _req: &mut Request, _depot: &mut Depot, res: &mut Response, _ctrl: &mut FlowCtrl) {
        let keywords = self
            .keywords
            .as_ref()
            .map(|s| {
                format!(
                    "<meta name=\"keywords\" content=\"{}\">",
                    s.split(',').map(|s| s.trim()).collect::<Vec<_>>().join(",")
                )
            })
            .unwrap_or_default();
        let description = self
            .description
            .as_ref()
            .map(|s| format!("<meta name=\"description\" content=\"{}\">", s))
            .unwrap_or_default();
        let favicon_url = self
            .favicon_url
            .as_ref()
            .map(|s| format!("<link rel=\"icon\" href=\"{}\" type=\"image/x-icon\">", s))
            .unwrap_or_default();
        let html = INDEX_TEMPLATE
            .replacen("{{spec_url}}", &self.spec_url, 1)
            .replacen("{{lib_url}}", &self.lib_url, 1)
            .replacen("{{description}}", &description, 1)
            .replacen("{{favicon_url}}", &favicon_url, 1)
            .replacen("{{keywords}}", &keywords, 1)
            .replacen("{{title}}", &self.title, 1);
        res.render(Text::Html(html));
    }
}
