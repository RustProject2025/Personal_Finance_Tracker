#!/bin/sh
set -e

echo "Waiting for database to be ready..."
sleep 5

echo "Running database migrations..."
sqlx migrate run || {
    echo "Migration failed!"
    exit 1
}

echo "Starting backend server..."
exec ./backend

