## **1. Create Database**

Make sure PostgreSQL is running, then create the database:

```sql
CREATE DATABASE finance_db;
```

## **2. Configure Environment**

Copy the example file:

```bash
cp .env.example .env
```

Edit `.env` and set your connection string:

```
DATABASE_URL=postgresql://<username>:<password>@localhost:5432/finance_db
```

## **3. Run Migrations**

```bash
# If sqlx-cli is installed
sqlx migrate run

# Or execute the SQL file manually
psql -U <username> -d finance_db -f migrations/20251102021659_init.sql
```

## **4. Start the Server**

```bash
cargo run
```

backend will start at http://127.0.0.1:3000
