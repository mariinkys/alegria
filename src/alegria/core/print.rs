// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use printers::{
    common::base::printer::{Printer, PrinterState},
    get_default_printer, get_printers,
};

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
            // TODO: Document Generation
            self.0
                .print("test print".as_bytes(), Some("Alegria Print Job"))
        })
        .await
        .unwrap()
    }
}
