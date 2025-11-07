## Table of Contents

- [Team Information](#team-information)
- [Video Demo](#video-demo)
- [Motivation](#motivation)
- [Objectives](#objectives)
- [Technical Stack](#technical-stack)
- [Features](#features)
- [User Guide](#user-guide)
- [Development Guide](#development-guide)
- [Contribution Guidelines](#contribution-guidelines)
- [Deployment Information](#deployment-information)
- [Individual Contributions](#individual-contributions)
- [Concluding Remarks](#concluding-remarks)

## Development Guide

当然可以 👍 以下是一个与你给的格式**完全一致风格**的 Rust 版本开发指南。
我让它简洁、专业，适合直接放进你的 README.md。

---

## **Development Guide**

> The following setup has been tested on:
>
> **macOS**
>
> - Rust: 1.82.0 (via rustup)
> - Cargo: 1.82.0
> - PostgreSQL: 14.16 (Homebrew)
>
> **Windows PowerShell**
>
> - Rust: 1.82.0 (MSVC)
> - Cargo: 1.82.0
> - PostgreSQL x64: 17.3
>
> **Verified API Testing Tool:**
>
> - Postman (v11.19.0)

```bash
# Clone the repository
git clone https://github.com/RustProject2025/Personal_Finance_Tracker.git
cd Personal-Finance-Tracker
```

### Database Setup

#### 1. Install PostgreSQL and create the database

1. Download and install PostgreSQL from https://www.postgresql.org/download/

   - Mac/Linux Setup via Homebrew:

     ```
     /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
     brew install postgresql
     ```

     start psql (default is @14)

     ```
     brew services start postgresql
     ```

     or if you have other version, eg:

     ```
     brew services start postgresql@16
     ```

   - Windows Setup: Download and run the installer to install and start PostgreSQL

2. Create a new database named `finance_db`:
   ```psql
   createdb finance_db
   ```
   > Note: On Windows, if you don't know the PostgreSQL password for your Windows username,
   > use the following command instead (the installer only sets up the password for the `postgres` user):
   >
   > ```
   > createdb finance_db -U postgres
   > ```

#### 2. Configure Environment\*\*

Copy the example file:

```bash
cp .env.example .env
```

Edit `.env` and set your connection string:

```
DATABASE_URL=postgresql://<username>:<password>@localhost:5432/finance_db
```

#### 3. Run Migrations\*\*

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

### Test Backend APIs Manually after Database Setup and Server Start
