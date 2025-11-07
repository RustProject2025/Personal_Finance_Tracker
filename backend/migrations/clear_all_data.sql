-- Clear all data from all tables
-- This script will delete all local data but keep the table structure

TRUNCATE TABLE 
    transactions,
    budgets,
    accounts,
    categories,
    sessions,
    users
CASCADE;

-- Reset all sequences to start from 1
ALTER SEQUENCE users_id_seq RESTART WITH 1;
ALTER SEQUENCE sessions_id_seq RESTART WITH 1;
ALTER SEQUENCE accounts_id_seq RESTART WITH 1;
ALTER SEQUENCE categories_id_seq RESTART WITH 1;
ALTER SEQUENCE transactions_id_seq RESTART WITH 1;
ALTER SEQUENCE budgets_id_seq RESTART WITH 1;


