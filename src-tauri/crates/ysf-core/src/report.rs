use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PdfReport {
    pub title: String,
    pub evidence_id: String,
    pub operator: String,
    pub case_name: String,
    pub device: String,
    pub date: String,
    pub sections: Vec<ReportSection>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReportSection {
    pub heading: String,
    pub content: String,
}

/// Generate a forensic PDF report
pub fn generate_pdf_report(report: &PdfReport) -> Result<Vec<u8>, String> {
    use printpdf::*;

    let (doc, page_idx, layer_idx) = PdfDocument::new(
        &report.title,
        Mm(210.0), // A4 width
        Mm(297.0), // A4 height
        "Evidence Layer",
    );

    let current_layer = doc.get_page(page_idx).get_layer(layer_idx);
    let font = doc.add_builtin_font(BuiltinFont::Helvetica).map_err(|e| e.to_string())?;
    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold).map_err(|e| e.to_string())?;

    let mut y = Mm(275.0); // Start near top
    let line_height = Mm(5.0);

    // Title
    current_layer.use_text(&report.title, 18.0, Mm(20.0), y, &font_bold);
    y -= Mm(10.0);

    // Metadata
    let meta = vec![
        ("Evidence ID:", &report.evidence_id),
        ("Operator:", &report.operator),
        ("Case:", &report.case_name),
        ("Device:", &report.device),
        ("Date:", &report.date),
    ];
    for (label, value) in meta {
        current_layer.use_text(&format!("{} {}", label, value), 10.0, Mm(20.0), y, &font);
        y -= line_height;
    }

    y -= Mm(5.0);

    // Sections
    for section in &report.sections {
        if y < Mm(30.0) { break; } // Stop before page bottom
        current_layer.use_text(&section.heading, 12.0, Mm(20.0), y, &font_bold);
        y -= line_height;
        current_layer.use_text(&section.content, 10.0, Mm(20.0), y, &font);
        y -= line_height * 2.0;
    }

    // Encrypted PDF if needed
    let bytes = doc.save_to_bytes().map_err(|e| e.to_string())?;
    Ok(bytes)
}
