use std::fs;
use std::path::Path;

use jiff::fmt::rfc2822;

use crate::error::{AppError, Result};
use crate::posts::{Post, render_to_html};

pub fn write(output_path: &Path, public_site_url: &str, posts: &[Post]) -> Result<()> {
    let feed_contents = render(public_site_url, posts)?;
    let feed_path = output_path.join("rss.xml");
    fs::write(&feed_path, feed_contents).map_err(|error| {
        AppError::Output(format!(
            "failed to write RSS feed {}: {error}",
            feed_path.display()
        ))
    })
}

fn render(public_site_url: &str, posts: &[Post]) -> Result<String> {
    let site_url = public_site_url.trim_end_matches('/');
    let feed_url = format!("{site_url}/rss.xml");
    let mut feed_items = String::new();

    for post in posts {
        let post_url = format!("{site_url}/{}/", post.slug);
        let publication_timestamp =
            rfc2822::to_string(&post.publication_timestamp).map_err(|error| {
                AppError::Post(format!(
                    "failed to format publication timestamp for post {}: {error}",
                    post.source_path.display()
                ))
            })?;
        let post_html = escape_xml(&render_to_html(post));
        feed_items.push_str(&format!(
            "<item>\n<title>{}</title>\n<link>{post_url}</link>\n<guid isPermaLink=\"true\">{post_url}</guid>\n<pubDate>{publication_timestamp}</pubDate>\n<description>{post_html}</description>\n</item>\n",
            escape_xml(&post.title),
        ));
    }

    Ok(format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<rss version=\"2.0\" xmlns:atom=\"http://www.w3.org/2005/Atom\">\n<channel>\n<title>Posts</title>\n<link>{site_url}/</link>\n<description>Posts</description>\n<atom:link href=\"{feed_url}\" rel=\"self\" type=\"application/rss+xml\" />\n{feed_items}</channel>\n</rss>\n"
    ))
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
