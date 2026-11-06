use std::collections::HashSet;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use jiff::Zoned;

use crate::error::{AppError, Result};
use crate::posts::{Post, contains_wasm_components, parse, render_to_html};
use crate::rss;
use crate::wasm_components::{WasmComponent, build_circle_component};

pub fn generate_site(source_path: &Path, output_path: &Path, public_site_url: &str) -> Result<()> {
    let source_path = fs::canonicalize(source_path).map_err(|error| {
        AppError::Configuration(format!(
            "failed to locate post source {}: {error}",
            source_path.display()
        ))
    })?;
    let output_path = resolve_path(output_path)?;
    if source_path.starts_with(&output_path) || output_path.starts_with(&source_path) {
        return Err(AppError::Configuration(format!(
            "source and output directories must not contain one another: source {}, output {}",
            source_path.display(),
            output_path.display()
        )));
    }

    let posts = discover_posts(&source_path)?;
    let wasm_component = if posts.iter().any(contains_wasm_components) {
        Some(build_circle_component()?)
    } else {
        None
    };

    write_site(
        &source_path,
        &output_path,
        &posts,
        wasm_component.as_ref(),
        public_site_url,
    )
}

fn discover_posts(source_path: &Path) -> Result<Vec<Post>> {
    if !source_path.is_dir() {
        return Err(AppError::Source(format!(
            "post source directory does not exist: {}",
            source_path.display()
        )));
    }

    let mut markdown_paths = Vec::new();
    collect_markdown_paths(
        source_path,
        &source_path.join("static"),
        &mut markdown_paths,
    )?;
    markdown_paths.sort();

    let mut posts = markdown_paths
        .into_iter()
        .map(|path| parse(&path))
        .collect::<Result<Vec<_>>>()?;
    posts.sort_by(|left_post, right_post| {
        right_post
            .publication_timestamp
            .cmp(&left_post.publication_timestamp)
            .then_with(|| left_post.slug.cmp(&right_post.slug))
    });

    let mut used_slugs = HashSet::new();
    for post in &posts {
        if !used_slugs.insert(post.slug.clone()) {
            return Err(AppError::Post(format!(
                "duplicate post slug '{}' from {}",
                post.slug,
                post.source_path.display()
            )));
        }
    }

    Ok(posts)
}

fn collect_markdown_paths(
    directory_path: &Path,
    static_directory_path: &Path,
    markdown_paths: &mut Vec<PathBuf>,
) -> Result<()> {
    let directory_entries = fs::read_dir(directory_path).map_err(|error| {
        AppError::Source(format!(
            "failed to read directory {}: {error}",
            directory_path.display()
        ))
    })?;
    for entry in directory_entries {
        let entry = entry.map_err(|error| {
            AppError::Source(format!(
                "failed to read an entry in {}: {error}",
                directory_path.display()
            ))
        })?;
        let entry_path = entry.path();
        let entry_type = entry.file_type().map_err(|error| {
            AppError::Source(format!(
                "failed to inspect source path {}: {error}",
                entry_path.display()
            ))
        })?;

        if entry_type.is_dir() {
            if entry_path != static_directory_path {
                collect_markdown_paths(&entry_path, static_directory_path, markdown_paths)?;
            }
        } else if entry_type.is_file()
            && entry_path
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
        {
            markdown_paths.push(entry_path);
        }
    }

    Ok(())
}

fn resolve_path(path: &Path) -> Result<PathBuf> {
    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| {
                AppError::Configuration(format!(
                    "failed to determine the current directory: {error}"
                ))
            })?
            .join(path)
    };
    let mut existing_path = absolute_path.clone();
    let mut missing_path_components: Vec<OsString> = Vec::new();

    while !existing_path.exists() {
        let component = existing_path
            .file_name()
            .ok_or_else(|| {
                AppError::Configuration("output path has no existing ancestor".to_owned())
            })?
            .to_os_string();
        missing_path_components.push(component);
        existing_path = existing_path
            .parent()
            .ok_or_else(|| {
                AppError::Configuration("output path has no existing ancestor".to_owned())
            })?
            .to_path_buf();
    }

    let mut resolved_path = fs::canonicalize(existing_path).map_err(|error| {
        AppError::Configuration(format!(
            "failed to resolve output path {}: {error}",
            path.display()
        ))
    })?;
    for component in missing_path_components.iter().rev() {
        resolved_path.push(component);
    }
    Ok(resolved_path)
}

fn write_site(
    source_path: &Path,
    output_path: &Path,
    posts: &[Post],
    wasm_component: Option<&WasmComponent>,
    public_site_url: &str,
) -> Result<()> {
    clear_output_directory(output_path)?;
    fs::create_dir_all(output_path).map_err(|error| {
        AppError::Output(format!(
            "failed to create output directory {}: {error}",
            output_path.display()
        ))
    })?;
    let repository_assets_directory_path =
        source_path.parent().unwrap_or(source_path).join("assets");
    let output_assets_directory_path = output_path.join("assets");
    copy_assets(
        &repository_assets_directory_path,
        &output_assets_directory_path,
    )?;
    let index_template_path = repository_assets_directory_path.join("index.html");
    let index_template_contents = fs::read_to_string(&index_template_path).map_err(|error| {
        AppError::Source(format!(
            "failed to read index template {}: {error}",
            index_template_path.display()
        ))
    })?;

    for post in posts {
        write_post_page(
            output_path,
            &index_template_path,
            &index_template_contents,
            post,
        )?;
    }
    write_post_index(
        output_path,
        &index_template_path,
        &index_template_contents,
        posts,
    )?;
    rss::write(output_path, public_site_url, posts)?;

    if posts.iter().any(contains_wasm_components) {
        let wasm_component = wasm_component.ok_or_else(|| {
            AppError::Component(
                "Wasm component is required for pages containing component mounts".to_owned(),
            )
        })?;
        write_wasm_assets(output_path, wasm_component)?;
    }

    Ok(())
}

fn clear_output_directory(output_path: &Path) -> Result<()> {
    if output_path.is_dir() {
        fs::remove_dir_all(output_path).map_err(|error| {
            AppError::Output(format!(
                "failed to clear output directory {}: {error}",
                output_path.display()
            ))
        })?;
    } else if output_path.exists() {
        fs::remove_file(output_path).map_err(|error| {
            AppError::Output(format!(
                "failed to clear output path {}: {error}",
                output_path.display()
            ))
        })?;
    }
    Ok(())
}

fn copy_assets(assets_directory_path: &Path, output_assets_directory_path: &Path) -> Result<()> {
    if !assets_directory_path.is_dir() {
        return Ok(());
    }

    fs::create_dir_all(output_assets_directory_path).map_err(|error| {
        AppError::Output(format!(
            "failed to create assets output directory {}: {error}",
            output_assets_directory_path.display()
        ))
    })?;
    copy_directory_contents(
        assets_directory_path,
        output_assets_directory_path,
        &assets_directory_path.join("index.html"),
    )
}

fn copy_directory_contents(
    source_directory_path: &Path,
    output_directory_path: &Path,
    index_template_path: &Path,
) -> Result<()> {
    let directory_entries = fs::read_dir(source_directory_path).map_err(|error| {
        AppError::Source(format!(
            "failed to read assets directory {}: {error}",
            source_directory_path.display()
        ))
    })?;
    for entry in directory_entries {
        let entry = entry.map_err(|error| {
            AppError::Source(format!(
                "failed to read an entry in {}: {error}",
                source_directory_path.display()
            ))
        })?;
        let source_entry_path = entry.path();
        let output_entry_path = output_directory_path.join(entry.file_name());
        let entry_type = entry.file_type().map_err(|error| {
            AppError::Source(format!(
                "failed to inspect asset {}: {error}",
                source_entry_path.display()
            ))
        })?;

        if entry_type.is_dir() {
            fs::create_dir_all(&output_entry_path).map_err(|error| {
                AppError::Output(format!(
                    "failed to create assets output directory {}: {error}",
                    output_entry_path.display()
                ))
            })?;
            copy_directory_contents(&source_entry_path, &output_entry_path, index_template_path)?;
        } else if entry_type.is_file() && source_entry_path != index_template_path {
            fs::copy(&source_entry_path, &output_entry_path).map_err(|error| {
                AppError::Output(format!(
                    "failed to copy asset {} to {}: {error}",
                    source_entry_path.display(),
                    output_entry_path.display()
                ))
            })?;
        }
    }

    Ok(())
}

fn write_post_page(
    output_path: &Path,
    index_template_path: &Path,
    index_template_contents: &str,
    post: &Post,
) -> Result<()> {
    let post_directory_path = output_path.join(&post.slug);
    fs::create_dir_all(&post_directory_path).map_err(|error| {
        AppError::Output(format!(
            "failed to create post output directory {}: {error}",
            post_directory_path.display()
        ))
    })?;
    let post_page_path = post_directory_path.join("index.html");
    let formatted_publication_timestamp = format_publication_timestamp(&post.publication_timestamp);
    let rendered_post_body = render_to_html(post);
    let rendered_body_with_publication_timestamp = insert_publication_timestamp_after_heading(
        &rendered_post_body,
        &formatted_publication_timestamp,
    );
    let component_bootstrap = if contains_wasm_components(post) {
        "<script type=\"module\" src=\"/assets/wasm/circle/bootstrap.js\"></script>"
    } else {
        ""
    };
    let post_page_body = format!(
        "\n<div class=blog__content>\n<div class=post>\n{rendered_body_with_publication_timestamp}\n</div>\n</div>\n{component_bootstrap}\n"
    );
    write_page(
        &post_page_path,
        index_template_path,
        index_template_contents,
        &post.title,
        &post_page_body,
    )
}

fn format_publication_timestamp(publication_timestamp: &Zoned) -> String {
    publication_timestamp.strftime("%d-%m-%y %H:%M").to_string()
}

fn insert_publication_timestamp_after_heading(
    rendered_body: &str,
    formatted_publication_timestamp: &str,
) -> String {
    let publication_timestamp_element =
        format!("<span class=post__date>{formatted_publication_timestamp}</span>");

    if let Some((rendered_body_before_heading_end, rendered_body_after_heading_end)) =
        rendered_body.split_once("</h1>")
    {
        format!(
            "{rendered_body_before_heading_end}</h1>\n{publication_timestamp_element}{rendered_body_after_heading_end}"
        )
    } else {
        format!("{publication_timestamp_element}\n{rendered_body}")
    }
}

fn write_post_index(
    output_path: &Path,
    index_template_path: &Path,
    index_template_contents: &str,
    posts: &[Post],
) -> Result<()> {
    let mut post_links = String::new();
    for post in posts {
        let post_title = escape_html(&post.title);
        let formatted_publication_timestamp =
            format_publication_timestamp(&post.publication_timestamp);
        post_links.push_str(&format!(
            "<div class=post-list__item><a class=post-list__link href=\"/{}/\">{post_title}</a><span class=post-list__date>{formatted_publication_timestamp}</span></div>\n",
            escape_html(&post.slug)
        ));
    }
    let post_list_contents = format!(
        "\n<div class=blog__content>\n<div class=post-index__header><h1>Posts</h1><a class=post-index__rss href=\"/rss.xml\">RSS</a></div>\n<div class=post-list>\n{post_links}</div>\n</div>\n"
    );
    write_page(
        &output_path.join("index.html"),
        index_template_path,
        index_template_contents,
        "Posts",
        &post_list_contents,
    )
}

fn write_page(
    page_path: &Path,
    index_template_path: &Path,
    index_template_contents: &str,
    page_title: &str,
    page_body: &str,
) -> Result<()> {
    let page_title_placeholder = "{{page_title}}";
    let (template_before_page_title, template_after_page_title) = index_template_contents
        .split_once(page_title_placeholder)
        .ok_or_else(|| {
            AppError::Source(format!(
                "index template {} is missing its page-title placeholder",
                index_template_path.display()
            ))
        })?;
    let escaped_page_title = escape_html(page_title);
    let template_with_page_title =
        format!("{template_before_page_title}{escaped_page_title}{template_after_page_title}");
    let body_closing_tag = "</body>";
    let (template_before_body_close, template_after_body_close) = template_with_page_title
        .split_once(body_closing_tag)
        .ok_or_else(|| {
            AppError::Source(format!(
                "index template {} is missing its closing body tag",
                index_template_path.display()
            ))
        })?;
    let page_contents = format!(
        "{template_before_body_close}{page_body}{body_closing_tag}{template_after_body_close}"
    );
    write_file(page_path, &page_contents)
}

fn write_wasm_assets(output_path: &Path, wasm_component: &WasmComponent) -> Result<()> {
    let wasm_output_directory = output_path.join("assets/wasm/circle");
    fs::create_dir_all(&wasm_output_directory).map_err(|error| {
        AppError::Output(format!(
            "failed to create Wasm output directory {}: {error}",
            wasm_output_directory.display()
        ))
    })?;

    for (source_path, filename) in [
        (&wasm_component.javascript_bindings_path, "circle.js"),
        (&wasm_component.wasm_binary_path, "circle_bg.wasm"),
    ] {
        let output_file_path = wasm_output_directory.join(filename);
        fs::copy(source_path, &output_file_path).map_err(|error| {
            AppError::Output(format!(
                "failed to copy Wasm artifact {} to {}: {error}",
                source_path.display(),
                output_file_path.display()
            ))
        })?;
    }

    let bootstrap_module = "import initialize, { mount } from \"./circle.js\";\n\nawait initialize();\ndocument.querySelectorAll('[data-wasm-component=\"circle\"]').forEach((mount_element) => {\n    mount(mount_element);\n});\n";
    write_file(
        &wasm_output_directory.join("bootstrap.js"),
        bootstrap_module,
    )
}

fn write_file(file_path: &Path, contents: &str) -> Result<()> {
    fs::write(file_path, contents).map_err(|error| {
        AppError::Output(format!(
            "failed to write output file {}: {error}",
            file_path.display()
        ))
    })
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
