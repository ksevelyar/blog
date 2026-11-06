use std::fs;
use std::path::{Path, PathBuf};

use jiff::Zoned;
use jiff::fmt::rfc2822;
use pulldown_cmark::{Options, Parser, html};
use serde::Deserialize;

use crate::error::{AppError, Result};

#[derive(Debug)]
pub struct Post {
    pub source_path: PathBuf,
    pub slug: String,
    pub title: String,
    pub publication_timestamp: Zoned,
    pub markdown_body: String,
}

#[derive(Deserialize)]
struct PostMetadata {
    date: Option<String>,
}

pub fn parse(source_path: &Path) -> Result<Post> {
    let document = fs::read_to_string(source_path).map_err(|error| {
        AppError::Post(format!(
            "failed to read post {}: {error}",
            source_path.display()
        ))
    })?;
    let (metadata_contents, markdown_body) = parse_front_matter(&document, source_path)?;
    let metadata: PostMetadata = serde_yaml::from_str(metadata_contents).map_err(|error| {
        AppError::Post(format!(
            "invalid YAML front matter in {}: {error}",
            source_path.display()
        ))
    })?;
    let timestamp_text = metadata.date.ok_or_else(|| {
        AppError::Post(format!(
            "missing required date field in post {}",
            source_path.display()
        ))
    })?;
    let publication_timestamp = rfc2822::parse(&timestamp_text).map_err(|_| {
        AppError::Post(format!(
            "invalid publication timestamp '{}' in post {}: expected RFC 822 with a timezone offset",
            timestamp_text,
            source_path.display()
        ))
    })?;

    let slug = create_slug(source_path);
    let title = find_post_title(markdown_body).unwrap_or_else(|| {
        panic!(
            "post {} is missing a required title: add a nonempty level-one Markdown heading",
            source_path.display()
        )
    });

    Ok(Post {
        source_path: source_path.to_path_buf(),
        slug,
        title,
        publication_timestamp,
        markdown_body: markdown_body.to_owned(),
    })
}

pub fn render_to_html(post: &Post) -> String {
    let mut markdown_options = Options::empty();
    markdown_options.insert(Options::ENABLE_TABLES);
    markdown_options.insert(Options::ENABLE_STRIKETHROUGH);
    markdown_options.insert(Options::ENABLE_TASKLISTS);

    let circle_shortcode = "{{< wasm-circle >}}";
    let rendered_markdown_body = post.markdown_body.replace(
        circle_shortcode,
        "<div data-wasm-component=\"circle\"></div>",
    );
    let markdown_parser = Parser::new_ext(&rendered_markdown_body, markdown_options);
    let mut rendered_html = String::new();
    html::push_html(&mut rendered_html, markdown_parser);
    rendered_html
}

fn parse_front_matter<'document>(
    document: &'document str,
    source_path: &Path,
) -> Result<(&'document str, &'document str)> {
    let Some(first_line_end) = document.find('\n') else {
        return Err(AppError::Post(format!(
            "missing YAML front matter in post {}",
            source_path.display()
        )));
    };
    let first_line = document[..first_line_end].trim_end_matches('\r');
    if first_line != "---" {
        return Err(AppError::Post(format!(
            "missing YAML front matter in post {}",
            source_path.display()
        )));
    }

    let metadata_start = first_line_end + 1;
    let mut line_start = metadata_start;
    for line in document[metadata_start..].split_inclusive('\n') {
        let line_contents = line.trim_end_matches(['\r', '\n']);
        if line_contents == "---" {
            let metadata_contents = &document[metadata_start..line_start];
            let markdown_body = &document[line_start + line.len()..];
            return Ok((metadata_contents, markdown_body));
        }
        line_start += line.len();
    }

    Err(AppError::Post(format!(
        "unterminated YAML front matter in post {}",
        source_path.display()
    )))
}

fn create_slug(source_path: &Path) -> String {
    let file_stem = source_path
        .file_stem()
        .and_then(|file_stem| file_stem.to_str())
        .unwrap_or("post");
    let slug_source = file_stem
        .split_once('-')
        .filter(|(year_prefix, remaining_slug)| {
            year_prefix.len() == 4
                && year_prefix
                    .chars()
                    .all(|character| character.is_ascii_digit())
                && !remaining_slug.is_empty()
        })
        .map_or(file_stem, |(_, remaining_slug)| remaining_slug);
    let mut slug = String::new();
    let mut previous_character_was_separator = true;

    for character in slug_source.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
            previous_character_was_separator = false;
        } else if !previous_character_was_separator {
            slug.push('-');
            previous_character_was_separator = true;
        }
    }

    while slug.ends_with('-') {
        slug.pop();
    }

    if slug.is_empty() {
        "post".to_owned()
    } else {
        slug
    }
}

pub fn contains_wasm_components(post: &Post) -> bool {
    post.markdown_body.contains("{{< wasm-circle >}}")
}

fn find_post_title(markdown_body: &str) -> Option<String> {
    markdown_body.lines().find_map(|line| {
        let heading = line.trim_start().strip_prefix('#')?;
        if !heading.starts_with(' ') {
            return None;
        }
        let title = heading.trim().trim_start_matches('#').trim();
        (!title.is_empty()).then(|| title.to_owned())
    })
}
