-- Your SQL goes here
CREATE TABLE stocks (
    id SERIAL PRIMARY KEY,
    product_id INTEGER UNIQUE
);

CREATE TABLE stock (
    id           SERIAL PRIMARY KEY,
    foreign_id   INTEGER NOT NULL UNIQUE,
    availability VARCHAR(255) NOT NULL,
    type_threshold VARCHAR(255) NOT NULL,
    stocks INT REFERENCES stocks (id) ON UPDATE CASCADE
);
