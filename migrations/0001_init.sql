-- Create ProductCategory Table
CREATE TABLE IF NOT EXISTS product_categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create Product Table
CREATE TABLE IF NOT EXISTS products (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    category_id INTEGER,
    name TEXT NOT NULL,
    inside_price REAL,
    outside_price REAL,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (category_id) REFERENCES product_categories(id) ON DELETE SET NULL
);

-- Create SimpleInvoice Table 
CREATE TABLE IF NOT EXISTS simple_invoices (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    paid INTEGER NOT NULL DEFAULT 0, -- Use INTEGER for paid field
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create SoldProduct Table 
CREATE TABLE IF NOT EXISTS sold_products (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    simple_invoice_id INTEGER NOT NULL,
    original_product_id INTEGER NOT NULL,
    price REAL,
    FOREIGN KEY (simple_invoice_id) 
        REFERENCES simple_invoices(id) 
        ON DELETE CASCADE -- Delete sold products if invoice is deleted
);

-- Create TemporalTicket Table 
CREATE TABLE IF NOT EXISTS temporal_tickets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    table_id INTEGER NOT NULL,
    ticket_location INTEGER NOT NULL,
    ticket_status INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create TemporalProduct Table 
CREATE TABLE IF NOT EXISTS temporal_products (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    original_product_id INTEGER NOT NULL,
    temporal_ticket_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 1,
    price REAL,
    FOREIGN KEY (original_product_id) 
        REFERENCES products(id) 
        ON DELETE CASCADE, -- Delete if original product is deleted
    FOREIGN KEY (temporal_ticket_id) 
        REFERENCES temporal_tickets(id) 
        ON DELETE CASCADE -- Delete products if ticket is deleted
);

-- Create RoomTypes Table 
CREATE TABLE IF NOT EXISTS room_types (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    price REAL,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Add Triggers to Update 'updated_at' Timestamps ------------------------------

-- Trigger for product_categories
CREATE TRIGGER IF NOT EXISTS update_product_categories_updated_at
AFTER UPDATE ON product_categories
BEGIN
    UPDATE product_categories 
    SET updated_at = CURRENT_TIMESTAMP 
    WHERE id = OLD.id;
END;

-- Trigger for products
CREATE TRIGGER IF NOT EXISTS update_products_updated_at
AFTER UPDATE ON products
BEGIN
    UPDATE products 
    SET updated_at = CURRENT_TIMESTAMP 
    WHERE id = OLD.id;
END;

-- Trigger for simple_invoices
CREATE TRIGGER IF NOT EXISTS update_simple_invoices_updated_at
AFTER UPDATE ON simple_invoices
BEGIN
    UPDATE simple_invoices 
    SET updated_at = CURRENT_TIMESTAMP 
    WHERE id = OLD.id;
END;

-- Trigger for temporal_tickets
CREATE TRIGGER IF NOT EXISTS update_temporal_tickets_updated_at
AFTER UPDATE ON temporal_tickets
BEGIN
    UPDATE temporal_tickets 
    SET updated_at = CURRENT_TIMESTAMP 
    WHERE id = OLD.id;
END;

-- Trigger for room_types
CREATE TRIGGER IF NOT EXISTS update_room_types_updated_at
AFTER UPDATE ON room_types
BEGIN
    UPDATE room_types 
    SET updated_at = CURRENT_TIMESTAMP 
    WHERE id = OLD.id;
END;
