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

docker guide:

to run backend:
`cd backend`

`docker compose up --build`

control + C to stop

backend and database sucessfully deployed. currently the app is archieved to save cost.
for future use,
the cloud API prefix is:

`https://personal-finance-tracker-8mem5.ondigitalocean.app/`

for example, change

`http://localhost:3000/api/auth/register`

to:

`https://personal-finance-tracker-8mem5.ondigitalocean.app/api/auth/login`

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
cd backend
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
#Backend runing the backend In the Docker environment
cargo run
```
```bash

#Before running the frontend, you can inject some sample data
#if you don't have the requests, you can use this command to install the requests
pip install requests
#inject the data
python inject_data.py

#frontend runing in the terminal window
cd frontend
cargo run
```

backend will start at http://127.0.0.1:3000

### Test Backend APIs Manually

Follow the steps below once the database is set up and the backend server is running.

---

### 1. Download and Install Postman

[https://www.postman.com/downloads/](https://www.postman.com/downloads/)

---

### 2. Import the API Collection

1. Open Postman
2. Go to **Import**
3. Select the file:
   `Finance_Tracker.postman_collection.json`
4. Import it into your workspace

---

### 3. Create an Environment

1. In the top-right corner, click **No Environment → New Environment**
2. This environment will store your auth token automatically after login

> **Important:**
> You must call the **Login API first** so that the authentication token is saved.
> Other APIs require this token to access.

---

### 4. Start Testing APIs

All test inputs (request bodies, parameters, headers) are **pre-filled**.
You do **NOT** need to enter anything manually—just:

1. Select an API
2. Click **Send**

## View API Documentation (OpenAPI)

https://interview-9310.postman.co/workspace/Finance_Tracker~3a938086-f2db-4498-82e3-787d1241b5e4/collection/41343257-f318989d-c7b4-42f2-b605-f0c753b0b333?action=share&creator=41343257&active-environment=41343257-e23b5276-cfcb-4145-baaf-0b80e54ff407

You can also view the API documentation in Swagger:

1. Open: [https://editor.swagger.io/](https://editor.swagger.io/)
2. Go to **File → Import File**
3. Select:

   ```
   Finance_Tracker.yaml
   ```
