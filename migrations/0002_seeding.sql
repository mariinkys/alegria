-- Seed product categories
INSERT INTO product_categories (name, is_deleted) VALUES ('Carne', false);
INSERT INTO product_categories (name, is_deleted) VALUES ('Postre', false);

-- Seed products
INSERT INTO products (category_id, name, inside_price, outside_price, tax_percentage, is_deleted) VALUES (1, 'Filete', 9.80, 9.90, 21.0, false);
INSERT INTO products (category_id, name, inside_price, outside_price, tax_percentage, is_deleted) VALUES (1, 'Pechuga de Coco', 1.00, 1.50, 21.0, false);
INSERT INTO products (category_id, name, inside_price, outside_price, tax_percentage, is_deleted) VALUES (2, 'Crema de Lima', 1.20, 1.30, 21.0, false);

-- Seed room types
INSERT INTO room_types (name, price) VALUES ('Individual', 60.00);
INSERT INTO room_types (name, price) VALUES ('Doble', 75.00);

-- Seed rooms 
INSERT INTO rooms (name, room_type_id) VALUES ('113', 2);
INSERT INTO rooms (name, room_type_id) VALUES ('112', 1);
INSERT INTO rooms (name, room_type_id) VALUES ('213', 2);

-- Seed identity document types (this table can not be altered by the user)
INSERT INTO identity_document_types (name) VALUES ('DNI'); -- At least one document type has to exist on the db for the app to work properly (with id 1)
INSERT INTO identity_document_types (name) VALUES ('NIE');
INSERT INTO identity_document_types (name) VALUES ('NIF');
INSERT INTO identity_document_types (name) VALUES ('Pasaporte');
INSERT INTO identity_document_types (name) VALUES ('Carnet de Conducir');

-- Seed genders (this table can not be altered by the user)
INSERT INTO genders (name) VALUES ('Hombre'); -- At least one document type has to exist on the db for the app to work properly (with id 1)
INSERT INTO genders (name) VALUES ('Mujer');

-- Seed payment_methods (this table can not be altered by the user)
INSERT INTO payment_methods (name) VALUES ('Efectivo'); -- At least one document type has to exist on the db for the app to work properly (with id 1)
INSERT INTO payment_methods (name) VALUES ('Tarjeta');
INSERT INTO payment_methods (name) VALUES ('Adeudo');
