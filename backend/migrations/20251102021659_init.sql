-- Add migration script here
-- users table
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

-- accounts table
CREATE TABLE accounts (
    id SERIAL PRIMARY KEY,
    user_id INT REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(50) NOT NULL,
    type VARCHAR(20),
    currency VARCHAR(10) DEFAULT 'USD',
    balance NUMERIC(12,2) DEFAULT 0.0
);

-- categories table
CREATE TABLE categories (
    id SERIAL PRIMARY KEY,
    user_id INT REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(50) NOT NULL,
    parent_id INT REFERENCES categories(id)
);

-- transactions table
CREATE TABLE transactions (
    id SERIAL PRIMARY KEY,
    account_id INT REFERENCES accounts(id) ON DELETE CASCADE,
    category_id INT REFERENCES categories(id),
    amount NUMERIC(12,2) NOT NULL,
    type VARCHAR(10),
    date DATE DEFAULT CURRENT_DATE,
    description TEXT
);

-- budgets table
CREATE TABLE budgets (
    id SERIAL PRIMARY KEY,
    user_id INT REFERENCES users(id) ON DELETE CASCADE,
    category_id INT REFERENCES categories(id),
    amount NUMERIC(12,2) NOT NULL,
    period VARCHAR(10),
    start_date DATE DEFAULT CURRENT_DATE
);

