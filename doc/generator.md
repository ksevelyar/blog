# Generator
1. Scans `/posts` and builds a list of Markdown posts.
2. Parses every post. Parsing fails if a post is invalid, for example if it has no level-one heading.
3. Renders every parsed post as HTML.
4. When a post contains a Wasm component shortcode, replaces it with the component mount point and compiles the component from Rust to WebAssembly.
5. Inserts the rendered list of posts into `index.html`, writes it to `/dist`, and writes each rendered post to its own page.
6. Generates `/rss.xml`.
7. Copies repository assets to `/dist/assets`.
