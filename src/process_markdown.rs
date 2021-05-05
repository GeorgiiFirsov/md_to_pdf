// This file contains Markdown parser implementation
//

use std::fs;
use comrak::{Arena, parse_document, ComrakOptions};

use crate::common::INCORRECT_MD_CONTENT;


pub(crate)
fn markdown_to_tree(filename: &str) {
    let md_content = fs::read_to_string(filename)
        .expect(INCORRECT_MD_CONTENT);

    let arena = Arena::new();
    let root = parse_document(
        &arena, &md_content, &ComrakOptions::default());


}