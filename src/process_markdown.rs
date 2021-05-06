// This file contains Markdown parser and converter implementation
//

extern crate comrak;
extern crate tracing;
extern crate phf;


use std::fs;
use phf::phf_map;
use tracing::{debug, info};
use comrak::{markdown_to_html, ComrakOptions};

use crate::common::ERROR_READING_MD_CONTENT;


// CSS style used to format intermediate HTML representation
const CSS_STYLE: &'static str = include_str!("../styles/pretty_pdf.css");

// HTML high-level tags
const HTML_HEAD_STYLE_BEGIN: &'static str = "<head><style>";
const HTML_HEAD_STYLE_END: &'static str = "</style></head>";
const HTML_BODY_BEGIN: &'static str = "<body>";
const HTML_BODY_END: &'static str = "</body>";

// Mapping between initial tokens in HTML and their replacements
const TOKEN_MAPPING: phf::Map<&'static str, &'static str> = phf_map!(
    "blockquote"    => "blockquote class=\"quote\"",
    "<p>~~"         => "<p class=\"crossed_out_text\">",
    "~~\n~~"        => "<br />\n",
    "~~"            => "",
    "<p>~"          => "<p class=\"underlined_text\">",
    "~\n~"          => "<br />\n",
    "~"             => ""
);


pub(crate)
fn convert_markdown_to_pretty_html(filename: &str) -> String {
    //
    // Read markdown content and convert it into HTML
    //

    let md_content = fs::read_to_string(filename)
        .expect(ERROR_READING_MD_CONTENT);

    let mut raw_html = markdown_to_html(
        &md_content, &ComrakOptions::default());

    info!("File {} converted in HTML successfully", filename);

    //
    // Embed a styles into HTML
    //

    for (&token, &replacement) in TOKEN_MAPPING.entries() {
        raw_html = raw_html.replace(token, replacement);
    }

    info!("HTML tokens replaced with their styled analogues");
    debug!("Style: {}", CSS_STYLE);

    //
    // Compose everything together and return
    //

    [HTML_HEAD_STYLE_BEGIN, CSS_STYLE, HTML_HEAD_STYLE_END,
        HTML_BODY_BEGIN, &raw_html, HTML_BODY_END].join("")
}