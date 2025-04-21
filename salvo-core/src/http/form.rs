//! Form parse module.
#[allow(unused)]
use std::ffi::OsStr;
#[allow(unused)]
use std::io::{Cursor, Write};
#[allow(unused)]
use std::path::{Path, PathBuf};

#[allow(unused)]
use base64::engine::Engine;
#[allow(unused)]
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
#[allow(unused)]
use futures_util::StreamExt;
#[allow(unused)]
use http_body_util::BodyExt;
use mime::Mime;
#[allow(unused)]
use multer::Field;
#[allow(unused)]
use multer::Multipart;
use multimap::MultiMap;
#[cfg(feature = "needless")]
use rand::TryRngCore;
#[cfg(feature = "needless")]
use rand::rngs::OsRng;
#[cfg(feature = "needless")]
use tempfile::Builder;
#[cfg(feature = "needless")]
use tokio::fs::File;
#[cfg(feature = "needless")]
use tokio::io::AsyncWriteExt;

#[allow(unused)]
use crate::http::ParseError;
#[allow(unused)]
use crate::http::body::ReqBody;
use crate::http::header::{CONTENT_TYPE, HeaderMap};

/// The extracted text fields and uploaded files from a `multipart/form-data` request.
#[derive(Debug)]
#[non_exhaustive]
pub struct FormData {
    /// Name-value pairs for plain text fields. Technically, these are form data parts with no
    /// filename specified in the part's `Content-Disposition`.
    pub fields: MultiMap<String, String>,
    /// Name-value pairs for temporary files. Technically, these are form data parts with a filename
    /// specified in the part's `Content-Disposition`.
    pub files: MultiMap<String, FilePart>,
}

impl FormData {
    /// Create new `FormData`.
    #[inline]
    pub fn new() -> FormData {
        FormData {
            fields: MultiMap::new(),
            files: MultiMap::new(),
        }
    }

    /// Parse MIME `multipart/*` information from a stream as a `FormData`.
    #[cfg(feature = "needless")]
    pub(crate) async fn read(headers: &HeaderMap, body: ReqBody) -> Result<FormData, ParseError> {
        let c_type: Option<Mime> = headers
            .get(CONTENT_TYPE)
            .and_then(|h| h.to_str().ok())
            .and_then(|v| v.parse().ok());
        match c_type {
            Some(c_type) if c_type.subtype() == mime::WWW_FORM_URLENCODED => {
                let data = BodyExt::collect(body).await.map_err(ParseError::other)?.to_bytes();
                let mut form_data = FormData::new();
                form_data.fields = form_urlencoded::parse(&data).into_owned().collect();
                Ok(form_data)
            }
            Some(c_type) if c_type.type_() == mime::MULTIPART => {
                let mut form_data = FormData::new();
                if let Some(boundary) = headers
                    .get(CONTENT_TYPE)
                    .and_then(|ct| ct.to_str().ok())
                    .and_then(|ct| multer::parse_boundary(ct).ok())
                {
                    let body = body.map(|f| f.map(|f| f.into_data().unwrap_or_default()));
                    let mut multipart = Multipart::new(body, boundary);
                    while let Some(mut field) = multipart.next_field().await? {
                        if let Some(name) = field.name().map(|s| s.to_owned()) {
                            if field.headers().get(CONTENT_TYPE).is_some() {
                                form_data.files.insert(name, FilePart::create(&mut field).await?);
                            } else {
                                form_data.fields.insert(name, field.text().await?);
                            }
                        }
                    }
                }
                Ok(form_data)
            }
            _ => Err(ParseError::InvalidContentType),
        }
    }
}
impl Default for FormData {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
/// A file that is to be inserted into a `multipart/*` or alternatively an uploaded file that
/// was received as part of `multipart/*` parsing.
#[derive(Clone, Debug)]
pub struct FilePart {
    name: Option<String>,
    /// The headers of the part
    headers: HeaderMap,
    /// A temporary file containing the file content
    path: PathBuf,
    /// Optionally, the size of the file.  This is filled when multi-parts are parsed, but is
    /// not necessary when they are generated.
    size: u64,
    // The temporary directory the upload was put into, saved for the Drop trait
    temp_dir: Option<PathBuf>,
}
impl FilePart {
    /// Get file name.
    #[inline]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
    /// Get file name mutable reference.
    #[inline]
    pub fn name_mut(&mut self) -> Option<&mut String> {
        self.name.as_mut()
    }
    /// Get headers.
    #[inline]
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }
    /// Get headers mutable reference.
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.headers
    }
    /// Get content type.
    #[inline]
    pub fn content_type(&self) -> Option<Mime> {
        self.headers
            .get(CONTENT_TYPE)
            .and_then(|h| h.to_str().ok())
            .and_then(|v| v.parse().ok())
    }
    /// Get file path.
    #[inline]
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
    /// Get file size.
    #[inline]
    pub fn size(&self) -> u64 {
        self.size
    }
    /// If you do not want the file on disk to be deleted when Self drops, call this
    /// function.  It will become your responsibility to clean up.
    #[inline]
    pub fn do_not_delete_on_drop(&mut self) {
        self.temp_dir = None;
    }

    /// Create a new temporary FilePart (when created this way, the file will be
    /// deleted once the FilePart object goes out of scope).
    #[cfg(feature = "needless")]
    pub async fn create(field: &mut Field<'_>) -> Result<FilePart, ParseError> {
        // Setup a file to capture the contents.
        let mut path = tokio::task::spawn_blocking(|| Builder::new().prefix("salvo_http_multipart").tempdir())
            .await
            .expect("Runtime spawn blocking poll error")?
            .into_path();
        let temp_dir = Some(path.clone());
        let name = field.file_name().map(|s| s.to_owned());
        path.push(format!(
            "{}.{}",
            text_nonce(),
            name.as_deref()
                .and_then(|name| { Path::new(name).extension().and_then(OsStr::to_str) })
                .unwrap_or("unknown")
        ));
        let mut file = File::create(&path).await?;
        let mut size = 0;
        while let Some(chunk) = field.chunk().await? {
            size += chunk.len() as u64;
            file.write_all(&chunk).await?;
        }
        file.sync_all().await?;
        Ok(FilePart {
            name,
            headers: field.headers().to_owned(),
            path,
            size,
            temp_dir,
        })
    }
}
#[cfg(feature = "needless")]
impl Drop for FilePart {
    fn drop(&mut self) {
        if let Some(temp_dir) = &self.temp_dir {
            let path = self.path.clone();
            let temp_dir = temp_dir.to_owned();
            tokio::task::spawn_blocking(move || {
                let _ = std::fs::remove_file(&path);
                let _ = std::fs::remove_dir(temp_dir);
            });
        }
    }
}

// Port from https://github.com/mikedilger/textnonce/blob/master/src/lib.rs
#[cfg(feature = "needless")]
fn text_nonce() -> String {
    const BYTE_LEN: usize = 24;
    let mut raw: Vec<u8> = vec![0; BYTE_LEN];

    // Get the first 12 bytes from the current time
    if let Ok(now) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        let secs: u64 = now.as_secs();
        let ns: u32 = now.subsec_nanos();

        let mut cursor = Cursor::new(&mut *raw);
        Write::write_all(&mut cursor, &ns.to_le_bytes()).expect("write_all failed");
        Write::write_all(&mut cursor, &secs.to_le_bytes()).expect("write_all failed");

        // Get the last bytes from random data
        OsRng
            .try_fill_bytes(&mut raw[12..BYTE_LEN])
            .expect("OsRng.try_fill_bytes failed");
    } else {
        OsRng.try_fill_bytes(&mut raw[..]).expect("OsRng.try_fill_bytes failed");
    }

    // base64 encode
    URL_SAFE_NO_PAD.encode(&raw)
}
