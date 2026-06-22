-- up
CREATE TABLE deliveries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tracking_number TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL,
    sender TEXT NOT NULL,
    recipient TEXT NOT NULL,
    current_status TEXT NOT NULL,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL
);

CREATE TABLE status_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    delivery_id INTEGER NOT NULL,
    status TEXT NOT NULL,
    changed_at DATETIME NOT NULL,
    FOREIGN KEY(delivery_id) REFERENCES deliveries(id) ON DELETE CASCADE
);

CREATE INDEX idx_deliveries_tracking_number ON deliveries(tracking_number);
CREATE INDEX idx_status_history_delivery_id ON status_history(delivery_id);