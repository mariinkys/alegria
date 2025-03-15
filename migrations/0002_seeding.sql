-- Seed product categories
INSERT INTO product_categories (name, is_deleted) VALUES ('Carne', false);
INSERT INTO product_categories (name, is_deleted) VALUES ('Postre', false);

-- Seed products
INSERT INTO products (category_id, name, inside_price, outside_price, is_deleted) VALUES (1, 'Filete', 9.80, 9.90, false);
INSERT INTO products (category_id, name, inside_price, outside_price, is_deleted) VALUES (1, 'Pechuga de Coco', 1.00, 1.50, false);
INSERT INTO products (category_id, name, inside_price, outside_price, is_deleted) VALUES (2, 'Crema de Lima', 1.20, 1.30, false);

-- Seed room types
INSERT INTO room_types (name, price) VALUES ('Individual', 60.00);
INSERT INTO room_types (name, price) VALUES ('Doble', 75.00);

-- Seed rooms 
INSERT INTO rooms (name, room_type_id) VALUES ('113', 2);
INSERT INTO rooms (name, room_type_id) VALUES ('112', 1);
INSERT INTO rooms (name, room_type_id) VALUES ('213', 2);
