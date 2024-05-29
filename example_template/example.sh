#!/bin/sh

cargo run -- \
    --no-annotate-headings \
    --no-annotate-external-links \
    --html="out.html" \
    --frontmatter-delimiter="---" \
    --master-template="./document.html.hbs" \
    --additional-stylesheets="./style.css" \
    ../README.md
weasyprint out.html out.pdf