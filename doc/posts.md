# Posts
## Requirements
* A post is a Markdown document.
* Every post has a publication timestamp and title.
* Title should be unique
* Publication timestamps use `YYYY-MM-DDTHH:MM:SS`.
* Publication timestamps do not include a timezone offset.
* Posts are ordered newest first.
* A post with missing or invalid metadata is rejected.

## Implementation
* Post metadata is YAML front matter at the start of a Markdown document.
* The generator parses timestamps as local date-times and sorts them in descending order.
* The generator stops with an error that identifies a post with invalid metadata.

## User scenarios
* Given two valid posts with different publication timestamps, when the site is generated, then the newer post is listed before the older post.
* Given a post without a publication timestamp, when the site is generated, then generation fails and identifies that post.
* Given a post with an invalid publication timestamp, when the site is generated, then generation fails and identifies that post.
