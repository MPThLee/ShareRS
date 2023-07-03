use anyhow::{anyhow, Context, Result};
use tera::{Context as TeraContext, Tera};

static INDEX: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Index</title>
</head>
<body>
  <p>This server is running SHARERS. (v{{version}}, {{git_version}}, {{built_time}})</p>
  <p>Currently storing {{total_files}} files and {{total_urls}} urls.</p>
</body>
</html>"#;

pub fn init_template() -> anyhow::Result<Tera> {
    let mut tera = Tera::default();

    tera.add_raw_template("index.html", INDEX)?;

    //  this is cursed as hell...
    let files = glob::glob("template/*.html")?
        .filter_map(Result::ok)
        .map(|p| -> anyhow::Result<(String, Option<String>)> {
            let path = p
                .canonicalize()?
                .into_os_string()
                .into_string()
                .map_err(|o| anyhow!("Failed to parse OsString path: {:?}", o))?;
            let name = p
                .file_name()
                .context("Failed to get filename")?
                .to_os_string()
                .into_string()
                .map_err(|o| anyhow!("Failed to parse OsString file name: {:?}", o))?;

            Ok((path, Some(name)))
        })
        .filter_map(Result::ok);
    tera.add_template_files(files)?;

    Ok(tera)
}

pub fn default_context() -> TeraContext {
    use crate::built_info;
    let mut context = TeraContext::new();
    context.insert("version", built_info::PKG_VERSION);

    let built_time = chrono::DateTime::parse_from_rfc2822(built_info::BUILT_TIME_UTC)
        .unwrap()
        .with_timezone(&chrono::offset::Utc);
    context.insert("built_time", &built_time.to_rfc3339());

    context.insert(
        "git_version",
        &format!(
            "{}{}",
            built_info::GIT_VERSION.unwrap_or("unknown"),
            if built_info::GIT_DIRTY.unwrap_or(false) {
                "+"
            } else {
                ""
            }
        ),
    );

    context
}
