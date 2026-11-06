# Markdown
## Requirements
* The generator renders post Markdown as HTML.
* A post can embed a Wasm component with a shortcode.
* The circle component shortcode is `{{< wasm-circle >}}`.

## Implementation
* The generator renders post bodies after parsing their front matter.
* The generator expands the circle shortcode into a component mount element.
* Generated pages that contain a component mount element load the component bootstrap asset.

## User scenarios
* Given a post with a Markdown heading and paragraph, when its generated page is opened, then the heading and paragraph are rendered as HTML.
* Given a post with a Markdown link, when its generated page is opened, then the link is rendered as an HTML link.
* Given a post containing `{{< wasm-circle >}}`, when its generated page is opened, then the page contains the circle component mount element.
