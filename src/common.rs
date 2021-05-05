// This file contains some constants and other commonly used in the application stuff
//

// Error message displayed when input file name does not have *.md extension
pub(crate) const INCORRECT_INPUT_FILE: &str = "input file must have \"md\" extension";

// Error message displayed when output file name does not have *.pdf extension
pub(crate) const INCORRECT_OUTPUT_FILE: &str = "output file must have \"pdf\" extension";

// Error message displayed when trace subscriber cannot be set
pub(crate) const CANNOT_SET_TRACE_SUBSCRIBER: &str = "cannot set a subscriber for tracing";

// Error message displayed when markdown file has incorrect content or it cannot be read
pub(crate) const ERROR_READING_MD_CONTENT: &str = "cannot read markdown file content";

// Error message displayed when PDF application for wkhtmltopdf cannot be initialized
pub(crate) const CANNOT_INIT_PDF_APP: &str = "cannot initialize PDF application";

// Error message displayed when PDF document cannot be rendered
pub(crate) const CANNOT_RENDER_PDF: &str = "cannot render PDF document";

// Error message displayed when PDF document cannot be saved
pub(crate) const CANNOT_SAVE_PDF: &str = "cannot save PDF document";

// Markdown file extension
pub(crate) const MD_EXTENSION: &str = ".md";

// PDF file extension
pub(crate) const PDF_EXTENSION: &str = ".pdf";

// Default markdown file name
pub(crate) const DEFAULT_MD_NAME: &str = "input.md";

// Default PDF file name
pub(crate) const DEFAULT_PDF_NAME: &str = "output.pdf";