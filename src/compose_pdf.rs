// This file contains PDF renderer implementation
//

extern crate wkhtmltopdf;
extern crate tracing;


use wkhtmltopdf::pdf;
use tracing::{debug, info};

use crate::common::{CANNOT_INIT_PDF_APP, CANNOT_RENDER_PDF,
                    CANNOT_SAVE_PDF};


pub(crate)
fn convert_html_to_pdf(filename: &str, html_content: &str) {
    let pdf_app = pdf::PdfApplication::new()
        .expect(CANNOT_INIT_PDF_APP);

    debug!("PDF application is initialized");

    // mut is necessary for method 'save'
    let mut pdf_output = pdf_app.builder()
        .orientation(pdf::Orientation::Portrait)
        .build_from_html(html_content)
        .expect(CANNOT_RENDER_PDF);

    info!("PDF file rendered successfully");

    pdf_output.save(filename)
        .expect(CANNOT_SAVE_PDF);

    info!("PDF file successfully saved to {}", filename);
}