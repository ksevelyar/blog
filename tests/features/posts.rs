use super::support::TemporarySite;

#[test]
fn generates_yearless_post_urls_and_formatted_timestamps() {
    let temporary_site = TemporarySite::create("post-pages");
    temporary_site.write_post(
        "2026-wasm-circle.md",
        "---\ndate: Tue, 31 Mar 2026 14:30:00 +0300\n---\n\n# Wasm circle\n\nA paragraph in the post.\n",
    );
    temporary_site.write_post(
        "older-post.md",
        "---\ndate: Sat, 01 Mar 2025 08:15:00 +0300\n---\n\n# Older post\n",
    );

    let generation_output = temporary_site.run_generator();
    assert!(
        generation_output.status.success(),
        "site generation failed: {}",
        String::from_utf8_lossy(&generation_output.stderr)
    );

    let generated_index = temporary_site.read_generated_file("index.html");
    let yearless_post_link_position = generated_index
        .find("href=\"/wasm-circle/\"")
        .expect("the generated index should link to the yearless post URL");
    let older_post_link_position = generated_index
        .find("href=\"/older-post/\"")
        .expect("the generated index should link to the older post URL");
    assert!(yearless_post_link_position < older_post_link_position);
    assert!(generated_index.contains("31-03-26 14:30"));
    assert!(generated_index.contains("<title>Posts</title>"));
    assert!(generated_index.contains("🐗"));
    assert!(!generated_index.contains("<main>"));
    assert!(!generated_index.contains("<ul>"));
    assert!(!generated_index.contains("<li>"));
    assert!(!generated_index.contains("<time>"));
    assert!(!temporary_site.generated_file_exists("assets/index.html"));
    assert!(temporary_site.generated_file_exists("wasm-circle/index.html"));
    assert!(!temporary_site.generated_file_exists("2026-wasm-circle/index.html"));

    let generated_post = temporary_site.read_generated_file("wasm-circle/index.html");
    assert!(generated_post.contains("<title>Wasm circle</title>"));
    assert!(generated_post.contains("<h1>Wasm circle</h1>"));
    assert!(generated_post.contains("<span class=post__date>31-03-26 14:30</span>"));
    assert!(generated_post.contains("A paragraph in the post."));
    assert!(generated_post.contains("🐗"));
    assert!(!generated_post.contains("<main>"));
    assert!(!generated_post.contains("<ul>"));
    assert!(!generated_post.contains("<li>"));
    assert!(!generated_post.contains("<time>"));

    let generated_stylesheet = temporary_site.read_generated_file("assets/blog.css");
    assert!(generated_stylesheet.contains("min(80ch"));
}

#[test]
fn rejects_posts_with_duplicate_generated_slugs() {
    let temporary_site = TemporarySite::create("duplicate-post-slugs");
    temporary_site.write_post(
        "2025-circle.md",
        "---\ndate: Thu, 27 Mar 2025 08:00:00 +0300\n---\n\n# Earlier circle\n",
    );
    temporary_site.write_post(
        "2026-circle.md",
        "---\ndate: Fri, 27 Mar 2026 08:00:00 +0300\n---\n\n# Later circle\n",
    );

    let generation_output = temporary_site.run_generator();

    assert!(!generation_output.status.success());
    assert!(
        String::from_utf8_lossy(&generation_output.stderr).contains("duplicate post slug 'circle'")
    );
}

#[test]
fn rejects_posts_without_publication_dates() {
    let temporary_site = TemporarySite::create("missing-publication-date");
    temporary_site.write_post("missing-date.md", "---\n{}\n---\n\n# Missing date\n");

    let generation_output = temporary_site.run_generator();

    assert!(!generation_output.status.success());
    assert!(
        String::from_utf8_lossy(&generation_output.stderr).contains("missing required date field")
    );
}

#[test]
fn rejects_posts_with_invalid_publication_dates() {
    let temporary_site = TemporarySite::create("invalid-publication-date");
    temporary_site.write_post(
        "invalid-date.md",
        "---\ndate: Mon, 30 Feb 2026 09:00:00 +0300\n---\n\n# Invalid date\n",
    );

    let generation_output = temporary_site.run_generator();

    assert!(!generation_output.status.success());
    assert!(
        String::from_utf8_lossy(&generation_output.stderr)
            .contains("invalid publication timestamp")
    );
}

#[test]
fn panics_for_posts_without_titles() {
    let temporary_site = TemporarySite::create("missing-post-title");
    temporary_site.write_post(
        "missing-title.md",
        "---\ndate: Tue, 31 Mar 2026 14:30:00 +0300\n---\n\nA post without a heading.\n",
    );

    let generation_output = temporary_site.run_generator();

    assert_eq!(generation_output.status.code(), Some(101));
    assert!(String::from_utf8_lossy(&generation_output.stderr).contains("missing-title.md"));
}
