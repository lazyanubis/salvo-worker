use std::borrow::Cow;

use super::*;

use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

/// 更新 open-api.json
#[allow(clippy::expect_used)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
pub fn update_open_api(router: salvo::Router, title: &str, version: &str, path: &str) {
    let doc = super::salvo::oapi::OpenApi::new(title, version).merge_router(&router);
    let open_api = doc.to_json().unwrap();
    // println!("{}", open_api);

    let _ = fs::remove_file(path);
    File::create(path)
        .unwrap_or_else(|_| panic!("create {path} file failed"))
        .write_all(open_api.as_bytes())
        .unwrap_or_else(|_| panic!("write {path} failed"));
}

/// 遍历所有文件
fn traverse_dir(path: &Path) -> Vec<String> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend_from_slice(&traverse_dir(&path));
            } else if let Some(path) = path.to_str() {
                if path.ends_with(".rs") {
                    files.push(path.to_string());
                }
            }
        }
    }
    files
}

// 替换单个文件
#[allow(clippy::panic)]
fn release_endpoint(path: &str) -> Result<(), std::io::Error> {
    let mut lines = Vec::new();

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    for line in reader.lines() {
        lines.push(line?);
    }

    let mut replaced = false;

    for i in 0..lines.len() {
        if lines[i].starts_with("// #[endpoint]")
            || (lines[i].starts_with("// #[endpoint(") && lines[i].starts_with(")]"))
        {
            lines[i] = lines[i].trim_start_matches("// ").to_string();
            lines[i + 1] = format!("// {}", lines[i + 1]);
            replaced = true;
        }
        if lines[i].starts_with("// #[endpoint(") {
            let mut j = i;
            loop {
                lines[j] = lines[j].trim_start_matches("// ").to_string();
                if lines[j].ends_with(")]") {
                    lines[j + 1] = format!("// {}", lines[j + 1]);
                    break;
                }
                j += 1;
                if lines.len() <= j {
                    panic!("error: {path}:{j}");
                }
            }
            replaced = true;
        }
    }
    if replaced {
        let _ = fs::remove_file(path);
        File::create(path)?.write_all(lines.join("\n").as_bytes())?;
    }
    Ok(())
}

// 替换单个文件
#[allow(clippy::panic)]
fn release_handler(path: &str) -> Result<(), std::io::Error> {
    let mut lines = Vec::new();

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    for line in reader.lines() {
        lines.push(line?);
    }

    let mut replaced = false;
    for i in 0..lines.len() {
        if lines[i].starts_with("#[endpoint]") || (lines[i].starts_with("#[endpoint(") && lines[i].starts_with(")]")) {
            lines[i + 1] = lines[i + 1].trim_start_matches("// ").to_string();
            lines[i] = format!("// {}", lines[i]);
            replaced = true;
        }
        if lines[i].starts_with("#[endpoint(") {
            let mut j = i;
            loop {
                lines[j] = format!("// {}", lines[j]);
                if lines[j].ends_with("// )]") {
                    lines[j + 1] = lines[j + 1].trim_start_matches("// ").to_string();
                    break;
                }
                j += 1;
                if lines.len() <= j {
                    panic!("error: {path}:{j}");
                }
            }
            replaced = true;
        }
    }
    if replaced {
        let _ = fs::remove_file(path);
        File::create(path)?.write_all(lines.join("\n").as_bytes())?;
    }
    Ok(())
}

/// 使用 endpoint
#[allow(clippy::unwrap_used)]
pub fn release_all_endpoints(root: &str) {
    let paths = traverse_dir(Path::new(root));
    for path in paths.iter() {
        release_endpoint(path).unwrap();
    }
}

/// 使用 handler
#[allow(clippy::unwrap_used)]
pub fn release_all_handlers(root: &str) {
    let paths = traverse_dir(Path::new(root));
    for path in paths.iter() {
        release_handler(path).unwrap();
    }
}

/// openapi ui swagger
pub fn ui_swagger(
    data: &'static str,
    path: impl Into<String>,
    favicon: impl Into<Cow<'static, str>>,
) -> super::salvo::Router {
    super::salvo::oapi::swagger_ui::SwaggerUi::new(data)
        .favicon_url(favicon)
        .into_router(path)
}

/// openapi ui rapidoc
pub fn ui_rapidoc(
    data: &'static str,
    path: impl Into<String>,
    favicon: impl Into<Cow<'static, str>>,
) -> super::salvo::Router {
    super::salvo::oapi::rapidoc::RapiDoc::new(data)
        .favicon_url(favicon)
        .into_router(path)
}

/// openapi ui redoc
pub fn ui_redoc(
    data: &'static str,
    path: impl Into<String>,
    favicon: impl Into<Cow<'static, str>>,
) -> super::salvo::Router {
    super::salvo::oapi::redoc::ReDoc::new(data)
        .favicon_url(favicon)
        .into_router(path)
}

/// openapi ui scalar
pub fn ui_scalar(
    data: &'static str,
    path: impl Into<String>,
    favicon: impl Into<Cow<'static, str>>,
) -> super::salvo::Router {
    super::salvo::oapi::scalar::Scalar::new(data)
        .favicon_url(favicon)
        .into_router(path)
}

/// openapi ui all
pub fn ui_all(data: &'static str) -> Vec<super::salvo::Router> {
    vec![
        ui_swagger(
            data,
            "/swagger-ui",
            "https://static1.smartbear.co/swagger/media/assets/swagger_fav.png",
        ),
        ui_rapidoc(data, "/rapidoc", "https://rapidocweb.com/images/logo.png"),
        ui_redoc(
            data,
            "/redoc",
            "https://redocly.com/assets/favicon.5465de6f2fccf1152f5965257241d831a8917043c85baf743706ae2e57736946.8a5edab2.ico",
        ),
        ui_scalar(
            data,
            "/scalar",
            // "https://docs.scalar.com/favicon.svg"
            "https://avatars.githubusercontent.com/u/301879",
        ),
    ]
}
