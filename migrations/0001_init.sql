-- Create ProductCategory table
CREATE TABLE product_categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Create Product table 
CREATE TABLE products (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    category_id INTEGER,
    name TEXT NOT NULL,
    price REAL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (category_id) REFERENCES product_categories (id)
);

-- Trigger to update 'updated_at' on ProductCategory
CREATE TRIGGER update_product_category_updated_at
AFTER UPDATE ON product_categories
FOR EACH ROW
BEGIN
    UPDATE product_categories
    SET updated_at = CURRENT_TIMESTAMP
    WHERE id = OLD.id;
END;

-- Trigger to update 'updated_at' on Product
CREATE TRIGGER update_product_updated_at
AFTER UPDATE ON products
FOR EACH ROW
BEGIN
    UPDATE products
    SET updated_at = CURRENT_TIMESTAMP
    WHERE id = OLD.id;
END;
