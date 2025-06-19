// SPDX-License-Identifier: GPL-3.0-only

use std::{collections::HashMap, fmt::Display, sync::Arc};

use printers::{common::base::printer::Printer, get_default_printer, get_printers};
use printpdf::*;

use super::models::simple_invoice::SimpleInvoice;

static TICKET_FONT_TTF: &[u8] = include_bytes!("../../resources/fonts/RobotoFlex.ttf");

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

#[derive(Default, Debug, Clone, PartialEq)]
pub enum TicketType {
    Invoice,
    #[default]
    Receipt,
}

impl Display for TicketType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            TicketType::Invoice => write!(f, "Invoice"),
            TicketType::Receipt => write!(f, "Receipt"),
        }
    }
}

impl AlegriaPrinter {
    // Loads the system printers on another tokio thread
    pub async fn load_printers() -> (Option<AlegriaPrinter>, Vec<AlegriaPrinter>) {
        tokio::task::spawn_blocking(|| {
            let mut default: Option<AlegriaPrinter> =
                get_default_printer().map(AlegriaPrinter::from);
            let all: Vec<AlegriaPrinter> = get_printers()
                .into_iter()
                .map(AlegriaPrinter::from)
                .collect();
            // If there is no default printer grab the first one (if any)
            if default.is_none() && !all.is_empty() {
                default = all.first().cloned()
            }
            (default, all)
        })
        .await
        .unwrap_or_else(|_| (None, Vec::new()))
    }

    pub async fn print(
        self: Arc<Self>,
        invoice: SimpleInvoice,
        ticket_type: TicketType,
    ) -> Result<(), &'static str> {
        tokio::task::spawn_blocking(move || match ticket_type {
            TicketType::Invoice => {
                if let Ok(doc) = generate_invoice(&invoice) {
                    self.0.print(&doc, Some("Alegria Print Job"))
                } else {
                    Err("Failed to generate invoice document")
                }
            }
            TicketType::Receipt => {
                if let Ok(doc) = generate_receipt(&invoice) {
                    self.0.print(&doc, Some("Alegria Print Job"))
                    // std::fs::write("./text_example.pdf", doc).unwrap();
                    // Ok(())
                } else {
                    Err("Failed to generate receipt document")
                }
            }
        })
        .await
        .unwrap_or(Err("Failed to spawn a blocking task"))
    }
}

/// TODO: Proper Doc Generation (Given a Simple Invoice)
fn generate_invoice(_invoice: &SimpleInvoice) -> Result<Vec<u8>, &'static str> {
    todo!()
}

fn generate_receipt(invoice: &SimpleInvoice) -> Result<Vec<u8>, &'static str> {
    // Create a new PDF document
    let mut doc = PdfDocument::new("Hotel Receipt");

    // Load and register an external font
    let custom_font = ParsedFont::from_bytes(TICKET_FONT_TTF, 0, &mut Vec::new()).unwrap();
    let custom_font_id = doc.add_font(&custom_font);

    let (needed_doc_height, taxes) = calculate_needed_height_and_tax(invoice);
    let mut total_price = 0.0;
    let mut current_height = needed_doc_height - 10.;

    // Write the title
    let mut ops = vec![
        // Save the graphics state to allow for position resets later
        Op::SaveGraphicsState,
        // Start a text section (required for text operations)
        Op::StartTextSection,
        // Position the text cursor from the bottom left
        Op::SetTextCursor {
            pos: Point::new(Mm(10.0), Mm(current_height)),
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
            items: vec![TextItem::Text("Hotel Name Name".to_string())],
            font: custom_font_id.clone(),
        },
        // Add a line break to move down
        Op::AddLineBreak,
        // End the text section
        Op::EndTextSection,
        // Restore the graphics state
        Op::RestoreGraphicsState,
    ];

    current_height -= 5.;
    // Write the subtitle
    ops.extend(vec![
        // Save the graphics state to allow for position resets later
        Op::SaveGraphicsState,
        // Start a text section (required for text operations)
        Op::StartTextSection,
        // Position the text cursor from the bottom left
        Op::SetTextCursor {
            pos: Point::new(Mm(15.0), Mm(current_height)),
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
            items: vec![TextItem::Text(format!(
                "Factura Simplificada Nº:{}",
                &invoice.id.unwrap_or_default()
            ))],
            font: custom_font_id.clone(),
        },
        // Add a line break to move down
        Op::AddLineBreak,
        // End the text section
        Op::EndTextSection,
        // Restore the graphics state
        Op::RestoreGraphicsState,
    ]);

    // Write each product line
    current_height -= 10.;
    for product in &invoice.products {
        // Check text width
        let mut product_name = product.original_product.name.clone();
        let mut text_width = get_text_width(&custom_font.original_bytes, &product_name, 12.0);
        // Adjust this number as needed
        while text_width > 150. {
            product_name.pop();
            text_width = get_text_width(&custom_font.original_bytes, &product_name, 12.0);
        }

        // Write the product name
        ops.extend(vec![
            // Save the graphics state to allow for position resets later
            Op::SaveGraphicsState,
            // Start a text section (required for text operations)
            Op::StartTextSection,
            // Position the text cursor from the bottom left
            Op::SetTextCursor {
                pos: Point::new(Mm(5.0), Mm(current_height)),
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
                items: vec![TextItem::Text(product_name.to_string())],
                font: custom_font_id.clone(),
            },
            // End the text section
            Op::EndTextSection,
            // Restore the graphics state
            Op::RestoreGraphicsState,
        ]);

        //Write the product price
        ops.extend(vec![
            // Save the graphics state to allow for position resets later
            Op::SaveGraphicsState,
            // Start a text section (required for text operations)
            Op::StartTextSection,
            // Position the text cursor from the bottom left
            Op::SetTextCursor {
                pos: Point::new(Mm(60.0), Mm(current_height)),
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
                items: vec![TextItem::Text(format!(
                    "{:.2}€",
                    product.price.unwrap_or_default()
                ))],
                font: custom_font_id.clone(),
            },
            // Add a line break to move down
            Op::AddLineBreak,
            // End the text section
            Op::EndTextSection,
            // Restore the graphics state
            Op::RestoreGraphicsState,
        ]);

        total_price += product.price.unwrap_or_default();
        current_height -= 5.;
    }

    // Write total taxes
    for (tax_per, tax_ammount) in taxes {
        ops.extend(vec![
            // Save the graphics state to allow for position resets later
            Op::SaveGraphicsState,
            // Start a text section (required for text operations)
            Op::StartTextSection,
            // Position the text cursor from the bottom left
            Op::SetTextCursor {
                pos: Point::new(Mm(44.0), Mm(current_height)),
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
                items: vec![TextItem::Text(format!(
                    "IVA: {:.2}% {:.2}€",
                    tax_per.to_string(),
                    tax_ammount
                ))],
                font: custom_font_id.clone(),
            },
            // End the text section
            Op::EndTextSection,
            // Restore the graphics state
            Op::RestoreGraphicsState,
        ]);
        current_height -= 5.;
    }

    // Write total price
    current_height -= 10.;
    ops.extend(vec![
        // Save the graphics state to allow for position resets later
        Op::SaveGraphicsState,
        // Start a text section (required for text operations)
        Op::StartTextSection,
        // Position the text cursor from the bottom left
        Op::SetTextCursor {
            pos: Point::new(Mm(44.0), Mm(current_height)),
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
            items: vec![TextItem::Text(format!("TOTAL: {total_price:.2}€"))],
            font: custom_font_id.clone(),
        },
        // End the text section
        Op::EndTextSection,
        // Restore the graphics state
        Op::RestoreGraphicsState,
    ]);

    // Create a page with our operations
    let page = PdfPage::new(Mm(80.0), Mm(needed_doc_height), ops);

    // Save the PDF to a file
    Ok(doc
        .with_pages(vec![page])
        .save(&PdfSaveOptions::default(), &mut Vec::new()))
}

fn calculate_needed_height_and_tax(invoice: &SimpleInvoice) -> (f32, HashMap<u64, f64>) {
    let mut result = 20.; // 10 for title and 5 for subtitle and 5 for spacing between subtitle and products
    let mut tax_totals: HashMap<u64, f64> = HashMap::new();

    for product in &invoice.products {
        result += 5.; // we need 5 for each product

        // Calculate tax for the current product
        let tax_percentage =
            round_tax_percentage(&product.original_product.tax_percentage.unwrap_or(21.)); // Round tax percentage to two decimals
        let tax_amount = calculate_tax(
            &product.price,
            &product.original_product.tax_percentage.unwrap_or(21.),
        ) as f64; // This is the tax amount for this product

        // Accumulate the tax in the correct group (rounding tax percentage to two decimal places)
        let entry = tax_totals.entry(tax_percentage).or_insert(0.0);
        *entry += tax_amount;
    }

    for _ in tax_totals.keys() {
        result += 5.; // For each different tax we need 5 more space
    }

    result += 10.; // For the price
    result += 10.; // For margin bottom
    (result, tax_totals)
}

fn get_text_width(font_data: &[u8], text: &str, font_size: f32) -> f32 {
    let font = fontdue::Font::from_bytes(font_data, fontdue::FontSettings::default())
        .expect("Failed to load font");
    text.chars().fold(0.0, |acc, c| {
        let metrics = font.metrics(c, font_size);
        acc + metrics.advance_width
    })
}

fn round_tax_percentage(tax_perc: &f32) -> u64 {
    (tax_perc * 100.0).round() as u64
}

pub fn calculate_tax(price: &Option<f32>, tax_perc: &f32) -> f32 {
    if let Some(price) = price {
        return price * tax_perc / (100.0 + tax_perc);
    }
    0.0
}
