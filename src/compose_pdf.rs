// This file contains PDF renderer implementation
//

extern crate wkhtmltopdf;


use wkhtmltopdf::pdf;

use crate::common::{CANNOT_INIT_PDF_APP, CANNOT_RENDER_PDF,
                    CANNOT_SAVE_PDF};


pub(crate)
fn convert_html_to_pdf(filename: &str, html_content: &str) {
    let pdf_app = pdf::PdfApplication::new()
        .expect(CANNOT_INIT_PDF_APP);

    // mut is necessary for method 'save'
    let mut pdf_output = pdf_app.builder()
        .orientation(pdf::Orientation::Portrait)
        .build_from_html(html_content)
        .expect(CANNOT_RENDER_PDF);

    pdf_output.save(filename)
        .expect(CANNOT_SAVE_PDF);
}