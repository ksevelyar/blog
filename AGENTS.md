* don't use #[allow] to bypass clippy
* no mod.rs allowed, edition is 2024
* after changes test with `ci`

## CI
* use oxfmt for formatting

## Template
* use bem namespace: block, element, modifier
* use span + div as main BEM tags, use others only when span and div will not work, don't use ids
* avoid redundant wrappers
