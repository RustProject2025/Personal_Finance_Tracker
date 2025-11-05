-- Add migration script here
-- users table (for registration)
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    password_hash TEXT NOT NULL, -- Argon2 hashed password
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- sessions table (for login/logout and session validation)
CREATE TABLE sessions (
    id SERIAL PRIMARY KEY,
    user_id INT REFERENCES users(id) ON DELETE CASCADE,
    token VARCHAR(255) UNIQUE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    is_valid BOOLEAN DEFAULT TRUE
);

CREATE TYPE account_type AS ENUM ('Checking', 'Savings', 'Credit Card', 'Cash');
-- accounts table
CREATE TABLE accounts (
    id SERIAL PRIMARY KEY,
    user_id INT REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(50) NOT NULL,
    type account_type NOT NULL, -- Use ENUM instead of VARCHAR
    currency VARCHAR(10) DEFAULT 'USD',
    balance NUMERIC(12,2) DEFAULT 0.0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- categories table
CREATE TABLE categories (
    id SERIAL PRIMARY KEY,
    user_id INT REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(50) NOT NULL,
    parent_id INT REFERENCES categories(id),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- transactions table
CREATE TABLE transactions (
    id SERIAL PRIMARY KEY,
    user_id INT REFERENCES users(id) ON DELETE CASCADE, -- Added for security
    account_id INT REFERENCES accounts(id) ON DELETE CASCADE,
    category_id INT REFERENCES categories(id) ON DELETE SET NULL, -- NULL for transfers
    amount NUMERIC(12,2) NOT NULL, -- Can be positive (income) or negative (expense)
    type VARCHAR(20) NOT NULL, -- 'income', 'expense', 'transfer'
    date DATE DEFAULT CURRENT_DATE,
    description TEXT, -- Notes/description field
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- budgets table
CREATE TABLE budgets (
    id SERIAL PRIMARY KEY,
    user_id INT REFERENCES users(id) ON DELETE CASCADE,
    category_id INT REFERENCES categories(id) ON DELETE CASCADE, -- NULL means monthly budget
    amount NUMERIC(12,2) NOT NULL,
    period VARCHAR(10), -- 'monthly' or other periods
    start_date DATE DEFAULT CURRENT_DATE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

