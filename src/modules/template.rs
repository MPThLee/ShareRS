use crate::built_info;
use anyhow::{anyhow, Context, Result};
use lazy_static::lazy_static;
use tera::{Context as TeraContext, Tera};

static INDEX: &str = r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <title>Index</title>
  </head>
  <body>
    <p>
      This server is running SHARERS v{{build_version}},
      {{build_git_commit_short}} (<code>{{build_git_ver}}</code>,
      <code>{{build_profile}}</code>). Built on <code>{{build_time}}</code>
    </p>
    <p>Currently storing {{total_files}} files and {{total_urls}} urls.</p>
  </body>
</html>
"#;

lazy_static! {
    static ref BUILD_TIME_UTC_RFC3339: String =
        chrono::DateTime::parse_from_rfc2822(built_info::BUILT_TIME_UTC)
            .unwrap()
            .with_timezone(&chrono::offset::Utc)
            .to_rfc3339();
    static ref GIT_DIRTY_VERSION: String = format!(
        "{}{}",
        built_info::GIT_VERSION.unwrap_or("unknown"),
        if built_info::GIT_DIRTY.unwrap_or(false) {
            "+"
        } else {
            ""
        }
    );
}

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
    let mut context = TeraContext::new();
    context.insert("build_version", built_info::PKG_VERSION);
    context.insert("build_time", BUILD_TIME_UTC_RFC3339.as_str());
    context.insert("build_git_ver", GIT_DIRTY_VERSION.as_str());
    context.insert(
        "build_git_version",
        built_info::GIT_VERSION.unwrap_or("Unknown"),
    );
    context.insert(
        "build_git_commit_short",
        built_info::GIT_COMMIT_HASH_SHORT.unwrap_or("Unknown"),
    );
    context.insert("build_profile", built_info::PROFILE);

    context
}
