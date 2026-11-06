use std::fs;

use super::support::TemporarySite;

#[test]
fn builds_and_embeds_circle_wasm_artifacts() {
    let temporary_site = TemporarySite::create("wasm-component");
    temporary_site.write_post(
        "2026-wasm-circle.md",
        "---\ndate: Tue, 31 Mar 2026 14:30:00 +0300\n---\n\n# Wasm circle\n\n{{< wasm-circle >}}\n",
    );

    let generation_output = temporary_site.run_generator();
    assert!(
        generation_output.status.success(),
        "site generation failed: {}",
        String::from_utf8_lossy(&generation_output.stderr)
    );

    let generated_post = temporary_site.read_generated_file("wasm-circle/index.html");
    assert!(generated_post.contains("<title>Wasm circle</title>"));
    assert!(generated_post.contains("<span class=post__date>31-03-26 14:30</span>"));
    assert!(generated_post.contains("data-wasm-component=\"circle\""));
    assert!(generated_post.contains("/assets/wasm/circle/bootstrap.js"));
    assert!(temporary_site.generated_file_exists("assets/wasm/circle/circle.js"));

    let generated_bootstrap = temporary_site.read_generated_file("assets/wasm/circle/bootstrap.js");
    assert!(generated_bootstrap.contains("import initialize, { mount } from \"./circle.js\""));
    assert!(generated_bootstrap.contains("data-wasm-component=\"circle\""));

    let generated_wasm_binary = fs::read(
        temporary_site
            .output_directory
            .join("assets/wasm/circle/circle_bg.wasm"),
    )
    .expect("the generated Wasm binary should be readable");
    assert!(generated_wasm_binary.starts_with(b"\0asm"));
}
