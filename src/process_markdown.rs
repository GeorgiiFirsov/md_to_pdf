// This file contains Markdown parser implementation
//

extern crate comrak;


use std::fs;
use comrak::{markdown_to_html, ComrakOptions};

use crate::common::ERROR_READING_MD_CONTENT;


// CSS style used to format intermediate HTML representation
const CSS_STYLE: &'static str = include_str!("../styles/pretty_pdf.css");


pub(crate)
fn parse_markdown_to_pretty_html(filename: &str) -> String {
    //
    // Read markdown content and convert it into HTML
    //

    let md_content = fs::read_to_string(filename)
        .expect(ERROR_READING_MD_CONTENT);

    let html = markdown_to_html(
        &md_content, &ComrakOptions::default());

    //
    // Embed a style into HTML and return pretty HTML text
    //

    // TODO: embed CSS styles

    html
}