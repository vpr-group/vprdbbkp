-- Create a test schema
CREATE SCHEMA test_schema;

-- Create users table
CREATE TABLE test_schema.users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) NOT NULL,
    email VARCHAR(100) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create posts table
CREATE TABLE test_schema.posts (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES test_schema.users(id),
    title VARCHAR(200) NOT NULL,
    content TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create comments table
CREATE TABLE test_schema.comments (
    id SERIAL PRIMARY KEY,
    post_id INTEGER REFERENCES test_schema.posts(id),
    user_id INTEGER REFERENCES test_schema.users(id),
    content TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Insert sample users
INSERT INTO test_schema.users (username, email) VALUES
('john_doe', 'john@example.com'),
('jane_smith', 'jane@example.com'),
('bob_johnson', 'bob@example.com'),
('alice_brown', 'alice@example.com'),
('charlie_davis', 'charlie@example.com');

-- Insert sample posts
INSERT INTO test_schema.posts (user_id, title, content) VALUES
(1, 'First Post', 'This is the content of the first post.'),
(2, 'About Databases', 'Databases are essential for storing structured data.'),
(1, 'Rust Programming', 'Rust is a great language for systems programming.'),
(3, 'Backup Strategies', 'Regular backups are crucial for data safety.'),
(4, 'Database Performance', 'Indexing is key to database performance.');

-- Insert sample comments
INSERT INTO test_schema.comments (post_id, user_id, content) VALUES
(1, 2, 'Great first post!'),
(1, 3, 'Looking forward to more content.'),
(2, 1, 'I agree, databases are fundamental.'),
(3, 4, 'Rust has been my favorite language recently.'),
(3, 5, 'The memory safety features are impressive.'),
(4, 2, 'What backup tools do you recommend?'),
(5, 3, 'Indexing made a huge difference in my queries.');

-- Create a second schema for another application
CREATE SCHEMA app2_schema;

-- Create products table
CREATE TABLE app2_schema.products (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    price DECIMAL(10, 2) NOT NULL,
    stock INTEGER NOT NULL DEFAULT 0
);

-- Create orders table
CREATE TABLE app2_schema.orders (
    id SERIAL PRIMARY KEY,
    customer_name VARCHAR(100) NOT NULL,
    customer_email VARCHAR(100) NOT NULL,
    order_date TIMESTAMP NOT NULL DEFAULT NOW(),
    total_amount DECIMAL(10, 2) NOT NULL
);

-- Create order_items table
CREATE TABLE app2_schema.order_items (
    id SERIAL PRIMARY KEY,
    order_id INTEGER REFERENCES app2_schema.orders(id),
    product_id INTEGER REFERENCES app2_schema.products(id),
    quantity INTEGER NOT NULL,
    price_per_unit DECIMAL(10, 2) NOT NULL
);

-- Insert sample products
INSERT INTO app2_schema.products (name, description, price, stock) VALUES
('Laptop', 'High-performance laptop with SSD', 1299.99, 15),
('Smartphone', 'Latest model with dual camera', 799.99, 25),
('Headphones', 'Noise-canceling wireless headphones', 249.99, 30),
('Tablet', '10-inch display with long battery life', 499.99, 20),
('Monitor', '27-inch 4K monitor', 349.99, 10);

-- Insert sample orders
INSERT INTO app2_schema.orders (customer_name, customer_email, total_amount) VALUES
('Michael Wilson', 'michael@example.com', 1549.98),
('Sarah Adams', 'sarah@example.com', 799.99),
('David Lee', 'david@example.com', 849.98),
('Emily Garcia', 'emily@example.com', 1299.99);

-- Insert sample order items
INSERT INTO app2_schema.order_items (order_id, product_id, quantity, price_per_unit) VALUES
(1, 1, 1, 1299.99),
(1, 3, 1, 249.99),
(2, 2, 1, 799.99),
(3, 3, 2, 249.99),
(3, 5, 1, 349.99),
(4, 1, 1, 1299.99);