// This file contains Markdown parser and converter implementation
//

use std::{cell::RefCell, fs::File, io::{BufWriter, Read}, path::Path};
use url::Url;
use base64::{Engine as _, engine::general_purpose};
use regex::Regex;
use tracing::info;
use comrak::{arena_tree::Node, format_html_with_plugins, nodes::{Ast, AstNode}, parse_document, Anchorizer, Arena, ComrakOptions, Plugins};

use crate::Opts;


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
    /* Tags */
    ("<p>#(.+)</p>",                    "<p class=\"tag\"><span class=\"tag_ns\">#</span>$1</p>"),
    /* Quotes */
    ("blockquote",                      "blockquote class=\"quote\""),
    /* Underlined and crossed out text */
    ("~~\n~~",                          "<br />"),
    ("~\n~",                            "<br />"),
    ("<p>~~(.*)~~</p>",                 "<p class=\"crossed_out_text\">$1</p>"),
    ("<p>~(.*)~</p>",                   "<p class=\"underlined_text\">$1</p>"),
    ("<img(.*)/>\\{width=([^}]+)\\}",   "<img$1 style=\"width: $2\" />"),
    /* Line endings */
    ("([^>])\n",                        "$1<br />\n"),
    ("</em>\n",                         "</em><br />\n"),
    ("</strong>\n",                     "</strong><br />\n"),
    /* Marks */
    ("<p>::(.*)::</p>",                 "<p class=\"mark\"><span class=\"mark_dots\">::</span>$1<span class=\"mark_dots\">::</span></p>"),

    /* Code snippets */
    ("<pre><code( class=\"(.*)\")?>",   "<pre><span class=\"code_multiline\"> $2\n<span class=\"code_backtick\">```</span><br />\n"),
    ("</code></pre>",                   "<span class=\"code_backtick\">```</span>\n</span></pre>"),
    ("^\\s*<code>(.*)</code>\\s*$",       "$1<span class=\"code_singleline\"><span class=\"code_backtick\">` </span>$2\
                                         <span class=\"code_backtick\"> `</span></span>$3"),
    ("</?pre>",                         ""),
    /* Lists and tasks */
    ("<li>\\[ \\] (.*)</li>",           "<li class=\"task_list\"><img src=\"data:image/gif;base64,__unchecked_task_base64_tag__\" \
                                         width=20px height=20px /> $1</li>"),
    ("<li>\\[x\\] (.*)</li>",           "<li class=\"task_list\"><img src=\"data:image/gif;base64,__checked_task_base64_tag__\" \
                                         width=20px height=20px /> <span class=\"crossed_out_text\">$1</span></li>"),
    ("<li>",                            "<li class=\"common_list\">"),
];


// Mapping between initial tokens in HTML and their replacements
const EXT_LINK_MAPPING: &'static [(&'static str, &'static str)] = &[
    ("<a href=(\".*\")>(.*)</a>", "<span class=\"link_bracket\">[ </span><a href=$1>$2</a><span class=\"link_bracket\"> ](</span>\
        <img src=\"data:image/gif;base64,__link_base64_tag__\" width=20px height=20px />\
        <span class=\"link_bracket\">)</span>")
];

// Mapping between initial tokens in HTML and their replacements
const HEADING_MAPPING: &'static [(&'static str, &'static str)] = &[
    ("<h1>(.*)</h1>",                   "<h1><span class=\"header_sign\">H</span><span class=\"header_sign_num\">1</span>$1</h1>"),
    ("<h2>(.*)</h2>",                   "<h2><span class=\"header_sign\">H</span><span class=\"header_sign_num\">2</span>$1</h2>"),
    ("<h3>(.*)</h3>",                   "<h3><span class=\"header_sign\">H</span><span class=\"header_sign_num\">3</span>$1</h3>"),
    ("<h4>(.*)</h4>",                   "<h4><span class=\"header_sign\">H</span><span class=\"header_sign_num\">4</span>$1</h4>"),
    ("<h5>(.*)</h5>",                   "<h5><span class=\"header_sign\">H</span><span class=\"header_sign_num\">5</span>$1</h5>"),
    ("<h6>(.*)</h6>",                   "<h6><span class=\"header_sign\">H</span><span class=\"header_sign_num\">6</span>$1</h6>"),
    
    ("<h1>",                            "<div class=\"page-break\"></div><h1>"),
];


fn gather_text<'a>(node: &'a AstNode<'a>) -> String {
    let mut text = String::from("");
    if let comrak::nodes::NodeValue::Text(t) = &node.data.borrow().value {
        text.push_str(&t);
    }
    let children = node.children();
    for child in children.into_iter() {
        text.push_str(&gather_text(child));
    }
    text
}


fn traverse_nodes<'a>(node: &'a AstNode<'a>) -> Vec<(u8, String)> {
    let mut v: Vec<(u8, String)> = vec![];
    if let comrak::nodes::NodeValue::Heading(heading) = &node.data.borrow().value {
        let mut text = String::new();
        for child in node.children() {
            let inner_text = gather_text(child);
            text.push_str(&inner_text);
        }
        let href = Anchorizer::new().anchorize(text.to_string());
        let lnk = format!("<a href=\"#{}\">{}</a>", href, text).to_string();
        v.push((heading.level, lnk));
    } else {
        for child in node.children() {
            for x in traverse_nodes(child) {
                v.push(x);
            }
        }
    }
    v
}

pub(crate) fn convert_markdown_to_pretty_html<P: AsRef<Path>>(filename: &P, md_content: &String, opts: &Opts) -> Result<(String, Vec<(u8, String)>), Box<dyn std::error::Error>> {
    //
    // Read markdown content and convert it into HTML
    //

    let mut comrak_opts = ComrakOptions::default();
    if let Some(fmd) = &opts.frontmatter_delimiter {
        comrak_opts.extension.front_matter_delimiter = Some(fmd.to_string());
    }
    comrak_opts.extension.table = true;
    comrak_opts.parse.smart = true;
    comrak_opts.extension.header_ids = Some(String::from(""));
    
    let arena: Arena<Node<RefCell<Ast>>> = Arena::new();
    let ast = parse_document(&arena, &md_content, &comrak_opts);
    let toc = traverse_nodes(&ast);


    let mut bw = BufWriter::new(Vec::new());
    format_html_with_plugins(ast, &comrak_opts, &mut bw, &Plugins::default()).unwrap();
    let mut raw_html = String::from_utf8(bw.into_inner().unwrap()).unwrap();

    info!("File converted in HTML successfully");

    //
    // Embed a styles into HTML
    //
    apply_mappings(&mut raw_html, TOKEN_MAPPING);

    //
    // If necessary embed link picture, unchecked and checked tasks pictures as base64 into HTML
    //

    if !opts.no_annotate_external_links {
        apply_mappings(&mut raw_html, EXT_LINK_MAPPING);
        let re = Regex::new(LINK_PICTURE_BASE64_TAG).unwrap();
        raw_html = re.replace_all(&raw_html, general_purpose::STANDARD.encode(LINK_PICTURE)).to_string();
    }

    if !opts.no_annotate_headings {
        apply_mappings(&mut raw_html, HEADING_MAPPING);
        let re = Regex::new(LINK_PICTURE_BASE64_TAG).unwrap();
        raw_html = re.replace_all(&raw_html, general_purpose::STANDARD.encode(LINK_PICTURE)).to_string();
    }

    let re = Regex::new(UNCHECKED_TASK_PICTURE_BASE64_TAG).unwrap();
    raw_html = re.replace_all(&raw_html, general_purpose::STANDARD.encode(UNCHECKED_TASK_PICTURE)).to_string();

    let re = Regex::new(CHECKED_TASK_PICTURE_BASE64_TAG).unwrap();
    raw_html = re.replace_all(&raw_html, general_purpose::STANDARD.encode(CHECKED_TASK_PICTURE)).to_string();


    if !opts.no_resolve_image_src {
        let re = Regex::new("<img(.*)src=\"([^\"]+)\"").unwrap();
        raw_html = re.replace_all(&raw_html, |cap: &regex::Captures| {
            let fluff = cap.get(1).unwrap().as_str();
            let url = cap.get(2).unwrap().as_str();
            let mut cleaned_url = url.to_string();
            if !is_valid_url(url) && !is_valid_filepath(url) {
                let new_path = Path::new(filename.as_ref()).parent().unwrap().join(url).to_string_lossy().to_string();
                if is_valid_filepath(&new_path) {
                    cleaned_url = new_path;
                }
            }

            if is_valid_filepath(&cleaned_url) {
                let mime = infer::get_from_path(&cleaned_url).expect("Could not get type from file").map(|k|k.mime_type()).unwrap_or("application/octet-stream");
                let mut file = File::open(&cleaned_url).expect("Could not open file");
                let mut content_str : Vec<u8> = vec![];
                file.read_to_end(&mut content_str).expect("Could not read file content");
                format!("<img{}src=\"data:{};base64,{}\"", fluff, mime, general_purpose::STANDARD.encode(content_str))
            } else {
                cap.get(0).unwrap().as_str().to_string()
            }
        }).to_string();
    }

    Ok((raw_html, toc))
}


fn is_valid_url(input: &str) -> bool {
    Url::parse(input).is_ok()
}

fn is_valid_filepath(input: &str) -> bool {
    Path::new(input).exists()
}

fn apply_mappings(raw_html: &mut String, mapping: &'static [(&'static str, &'static str)]) {
    for (token, replacement) in mapping.iter() {
        let re = Regex::new(*token).unwrap();
        *raw_html = re.replace_all(&*raw_html, *replacement).to_string();
    }
}