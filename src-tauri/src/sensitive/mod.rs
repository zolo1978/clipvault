pub mod patterns;

use crate::models::Sensitivity;

/// Run the sensitive detection pipeline on text content.
pub fn detect_sensitive(text: &str) -> Sensitivity {
    if text.is_empty() {
        return Sensitivity::Clean;
    }

    // Regex content scanning — priority: most specific first
    if let Some(kind) = patterns::scan_text(text) {
        return Sensitivity::Sensitive(kind);
    }

    Sensitivity::Clean
}
