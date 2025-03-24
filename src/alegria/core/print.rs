// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use printers::{common::base::printer::Printer, get_default_printer, get_printers};
use printpdf::*;

static TICKET_FONT_TTF: &[u8] = include_bytes!("../../../resources/fonts/RobotoFlex.ttf");

#[derive(Debug, Clone)]
pub struct AlegriaPrinter(Printer);

impl std::fmt::Display for AlegriaPrinter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.name)
    }
}

impl PartialEq for AlegriaPrinter {
    fn eq(&self, other: &Self) -> bool {
        self.0.name == other.0.name
    }
}

impl From<Printer> for AlegriaPrinter {
    fn from(printer: Printer) -> Self {
        AlegriaPrinter(printer)
    }
}

impl AlegriaPrinter {
    // Loads the system printers on another tokio thread
    pub async fn load_printers() -> (Option<AlegriaPrinter>, Vec<AlegriaPrinter>) {
        tokio::task::spawn_blocking(|| {
            let default = get_default_printer().map(AlegriaPrinter::from);
            let all = get_printers()
                .into_iter()
                .map(AlegriaPrinter::from)
                .collect();
            (default, all)
        })
        .await
        .unwrap_or_else(|_| (None, Vec::new()))
    }

    pub async fn print(self: Arc<Self>) -> Result<(), &'static str> {
        tokio::task::spawn_blocking(move || {
            if let Ok(doc) = generate_ticket() {
                self.0.print(&doc, Some("Alegria Print Job"))
            } else {
                Err("Failed to generate ticket")
            }
        })
        .await
        .unwrap_or(Err("Failed to spawn a blocking task"))
    }
}

/// TODO: Proper Doc Generation (Given a Simple Invoice)
fn generate_ticket() -> Result<Vec<u8>, &'static str> {
    // Create a new PDF document
    let mut doc = PdfDocument::new("Hotel Name Ticket");

    // Load and register an external font
    let custom_font =
        ParsedFont::from_bytes(TICKET_FONT_TTF, 0, &mut Vec::new()).ok_or("Failed to load font")?;
    let custom_font_id = doc.add_font(&custom_font);

    // Create operations for different text styles
    let mut ops = vec![
        // Save the graphics state to allow for position resets later
        Op::SaveGraphicsState,
        // Start a text section (required for text operations)
        Op::StartTextSection,
        // Position the text cursor from the bottom left
        Op::SetTextCursor {
            pos: Point::new(Mm(10.0), Mm(280.0)),
        },
        // Set a built-in font (Helvetica) with its size
        Op::SetFontSize {
            size: Pt(24.0),
            font: custom_font_id.clone(),
        },
        Op::SetLineHeight { lh: Pt(24.0) },
        // Set text color to blue
        Op::SetFillColor {
            col: Color::Rgb(Rgb {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                icc_profile: None,
            }),
        },
        // Write text with the built-in font
        Op::WriteText {
            items: vec![TextItem::Text("My Hotel Name".to_string())],
            font: custom_font_id.clone(),
        },
        // Add a line break to move down
        Op::AddLineBreak,
        // End the text section
        Op::EndTextSection,
        // Restore the graphics state
        Op::RestoreGraphicsState,
    ];

    ops.extend(vec![
        // Save the graphics state to allow for position resets later
        Op::SaveGraphicsState,
        // Start a text section (required for text operations)
        Op::StartTextSection,
        // Position the text cursor from the bottom left
        Op::SetTextCursor {
            pos: Point::new(Mm(20.0), Mm(275.0)),
        },
        // Set a built-in font (Helvetica) with its size
        Op::SetFontSize {
            size: Pt(12.0),
            font: custom_font_id.clone(),
        },
        Op::SetLineHeight { lh: Pt(12.0) },
        // Set text color to blue
        Op::SetFillColor {
            col: Color::Rgb(Rgb {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                icc_profile: None,
            }),
        },
        // Write text with the built-in font
        Op::WriteText {
            items: vec![TextItem::Text("Factura Simplificada".to_string())],
            font: custom_font_id.clone(),
        },
        // Add a line break to move down
        Op::AddLineBreak,
        // End the text section
        Op::EndTextSection,
        // Restore the graphics state
        Op::RestoreGraphicsState,
    ]);

    // Create a page with our operations
    let page = PdfPage::new(Mm(80.0), Mm(290.0), ops);

    // Save the PDF to a file
    Ok(doc
        .with_pages(vec![page])
        .save(&PdfSaveOptions::default(), &mut Vec::new()))
}
