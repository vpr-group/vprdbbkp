-- Create a customers database
CREATE DATABASE IF NOT EXISTS customers;
USE customers;

-- Create customers table
CREATE TABLE IF NOT EXISTS customer (
    id INT AUTO_INCREMENT PRIMARY KEY,
    first_name VARCHAR(50) NOT NULL,
    last_name VARCHAR(50) NOT NULL,
    email VARCHAR(100) NOT NULL UNIQUE,
    phone VARCHAR(20),
    address VARCHAR(255),
    city VARCHAR(50),
    state VARCHAR(50),
    country VARCHAR(50),
    postal_code VARCHAR(20),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create subscriptions table
CREATE TABLE IF NOT EXISTS subscription (
    id INT AUTO_INCREMENT PRIMARY KEY,
    customer_id INT NOT NULL,
    plan_name VARCHAR(50) NOT NULL,
    price DECIMAL(10, 2) NOT NULL,
    billing_cycle ENUM('monthly', 'quarterly', 'annual') NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE,
    status ENUM('active', 'expired', 'cancelled') NOT NULL,
    FOREIGN KEY (customer_id) REFERENCES customer(id)
);

-- Create invoices table
CREATE TABLE IF NOT EXISTS invoice (
    id INT AUTO_INCREMENT PRIMARY KEY,
    customer_id INT NOT NULL,
    subscription_id INT NOT NULL,
    amount DECIMAL(10, 2) NOT NULL,
    issue_date DATE NOT NULL,
    due_date DATE NOT NULL,
    paid BOOLEAN DEFAULT FALSE,
    FOREIGN KEY (customer_id) REFERENCES customer(id),
    FOREIGN KEY (subscription_id) REFERENCES subscription(id)
);

-- Insert sample customers
INSERT INTO customer (first_name, last_name, email, phone, address, city, state, country, postal_code) VALUES
('John', 'Smith', 'john.smith@example.com', '555-123-4567', '123 Main St', 'Boston', 'MA', 'USA', '02108'),
('Jane', 'Doe', 'jane.doe@example.com', '555-234-5678', '456 Oak Ave', 'San Francisco', 'CA', 'USA', '94102'),
('Robert', 'Johnson', 'robert.johnson@example.com', '555-345-6789', '789 Pine Rd', 'Chicago', 'IL', 'USA', '60601'),
('Maria', 'Garcia', 'maria.garcia@example.com', '555-456-7890', '321 Elm St', 'Miami', 'FL', 'USA', '33101'),
('James', 'Wilson', 'james.wilson@example.com', '555-567-8901', '654 Maple Dr', 'Seattle', 'WA', 'USA', '98101');

-- Insert sample subscriptions
INSERT INTO subscription (customer_id, plan_name, price, billing_cycle, start_date, end_date, status) VALUES
(1, 'Basic', 9.99, 'monthly', '2023-01-01', '2024-01-01', 'active'),
(2, 'Premium', 29.99, 'monthly', '2023-02-15', NULL, 'active'),
(3, 'Enterprise', 99.99, 'annual', '2023-03-10', '2024-03-10', 'active'),
(4, 'Basic', 9.99, 'monthly', '2023-01-20', '2023-07-20', 'cancelled'),
(5, 'Premium', 29.99, 'quarterly', '2023-04-05', NULL, 'active'),
(1, 'Premium', 29.99, 'monthly', '2024-01-01', NULL, 'active');

-- Insert sample invoices
INSERT INTO invoice (customer_id, subscription_id, amount, issue_date, due_date, paid) VALUES
(1, 1, 9.99, '2023-01-01', '2023-01-15', TRUE),
(1, 1, 9.99, '2023-02-01', '2023-02-15', TRUE),
(1, 1, 9.99, '2023-03-01', '2023-03-15', TRUE),
(2, 2, 29.99, '2023-02-15', '2023-03-01', TRUE),
(2, 2, 29.99, '2023-03-15', '2023-04-01', TRUE),
(3, 3, 99.99, '2023-03-10', '2023-03-24', TRUE),
(4, 4, 9.99, '2023-01-20', '2023-02-03', TRUE),
(4, 4, 9.99, '2023-02-20', '2023-03-06', TRUE),
(5, 5, 89.97, '2023-04-05', '2023-04-19', TRUE),
(1, 6, 29.99, '2024-01-01', '2024-01-15', TRUE);

-- Create a separate inventory database
CREATE DATABASE IF NOT EXISTS inventory;
USE inventory;

-- Create categories table
CREATE TABLE IF NOT EXISTS category (
    id INT AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(50) NOT NULL,
    description TEXT
);

-- Create products table
CREATE TABLE IF NOT EXISTS product (
    id INT AUTO_INCREMENT PRIMARY KEY,
    category_id INT NOT NULL,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    sku VARCHAR(50) UNIQUE NOT NULL,
    price DECIMAL(10, 2) NOT NULL,
    stock INT NOT NULL DEFAULT 0,
    FOREIGN KEY (category_id) REFERENCES category(id)
);

-- Create suppliers table
CREATE TABLE IF NOT EXISTS supplier (
    id INT AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    contact_name VARCHAR(100),
    email VARCHAR(100),
    phone VARCHAR(20),
    address TEXT
);

-- Create purchases table
CREATE TABLE IF NOT EXISTS purchase (
    id INT AUTO_INCREMENT PRIMARY KEY,
    supplier_id INT NOT NULL,
    purchase_date DATE NOT NULL,
    total_amount DECIMAL(10, 2) NOT NULL,
    FOREIGN KEY (supplier_id) REFERENCES supplier(id)
);

-- Create purchase_items table
CREATE TABLE IF NOT EXISTS purchase_item (
    id INT AUTO_INCREMENT PRIMARY KEY,
    purchase_id INT NOT NULL,
    product_id INT NOT NULL,
    quantity INT NOT NULL,
    unit_price DECIMAL(10, 2) NOT NULL,
    FOREIGN KEY (purchase_id) REFERENCES purchase(id),
    FOREIGN KEY (product_id) REFERENCES product(id)
);

-- Insert sample categories
INSERT INTO category (name, description) VALUES
('Electronics', 'Electronic devices and accessories'),
('Books', 'Books of various genres'),
('Clothing', 'Apparel for men, women, and children'),
('Home & Garden', 'Products for home improvement and gardening'),
('Toys', 'Children toys and games');

-- Insert sample products
INSERT INTO product (category_id, name, description, sku, price, stock) VALUES
(1, 'Smartphone X', 'Latest smartphone with advanced features', 'ELPHN-001', 899.99, 50),
(1, 'Laptop Pro', '15-inch laptop with high performance', 'ELLPT-002', 1299.99, 30),
(2, 'SQL Programming Guide', 'Comprehensive guide to SQL programming', 'BKSQL-001', 39.99, 100),
(2, 'Rust for Beginners', 'Introduction to Rust programming language', 'BKRST-002', 45.99, 75),
(3, 'T-shirt', 'Cotton T-shirt available in various colors', 'CLTSH-001', 19.99, 200),
(4, 'Garden Tools Set', 'Set of essential garden tools', 'HMGRD-001', 59.99, 25),
(5, 'Building Blocks', 'Creative building blocks for children', 'TYBLD-001', 29.99, 60);

-- Insert sample suppliers
INSERT INTO supplier (name, contact_name, email, phone, address) VALUES
('TechSuppliers Inc.', 'Mark Johnson', 'mark@techsuppliers.com', '555-789-0123', '789 Tech Blvd, San Jose, CA 95110'),
('BookWholesale Ltd.', 'Susan Brown', 'susan@bookwholesale.com', '555-890-1234', '456 Read St, New York, NY 10001'),
('FashionLine', 'Michael Lee', 'michael@fashionline.com', '555-901-2345', '123 Style Ave, Los Angeles, CA 90001'),
('HomeEssentials', 'Laura White', 'laura@homeessentials.com', '555-012-3456', '321 Home Rd, Chicago, IL 60601'),
('ToyWorld', 'David Miller', 'david@toyworld.com', '555-123-4567', '654 Play Dr, Orlando, FL 32801');

-- Insert sample purchases
INSERT INTO purchase (supplier_id, purchase_date, total_amount) VALUES
(1, '2023-01-15', 29999.70),
(1, '2023-02-20', 15999.80),
(2, '2023-01-25', 4299.00),
(3, '2023-03-05', 3998.00),
(4, '2023-02-10', 1499.75),
(5, '2023-03-15', 1799.40);

-- Insert sample purchase items
INSERT INTO purchase_item (purchase_id, product_id, quantity, unit_price) VALUES
(1, 1, 20, 799.99),
(1, 2, 10, 1199.99),
(2, 2, 10, 1199.99),
(2, 1, 5, 799.99),
(3, 3, 50, 35.99),
(3, 4, 40, 42.99),
(4, 5, 200, 19.99),
(5, 6, 25, 59.99),
(6, 7, 60, 29.99);