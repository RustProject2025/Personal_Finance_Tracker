## Table of Contents

- [Team Information](#team-information)
- [Video Demo](#video-demo)
- [Motivation](#motivation)
- [Objectives](#objectives)
- [Features](#features)
- [User Guide](#user-guide)
- [Reproducibility Guide](#reproducibility-guide)
- [Contribution Guidelines](#contribution-guidelines)
- [Deployment Information](#deployment-information)
- [Individual Contributions](#individual-contributions)
- [Concluding Remarks](#concluding-remarks)

---

## Team Information

| Name        | Student Number | Email                      |
| ----------- | -------------- | -------------------------- |
| Shiyao Sun  | 1234567890     | @mail.utoronto.ca          |
| Yiyang Wang | 1010033278     | ydev.wang@mail.utoronto.ca |

---

# Video Slide Presentation

---

# Video Demo

---

# Features

---

# User Guide

> **Pre-configurataion:**

> cd frontend

> cargo run

> This is all you need to test the app.

---

# Reproducibility Guide

### Development Guide

The following setup has been tested on:

**macOS:**

- Rust: 1.82.0 (via rustup)
- Cargo: 1.82.0
- PostgreSQL: 14.16 (Homebrew)

**Windows PowerShell:**

- Rust: 1.82.0 (MSVC)
- Cargo: 1.82.0
- PostgreSQL x64: 17.3

**Verified API Testing Tool:**

- Postman (v11.19.0)

**Verified Terminal:**

- Terminal.app (macOS)
- PowerShell (Windows)
- iTerm2 (macOS)

### Running Option 0: Frontend Setup

Under `frontend` folder:

```bash
cd frontend
```

**Install Rust dependencies (if needed):**

```bash
cargo build
```

**Start the frontend TUI application:**

```bash
cargo run
```

This will start the terminal-based user interface (TUI) application. The frontend is configured to automatically try the deployed API first, and fall back to localhost if the deployed version is unavailable.

> **Note:** The frontend is currently configured to use the deployed API at `https://personal-finance-tracker-8mem5.ondigitalocean.app/api` by default, so you don't need to do anything about database or backend setup if you just want to test the app.

If you want to test with a local backend server, You can following the steps below. (You may change the DEPLOYED_URL with `http://localhost:3000/api` or None in frontend/src/api.rs.)

### Running Option 1: Docker Instructions

If you use Docker to run the backend and database:

Make sure your Docker app is running.

**Start the application:**

```bash
cd backend
docker compose up --build
```

**Stop the application (keeping containers):**

```bash
docker compose stop
```

**Stop and remove containers, networks, and volumes:**

```bash
docker compose down
```

### Running Option 2: Local Development Setup

If you don't user docker then you need to setup enviornments and do the following steps:

**Clone the repository:**

```bash
git clone https://github.com/RustProject2025/Personal_Finance_Tracker.git
cd Personal_Finance_Tracker
```

### Database Setup

#### Install PostgreSQL and create the database

1. Download and install PostgreSQL from https://www.postgresql.org/download/

   - Mac/Linux Setup via Homebrew:

     ```
     /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
     brew install postgresql
     ```

     Start PostgreSQL (default is @14):

     ```
     brew services start postgresql
     ```

     If your version is different, e.g., PostgreSQL 16:

     ```
     brew services start postgresql@16
     ```

   - Windows Setup: Download and run the installer to install and start PostgreSQL

2. Create a new database named `finance_db`:

   ```
   createdb finance_db
   ```

   > Note: On Windows, if you don't know the PostgreSQL password for your Windows username, use the following command instead (the installer only sets up the password for the `postgres` user):
   >
   > ```
   > createdb finance_db -U postgres
   > ```

#### Configure Environment

1. Navigate to the backend folder:

   ```bash
   cd backend
   ```

2. Copy the example file:

```bash
cp .env.example .env
```

Edit `.env` and set your connection string:

```
DATABASE_URL=postgresql://<username>:<password>@localhost:5432/finance_db
```

> Note: You need to change `<username>` to your PostgreSQL username and `<password>` to your PostgreSQL password.

#### Run Migrations

```bash
# If sqlx-cli is installed globally
sqlx migrate run

# Or execute the SQL file manually
psql -U <username> -d finance_db -f migrations/20251102021659_init.sql
```

> **Reset and re-create the database schema (Optional, only if you want to reset the database from a previous schema)**
>
> 1. Drop and recreate the database:
>    ```
>    dropdb finance_db
>    createdb finance_db
>    ```
> 2. Run migrations again:
>    ```
>    sqlx migrate run
>    ```

### Seed the Database

Under the project root folder:

1. Install Python requests library (if not already installed):

   ```bash
   pip install requests
   ```

2. Seed initial data (run once per fresh setup):

   ```bash
   python inject_data.py
   ```

   This creates:

   - Demo user accounts for testing
   - Sample accounts, categories, transactions, and budgets

3. Re-seed (optional):

   - You can run `python inject_data.py` again to add the same baseline data if you cleared the tables.
   - For a clean reset, drop and recreate the database, then run migrations and seed again.

### Run the Backend Server after Database Setup

Under `backend` folder:

1. Install dependencies:

   ```bash
   cargo build
   ```

2. Start the server:

   ```bash
   cargo run
   ```

   OR for development with auto-reload:

   ```bash
   cargo watch -x run
   ```

3. The backend will start at `http://127.0.0.1:3000`

### Test Backend APIs Manually after Database Setup and Server Start

After running the seed script, you can test the APIs by using the Postman Collection or access the Swagger UI.

### Postman Collection Demo

#### 1. Create a Workspace

Start by creating a new workspace in Postman.

#### 2. Import API Collection

Import the `Finance_Tracker.postman_collection.json` file into your workspace.

#### 3. Create an Environment

In the top-right corner, create a new environment.  
The environment is used to store the authentication token after login (the token is automatically saved via the Postman script in the login API's response).

> **Note:**  
> You must run the **Login API** first to authenticate and get the token before accessing other APIs. Check out environment variables.

#### API Usage Made Easy

All sample inputs (parameters, request bodies) are pre-configured.  
As a developer, you do not need to manually input anything — just select the API you want to test and click Send.

![Image](https://github.com/user-attachments/assets/71fd90b6-6351-4add-990e-090118441d2b)

### (Optional) View API Documentation

**Postman Workspace:**
https://interview-9310.postman.co/workspace/Finance_Tracker~3a938086-f2db-4498-82e3-787d1241b5e4/collection/41343257-f318989d-c7b4-42f2-b605-f0c753b0b333?action=share&creator=41343257&active-environment=41343257-e23b5276-cfcb-4145-baaf-0b80e54ff407

**Swagger UI:**

1. Open: [https://editor.swagger.io/](https://editor.swagger.io/)
2. Go to **File → Import File**
3. Select: `Finance_Tracker.yaml`

### Deployment Information

**Cloud API Endpoint:**
The backend and database are successfully deployed. For future use, the cloud API prefix is:

`https://personal-finance-tracker-8mem5.ondigitalocean.app/`

For example, change:

```
http://localhost:3000/api/auth/register
```

to:

```
https://personal-finance-tracker-8mem5.ondigitalocean.app/api/auth/login
```

The frontend automatically tries the deployed version first and falls back to localhost if unavailable.

---

## Contribution Guidelines

We welcome contributions to Personal Finance Tracker! This document outlines the process and guidelines for contributing to the project.

### Getting Started

1. **Fork the repository**

2. **Clone your fork**

   ```
   git clone https://github.com/your-username/Personal_Finance_Tracker.git
   ```

3. **Set up your development environment** following the Development Guide above

4. **Create a new branch for your feature/fix:**

   ```
   git checkout -b feature/your-feature-name
   ```

### Development Process

**Follow the development guide:**

- Setup database and environments
- Check APIs using Postman or Swagger

**Make your changes following our coding standards:**

- Use Rust for both frontend and backend development
- Follow Rust naming conventions and style guidelines
- Write clear, descriptive commit messages
- Add comments for complex logic
- Update documentation as needed
- Ensure all code compiles without warnings

**Test your changes:**

- Test using the Postman Collection
- Test the TUI frontend in the terminal
- Verify database migrations work correctly
- Test error handling and edge cases

**Commit your changes:**

```
git add .
git commit -m "Description of your changes"
```

**Push to your fork:**

```
git push origin feature/your-feature-name
```

### Pull Request Process

1. Create a Pull Request (PR) from your fork to the main repository
2. Ensure your PR description clearly describes the problem and solution
3. Include the relevant issue number if applicable
4. The PR will be reviewed by our team members
5. Address any feedback or requested changes
6. Once approved, your PR will be merged

### Code Review Guidelines

- All code must be reviewed by at least one team member
- Code should be well-documented and follow Rust coding standards
- Tests should be included for new features where applicable
- The PR should not introduce any new compilation warnings or errors
- The changes should be focused and not include unrelated modifications
- Database migrations should be backward compatible when possible

### Questions or Problems?

If you have any questions or run into problems, please:

- Check the Development Guide above
- Open an issue in the repository
- Contact the team members listed in Team Information

---

## Individual Contributions

### Shiyao Sun

- Frontend:

### Yiyang Wang

- Backend:
  - Designed and implemented PostgreSQL database schema with 6 core tables supporting user isolation and data integrity
  - Implemented user authentication and authorization using Argon2 password hashing and token-based session management
  - Developed RESTful API endpoints using Axum framework for accounts, transactions, categories, budgets, and transfers
  - Implemented input validation and error handling with balance checks for transfers and user permission verification
  - Created Docker Compose configuration to orchestrate database and backend services for local development
