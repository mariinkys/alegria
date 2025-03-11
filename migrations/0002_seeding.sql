-- Seed product categories
INSERT INTO product_categories (name, is_deleted) VALUES ('Carne', 0);
INSERT INTO product_categories (name, is_deleted) VALUES ('Postre', 0);

-- Seed products
INSERT INTO products (category_id, name, inside_price, outside_price, is_deleted) VALUES (1, 'Filete', 9.80, 9.90, 0);
INSERT INTO products (category_id, name, inside_price, outside_price, is_deleted) VALUES (1, 'Pechuga de Coco', 1.00, 1.50, 0);
INSERT INTO products (category_id, name, inside_price, outside_price, is_deleted) VALUES (2, 'Crema de Lima', 1.20, 1.30, 0);

-- Seed room types
INSERT INTO room_types (name, price) VALUES ('Individual', 60.00);
INSERT INTO room_types (name, price) VALUES ('Doble', 75.00);
