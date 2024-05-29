// This file contains application's entry point
//

mod process_markdown;
mod compose_pdf;
mod common;
mod custom_error;
mod frontmatter;

extern crate clap;
extern crate regex;
extern crate base64;
extern crate comrak;
extern crate tracing;
extern crate wkhtmltopdf;

use std::{ffi::OsString, fs::{self, File}, io::{Read, Write}, path::Path};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use clap::{ArgAction, Parser};
use custom_error::custom_err_with_cause;
use frontmatter::Frontmatter;
use tracing::{error, Level};

use process_markdown::convert_markdown_to_pretty_html;
use compose_pdf::convert_html_to_pdf;
use common::CANNOT_SET_TRACE_SUBSCRIBER;
use handlebars::{Context, Handlebars, Helper, HelperResult, RenderContext, RenderErrorReason};
use serde::Serialize;

use crate::common::ERROR_READING_MD_CONTENT;

#[derive(Parser, Debug)]
#[command(
    name = "md_to_pdf",
    version = "1.2",
    author = "GeorgyFirsov <gfirsov007@gmail.com>, Oliver Paraskos <oparaskos@gmail.com>",
    about = "Converts a markdown file into a pdf"
)]
struct Opts {
    // Path to the (main) input file
    #[arg(value_name = "Markdown File")]
    input: OsString,

    // Path to additional input files included in order
    #[arg(value_name = "Markdown Files")]
    additional_inputs: Option<Vec<OsString>>,

    // Path to the output file to be created (optional)
    #[arg(short, long, value_name = "PDF file")]
    pdf: Option<OsString>,

    #[arg(short, long)]
    additional_stylesheets: Option<Vec<OsString>>,

    // Path to the output file to be created (optional)
    #[arg(long, value_name = "HTML Output file")]
    html: Option<OsString>,

    // Annotate external links
    #[arg(long, action = ArgAction::SetTrue, overrides_with = "no_annotate_external_links")]
    annotate_external_links: (),

    // Annotate external links
    #[arg(long, action = ArgAction::SetTrue, overrides_with = "no_annotate_headings")]
    annotate_headings: (),

    // Annotate external links
    #[arg(long, action = ArgAction::SetTrue, overrides_with = "no_resolve_image_src")]
    resolve_image_src: (),

    #[arg(long)]
    no_resolve_image_src: bool,
    
    #[arg(long)]
    no_annotate_headings: bool,
    
    #[arg(long)]
    no_annotate_external_links: bool,
    
    #[arg(long)]
    stylesheet: Option<OsString>,

    #[arg(long)]
    frontmatter_delimiter: Option<String>,

    #[arg(long)]
    master_template: Option<String>,
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    //
    // Setup trace service
    //
    color_backtrace::install();
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect(CANNOT_SET_TRACE_SUBSCRIBER);

    //
    // Read and parse command line arguments
    //
    let opts = Opts::parse();

    let input_file = Path::new(&opts.input).to_path_buf();

    let pdf_output_file = match &opts.pdf {
        Some(f) => Some(Path::new(f).to_path_buf()),
        None => None,
    };

    let html_output_file = match &opts.html {
        Some(p) => Some(File::create(p)?),
        None => None,
    };

    //
    // Parse markdown and compose PDF
    //
    let md_content = fs::read_to_string(&input_file).map_err(|e|custom_err_with_cause(ERROR_READING_MD_CONTENT, Box::new(e)))?;
    let frontmatter = match frontmatter::Frontmatter::parse(&md_content) {
        Ok(f) => {
            Some(f.0)
        },
        Err(e) => {
            error!(e);
            None
        }
    };
    let result = convert_markdown_to_pretty_html(&input_file, &md_content, &opts)?;
    let mut toc = Vec::from(result.1);
    if let Some(f) = &frontmatter {
        if let Some(title) = &f.title { 
            toc.insert(0, (0, format!("<a href=\"#_0\">{}</a>", title)))
        }
    }
    let main_document = Document {
        number: 0,
        frontmatter,
        html: result.0,
    };
    let mut all_htmls = vec![main_document];
    if let Some(additional_files) = &opts.additional_inputs {
        let mut i = 1;
        for f in additional_files {
            let input_file = Path::new(&f).to_path_buf();
            let md_content = fs::read_to_string(&input_file).map_err(|e|custom_err_with_cause(ERROR_READING_MD_CONTENT, Box::new(e)))?;
            let frontmatter = match frontmatter::Frontmatter::parse(&md_content) {
                Ok(f) => {
                    Some(f.0)
                },
                Err(e) => {
                    error!(e);
                    None
                }
            };
            let result = convert_markdown_to_pretty_html(&input_file, &md_content, &opts)?;
            if let Some(f) = &frontmatter {
                if let Some(title) = &f.title { 
                    toc.push((0, format!("<a href=\"#_{}\">{}</a>", i, title)))
                }
            }
            let document = Document {
                number: i,
                frontmatter,
                html: result.0,
            };
            for v in result.1 {
                toc.push(v);
            }
            all_htmls.push(document);
            i += 1;
        }
    }
    let final_html = combine_markdown(&toc, all_htmls, &opts)?;

    if let Some(mut html_output) = html_output_file {
        html_output.write_all(final_html.as_bytes())?;
    }
    
    if let Some(pdf_output_path) = pdf_output_file {
        convert_html_to_pdf(&pdf_output_path, final_html.as_str())?;
    }
    return Ok(());
}

#[derive(Serialize)]
struct Document {
    number: usize,
    frontmatter: Option<Frontmatter>,
    html: String
}

fn generate_toc(entries: &[(u8, String)]) -> String {
    let mut toc_html = String::from("<ul class=\"table-of-contents\">");
    let mut level: u8 = 0;
    let mut open = false;

    for (entry_level, link) in entries {
        if entry_level == &level && open {
            toc_html.push_str("</li>");
            open = false;
        }
        while entry_level > &level {
            if !open {
                toc_html.push_str("<li>");
                open = true;
            }
            toc_html.push_str("<ul>");
            level += 1;
        }
        while entry_level < &level {
            toc_html.push_str("</ul>");
            level -= 1;
        }

        toc_html.push_str("<li>");
        toc_html.push_str(&link);
        open = true;
        level = *entry_level;
    }
    if open {
        toc_html.push_str("</li>");
    }

    // Close any remaining open ul tags
    while level > 0 {
        toc_html.push_str("</ul></li>");
        level -= 1;
    }
    toc_html.push_str("</ul>");

    toc_html
}

fn combine_markdown(toc: &Vec<(u8, String)>, content: Vec<Document>, opts: &Opts) -> Result<String, Box<dyn std::error::Error>> {
    let stylesheets_html = load_stylesheets(opts)?;    //
    // Compose everything together and return
    //
    let mut reg = Handlebars::new();
    let template = load_master_template(&opts)?;
    let dat = TemplateData {
        styles: stylesheets_html,
        toc: generate_toc(&toc.as_slice()),
        content: content
    };


    reg.register_helper("dataurl", Box::new(handlebars_data_url));
    

    let mut ctx = Context::wraps(dat)?;
    match &opts.master_template {
        Some(master_path) => {
            let dir = Path::new(&master_path).parent().unwrap().to_path_buf(); 
            match ctx.data_mut() {
                serde_json::value::Value::Object(m) => m.insert("__master_template_dir".to_owned(), dir.to_string_lossy().into()),
                _ => None,
            };
        },
        None => ()
    };
    let result = reg.render_template_with_context(&template, &ctx)?;
    Ok(result)
}

fn handlebars_data_url(
    h: &Helper,
    _hbs: &Handlebars,
    ctx: &handlebars::Context,
    _rc: &mut RenderContext,
    out: &mut dyn handlebars::Output,
) -> HelperResult {
    let a = h
        .param(0)
        .and_then(|v| v.value().as_str())
        .ok_or(RenderErrorReason::ParamNotFoundForIndex("dataurl", 0))?;
    let b = h
        .param(1)
        .and_then(|v| v.value().as_str());
    let path = if let Some(path) = b { path } else { a };
    let mime = if b.is_some() { Some(a) } else { None };
    let d = ctx.data().get("__master_template_dir");

    let from_path = match d.and_then(|d|d.as_str()) {
        Some(master_path) => Path::new(&master_path).to_path_buf(),
        None => std::env::current_dir().expect("No Current Dir")
    };
    let resolved_path = from_path.join(&path);
    let is_valid_filepath = resolved_path.exists();
    let replacement = if is_valid_filepath {
        let mime_type = if let Some(t) = mime {
            t.to_owned()
        } else {
            let t = infer::get_from_path(&resolved_path).expect("Could not get type from file").map(|k|k.mime_type()).unwrap_or("application/octet-stream");
            t.to_owned()
        };
        let mut file = File::open(&resolved_path).expect("Could not open file");
        let mut content_str : Vec<u8> = vec![];
        file.read_to_end(&mut content_str).expect("Could not read file content");
        format!("data:{};base64,{}", mime_type, STANDARD.encode(content_str))
    } else {
        error!("Failed to generate base64 data url for {}", path);
        format!("{}?b64_failed!", path)
    };
    out.write(&replacement).expect("Could not write replacement");
    Ok(())
}

#[derive(Serialize)]
struct TemplateData {
    styles: String,
    toc: String,
    content: Vec<Document>,
}


fn load_master_template(opts: &Opts) -> Result<String, Box<dyn std::error::Error>> {
    let template_content = match &opts.master_template {
        Some(path) => {
            let mut f = File::open(path)?;
            let mut template_str = String::new();
            f.read_to_string(&mut template_str)?;
            template_str
        }
        None => String::from_utf8(DEFAULT_TEMPLATE.to_vec())?
    };
    Ok(template_content)
}

// CSS style used to format intermediate HTML representation
const DEFAULT_CSS_STYLE: &'static str = include_str!("../styles/pretty_pdf.css");
const DEFAULT_TEMPLATE: &'static [u8] = include_bytes!("../template/document.html.hbs");

fn load_stylesheets(opts: &Opts) -> Result<String, Box<dyn std::error::Error>> {
    let mut stylesheets_html: String = String::new();
    if let Some(base_stylesheet_path) = &opts.stylesheet {
        let mut f = File::open(base_stylesheet_path).map_err(|e|custom_err_with_cause("Unable to open base stylesheet", Box::new(e)))?;
        let mut stylesheet_str = String::new();
        f.read_to_string(&mut stylesheet_str).map_err(|e|custom_err_with_cause("Unable to read base stylesheet", Box::new(e)))?;

        let opening_tag = format!("<style id=\"base\" data-href=\"{}\" type=\"text/css\">", base_stylesheet_path.to_string_lossy());
        stylesheets_html.push_str(&opening_tag);
        stylesheets_html.push_str(&stylesheet_str);
        stylesheets_html.push_str("</style>");
    } else {
        stylesheets_html.push_str("<style id=\"base\" type=\"text/css\">");
        stylesheets_html.push_str(DEFAULT_CSS_STYLE);
        stylesheets_html.push_str("</style>");
    }
    if let Some(add) = &opts.additional_stylesheets {
        for path in add {
            let mut f = File::open(path).map_err(|e|custom_err_with_cause("Unable to open additional stylesheet", Box::new(e)))?;
            let mut stylesheet_str = String::new();
            f.read_to_string(&mut stylesheet_str).map_err(|e|custom_err_with_cause("Unable to read additional stylesheet", Box::new(e)))?;
            let opening_tag = format!("<style data-href=\"{}\" type=\"text/css\">", path.to_string_lossy());
            stylesheets_html.push_str(&opening_tag);
            stylesheets_html.push_str(&stylesheet_str);
            stylesheets_html.push_str("</style>");
        }
    }

    Ok(stylesheets_html)
}

/*
cargo run -- --no-annotate-headings --no-annotate-external-links --html out.html --frontmatter-delimiter="---" --master-template="macs/document.html.hbs" --additional-stylesheets="macs/styles.css" C:\Users\User\Workspace\macs-visitor-portal\USER_GUIDE.md
C:\Users\User\Downloads\weasyprint-windows\dist\weasyprint.exe .\out.html out.pdf
 */