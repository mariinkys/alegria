-- Create ProductCategory Table
CREATE TABLE IF NOT EXISTS product_categories (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create Product Table
CREATE TABLE IF NOT EXISTS products (
    id SERIAL PRIMARY KEY,
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
    id SERIAL PRIMARY KEY,
    paid BOOLEAN NOT NULL DEFAULT FALSE,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create SoldProduct Table
CREATE TABLE IF NOT EXISTS sold_products (
    id SERIAL PRIMARY KEY,
    simple_invoice_id INTEGER NOT NULL,
    original_product_id INTEGER NOT NULL,
    price NUMERIC,
    FOREIGN KEY (simple_invoice_id)
        REFERENCES simple_invoices(id)
        ON DELETE CASCADE -- Delete sold products if invoice is deleted
);

-- Create TemporalTicket Table
CREATE TABLE IF NOT EXISTS temporal_tickets (
    id SERIAL PRIMARY KEY,
    table_id INTEGER NOT NULL,
    ticket_location INTEGER NOT NULL,
    ticket_status INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create TemporalProduct Table
CREATE TABLE IF NOT EXISTS temporal_products (
    id SERIAL PRIMARY KEY,
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
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    price REAL,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create RoomTypes Table
CREATE TABLE IF NOT EXISTS rooms (
    id SERIAL PRIMARY KEY,
    room_type_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create IdentityDocumentTypes Table
CREATE TABLE IF NOT EXISTS identity_document_types (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL
);

-- Create Genders Table
CREATE TABLE IF NOT EXISTS genders (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL
);

-- Create Clients Table
CREATE TABLE IF NOT EXISTS clients (
    id SERIAL PRIMARY KEY,
    gender_id INTEGER NOT NULL,
    identity_document_type_id INTEGER NOT NULL,
    identity_document TEXT NOT NULL,
    identity_document_expedition_date TIMESTAMP NOT NULL,
    identity_document_expiration_date TIMESTAMP NOT NULL,
    name TEXT NOT NULL,
    first_surname TEXT NOT NULL,
    second_surname TEXT NOT NULL,
    birthdate TIMESTAMP NOT NULL,
    address TEXT,
    postal_code TEXT,
    city TEXT,
    province TEXT,
    country TEXT,
    nationality TEXT,
    phone_number TEXT,
    mobile_phone TEXT,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Add Functions and Triggers to Update 'updated_at' Timestamps ------------------------------

-- Create update timestamp function
CREATE OR REPLACE FUNCTION update_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger for product_categories
CREATE TRIGGER update_product_categories_updated_at
BEFORE UPDATE ON product_categories
FOR EACH ROW
EXECUTE FUNCTION update_timestamp();

-- Trigger for products
CREATE TRIGGER update_products_updated_at
BEFORE UPDATE ON products
FOR EACH ROW
EXECUTE FUNCTION update_timestamp();

-- Trigger for simple_invoices
CREATE TRIGGER update_simple_invoices_updated_at
BEFORE UPDATE ON simple_invoices
FOR EACH ROW
EXECUTE FUNCTION update_timestamp();

-- Trigger for temporal_tickets
CREATE TRIGGER update_temporal_tickets_updated_at
BEFORE UPDATE ON temporal_tickets
FOR EACH ROW
EXECUTE FUNCTION update_timestamp();

-- Trigger for room_types
CREATE TRIGGER update_room_types_updated_at
BEFORE UPDATE ON room_types
FOR EACH ROW
EXECUTE FUNCTION update_timestamp();

-- Trigger for room_types
CREATE TRIGGER update_room_types_updated_at
BEFORE UPDATE ON rooms
FOR EACH ROW
EXECUTE FUNCTION update_timestamp();

-- Trigger for clients
CREATE TRIGGER update_clients_updated_at
BEFORE UPDATE ON clients
FOR EACH ROW
EXECUTE FUNCTION update_timestamp();