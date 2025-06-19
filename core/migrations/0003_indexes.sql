-- Speed up bar page (upsert_ticket_by_id_and_tableloc - TemporalTicket)
CREATE INDEX idx_products_id ON products(id);

CREATE INDEX idx_temporal_tickets_table_loc 
ON temporal_tickets(table_id, ticket_location);

CREATE INDEX idx_temporal_products_ticket_id 
ON temporal_products(temporal_ticket_id);
