use super::support::TemporarySite;

#[test]
fn generates_an_rss_feed_with_posts_in_publication_order() {
    let temporary_site = TemporarySite::create("rss-feed");
    temporary_site.write_post(
        "newer-post.md",
        "---\ndate: Tue, 31 Mar 2026 14:30:00 +0300\n---\n\n# Newer post\n\nNewer post body.\n",
    );
    temporary_site.write_post(
        "older-post.md",
        "---\ndate: Sat, 01 Mar 2025 08:15:00 +0300\n---\n\n# Older post\n\nOlder post body.\n",
    );

    let generation_output = temporary_site.run_generator();
    assert!(
        generation_output.status.success(),
        "site generation failed: {}",
        String::from_utf8_lossy(&generation_output.stderr)
    );

    assert!(temporary_site.generated_file_exists("rss.xml"));
    let generated_feed = temporary_site.read_generated_file("rss.xml");
    let newer_post_position = generated_feed
        .find("<title>Newer post</title>")
        .expect("the RSS feed should include the newer post");
    let older_post_position = generated_feed
        .find("<title>Older post</title>")
        .expect("the RSS feed should include the older post");
    assert!(newer_post_position < older_post_position);
    assert!(
        generated_feed.contains("<link>https://ksevelyar.rusty-cluster.net/newer-post/</link>")
    );
    assert!(generated_feed.contains(
        "<guid isPermaLink=\"true\">https://ksevelyar.rusty-cluster.net/newer-post/</guid>"
    ));
    assert!(generated_feed.contains("<pubDate>Tue, 31 Mar 2026 14:30:00 +0300</pubDate>"));
    assert!(generated_feed.contains("<description>&lt;h1&gt;Newer post&lt;/h1&gt;\n&lt;p&gt;Newer post body.&lt;/p&gt;\n</description>"));
    assert!(
        generated_feed.contains("<link>https://ksevelyar.rusty-cluster.net/older-post/</link>")
    );
    assert!(generated_feed.contains(
        "<guid isPermaLink=\"true\">https://ksevelyar.rusty-cluster.net/older-post/</guid>"
    ));
    assert!(generated_feed.contains("<pubDate>Sat, 1 Mar 2025 08:15:00 +0300</pubDate>"));
    assert!(generated_feed.contains("<description>&lt;h1&gt;Older post&lt;/h1&gt;\n&lt;p&gt;Older post body.&lt;/p&gt;\n</description>"));
}
