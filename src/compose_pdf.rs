// This file contains PDF renderer implementation
//

use std::path::Path;

use wkhtmltopdf::pdf;
use tracing::{debug, info};

use crate::common::{CANNOT_INIT_PDF_APP, CANNOT_RENDER_PDF,
                    CANNOT_SAVE_PDF};

use crate::custom_error::custom_err_with_cause;

pub(crate) fn convert_html_to_pdf<P: AsRef<Path>>(filename: &P, html_content: &str) -> Result<(), Box<dyn std::error::Error>> {
    let pdf_app = pdf::PdfApplication::new()
        .map_err(|e|custom_err_with_cause(CANNOT_INIT_PDF_APP, Box::new(e)))?;

        debug!("PDF application is initialized");

    let mut pdf_output = pdf_app.builder()
        .orientation(pdf::Orientation::Portrait)
        .build_from_html(html_content)
        .map_err(|e|custom_err_with_cause(CANNOT_RENDER_PDF, Box::new(e)))?;
    
    debug!("PDF file rendered successfully");
    
    pdf_output.save(filename)
        .map_err(|e|custom_err_with_cause(CANNOT_SAVE_PDF, Box::new(e)))?;
    info!("PDF file successfully saved");
    Ok(())
}
