// This file contains Markdown parser and converter implementation
//

use std::fs;
use regex::Regex;
use tracing::{debug, info};
use comrak::{markdown_to_html, ComrakOptions};

use crate::common::ERROR_READING_MD_CONTENT;


// CSS style used to format intermediate HTML representation
const CSS_STYLE: &'static str = include_str!("../styles/pretty_pdf.css");

// Binary image of link and token to be replaced with base64 encoded picture data
const LINK_PICTURE: &'static [u8] = include_bytes!("../resources/link.png");
const LINK_PICTURE_BASE64_TAG: &'static str = "__link_base64_tag__";

// Binary image of unchecked task and token to be replaced with base64 encoded picture data
const UNCHECKED_TASK_PICTURE: &'static [u8] = include_bytes!("../resources/task_unchecked.png");
const UNCHECKED_TASK_PICTURE_BASE64_TAG: &'static str = "__unchecked_task_base64_tag__";

// Binary image of checked task and token to be replaced with base64 encoded picture data
const CHECKED_TASK_PICTURE: &'static [u8] = include_bytes!("../resources/task_checked.png");
const CHECKED_TASK_PICTURE_BASE64_TAG: &'static str = "__checked_task_base64_tag__";

// Mapping between initial tokens in HTML and their replacements
const TOKEN_MAPPING: &'static [(&'static str, &'static str)] = &[
    /* Headers */
    ("<h1>(.*)</h1>",                   "<h1><span class=\"header_sign\">H</span><span class=\"header_sign_num\">1</span>$1</h1>"),
    ("<h2>(.*)</h2>",                   "<h2><span class=\"header_sign\">H</span><span class=\"header_sign_num\">2</span>$1</h2>"),
    ("<h3>(.*)</h3>",                   "<h3><span class=\"header_sign\">H</span><span class=\"header_sign_num\">3</span>$1</h3>"),
    ("<h4>(.*)</h4>",                   "<h4><span class=\"header_sign\">H</span><span class=\"header_sign_num\">4</span>$1</h4>"),
    ("<h5>(.*)</h5>",                   "<h5><span class=\"header_sign\">H</span><span class=\"header_sign_num\">5</span>$1</h5>"),
    ("<h6>(.*)</h6>",                   "<h6><span class=\"header_sign\">H</span><span class=\"header_sign_num\">6</span>$1</h6>"),
    /* Tags */
    ("<p>#(.+)</p>",                    "<p class=\"tag\"><span class=\"tag_ns\">#</span>$1</p>"),
    /* Quotes */
    ("blockquote",                      "blockquote class=\"quote\""),
    /* Underlined and crossed out text */
    ("~~\n~~",                          "<br />"),
    ("~\n~",                            "<br />"),
    ("<p>~~(.*)~~</p>",                 "<p class=\"crossed_out_text\">$1</p>"),
    ("<p>~(.*)~</p>",                   "<p class=\"underlined_text\">$1</p>"),
    /* Line endings */
    ("([^>])\n",                        "$1<br />\n"),
    ("</em>\n",                         "</em><br />\n"),
    ("</strong>\n",                     "</strong><br />\n"),
    /* Marks */
    ("<p>::(.*)::</p>",                 "<p class=\"mark\"><span class=\"mark_dots\">::</span>$1<span class=\"mark_dots\">::</span></p>"),
    /* Links */
    ("<a href=(\".*\")>(.*)</a>",       "<span class=\"link_bracket\">[ </span><a href=$1>$2</a><span class=\"link_bracket\"> ](</span>\
                                         <img src=\"data:image/gif;base64,__link_base64_tag__\" width=20px height=20px />\
                                         <span class=\"link_bracket\">)</span>"),
    /* Code snippets */
    ("<pre><code( class=\"(.*)\")?>",   "<pre><span class=\"code_multiline\"> $2\n<span class=\"code_backtick\">```</span><br />\n"),
    ("</code></pre>",                   "<span class=\"code_backtick\">```</span>\n</span></pre>"),
    ("(.*)<code>(.*)</code>(.*)",       "$1<span class=\"code_singleline\"><span class=\"code_backtick\">` </span>$2\
                                         <span class=\"code_backtick\"> `</span></span>$3"),
    ("</?pre>",                         ""),
    /* Lists and tasks */
    ("<li>\\[ \\] (.*)</li>",           "<li class=\"task_list\"><img src=\"data:image/gif;base64,__unchecked_task_base64_tag__\" \
                                         width=20px height=20px /> $1</li>"),
    ("<li>\\[x\\] (.*)</li>",           "<li class=\"task_list\"><img src=\"data:image/gif;base64,__checked_task_base64_tag__\" \
                                         width=20px height=20px /> <span class=\"crossed_out_text\">$1</span></li>"),
    ("<li>",                            "<li class=\"common_list\">"),
];


pub(crate) fn convert_markdown_to_pretty_html(filename: &str) -> String {
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

    for (token, replacement) in TOKEN_MAPPING.iter() {
        let re = Regex::new(*token).unwrap();
        raw_html = re.replace_all(&raw_html, *replacement).to_string();
    }

    //
    // If necessary embed link picture, unchecked and checked tasks pictures as base64 into HTML
    //

    let re = Regex::new(LINK_PICTURE_BASE64_TAG).unwrap();
    raw_html = re.replace_all(&raw_html, base64::encode(LINK_PICTURE)).to_string();

    let re = Regex::new(UNCHECKED_TASK_PICTURE_BASE64_TAG).unwrap();
    raw_html = re.replace_all(&raw_html, base64::encode(UNCHECKED_TASK_PICTURE)).to_string();

    let re = Regex::new(CHECKED_TASK_PICTURE_BASE64_TAG).unwrap();
    raw_html = re.replace_all(&raw_html, base64::encode(CHECKED_TASK_PICTURE)).to_string();

    info!("HTML tokens replaced with their styled analogues");
    debug!("Style: {}", CSS_STYLE);

    //
    // Compose everything together and return
    //

    std::format!(
        "<head>\
             <meta http-equiv=\"Content-Type\" content=\"text/html; charset=UTF-8\">\
             <style type=\"text/css\">{}</style>\
         </head>\
         <body>\
             {}\
         </body>",
        CSS_STYLE, &raw_html)
}