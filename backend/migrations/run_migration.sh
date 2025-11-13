#!/bin/bash
set -e

echo "Running database migrations..."
sqlx migrate run --database-url "$DATABASE_URL"

if [ $? -eq 0 ]; then
    echo "Migrations completed successfully"
else
    echo "Migration failed"
    exit 1
fi

