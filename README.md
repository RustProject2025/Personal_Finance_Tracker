## Table of Contents

- [Team Information](#team-information)
- [Video Slide Presentation](#video-slide-presentation)
- [Video Demo](#video-demo)
- [Motivation](#motivation)
- [Objectives](#objectives)
- [Features](#features)
- [User Guide](#user-guide)
- [Reproducibility Guide](#reproducibility-guide)
- [Contribution Guidelines](#contribution-guidelines)
- [Individual Contributions](#individual-contributions)
- [Concluding Remarks](#concluding-remarks)

---

## Team Information

| Name        | Student Number | Email                      |
| ----------- | -------------- | -------------------------- |
| Shiyao Sun  | 1006769793     | shiyao.sun@mail.utoronto.ca          |
| Yiyang Wang | 1010033278     | ydev.wang@mail.utoronto.ca |

---

# Video Slide Presentation

---

# Video Demo

https://drive.google.com/file/d/1F-4DSNa9Lci5kR8WsPY1DB7OvPpIye7v/view?usp=drive_link

---


# Motivation

For students and early-career professionals, managing finances across scattered checking accounts, credit cards, cash, and e-wallets often leads to disorganized records and painful reconciliation. Existing solutions typically fall into three categories with distinct drawbacks:
* **Commercial Apps:** Often come with high subscription costs and potential privacy risks regarding sensitive financial data.
* **Bank Apps:** Limited to a single institution, lacking a unified view across multiple assets and robust budgeting tools.
* **Spreadsheets:** Flexible but require high maintenance, lacking automation and real-time capabilities.

This project aims to build a **Rust-based Full-Stack Personal Finance Tracker**, combining a high-performance Terminal User Interface (TUI) with a secure HTTPS backend. Our motivation is to provide a **zero-cost, local-first, and fully controllable** financial management tool for privacy-conscious technical users. By leveraging Rust's strong type system and memory safety, we solve the issues of "bloated" and "opaque" traditional financial software, allowing users to efficiently control their cash flow via keyboard interactions.

# Objectives

The core objective of this project is to develop a lightweight, high-performance, and extensible personal finance system. Our technical goals include:

1.  **High-Performance Backend:** Develop a RESTful API using the **Axum** framework and **SQLx** for asynchronous database operations, ensuring high concurrency handling and type safety.
2.  **Immersive Terminal Experience:** Build a text-based user interface using **Ratatui**, tailored for developers and geeks who prefer pure keyboard navigation, instant data refreshing, and intuitive dashboards.
3.  **Data Integrity & Security:** Guarantee financial data accuracy through **PostgreSQL** transactions (ACID compliance) and secure user data with **Argon2** password hashing and token-based session management.
4.  **Modern Deployment Architecture:** Adopt a decoupled frontend-backend architecture with **Docker** and **Docker Compose** support, facilitating both local execution and cloud deployment (e.g., DigitalOcean).


# Features

This project fulfills the requirements of a robust command-line financial utility by combining a secure, persistent backend with a responsive terminal interface.

### 1. Secure HTTPS Backend & Database


* **Implementation:** Built with **Axum** (Rust) exposing a RESTful API. Data is persisted in **PostgreSQL** using **SQLx** for async database interaction.
* **Security:**
    * **Authentication:** User passwords are securely hashed using **Argon2** (via `auth.rs`).
    * **Session Management:** Implements token-based authentication with session expiration and invalidation logic (`middleware.rs`).
    * **Cloud Ready:** Dockerized architecture allows the backend to be deployed on platforms like DigitalOcean with HTTPS support, while the frontend automatically switches between local and remote endpoints (`api.rs`).

### 2. Multi-Type Account Management


* **Implementation:** The system supports creating arbitrary account types (Checking, Savings, Credit, Cash) with custom names and currencies.
* **Real-time Aggregation:** Account balances are not static; they are dynamically calculated or updated atomically upon every transaction to ensure the dashboard always reflects the true financial state (`accounts.rs`).

### 3. Transaction Logging & Atomic Transfers


* **Implementation:**
    * **Income & Expenses:** Users can record detailed transactions with dates, descriptions, and amounts. The system automatically categorizes positive values as income and negative as expenses (`transactions.rs`).
    * **Atomic Transfers:** Transferring money between accounts (e.g., Checking to Savings) uses **ACID database transactions** (`pool.begin()`). This ensures that if the deduction fails, the addition is rolled back, preventing data corruption.
    * **History Tracking:** All transactions are timestamped and retrievable via time-range filters.

### 4. Customizable Category System


* **Implementation:**
    * Users can create custom categories (e.g., Food, Rent, Salary) to organize their finances (`categories.rs`).
    * The database schema supports hierarchical data (`parent_id`), allowing for future expansion into sub-categories.
    * Transactions are strictly validated against existing user categories to maintain data consistency.

### 5. Smart Budgeting & Monitoring


* **Implementation:**
    * **Budget Tracking:** Users can set monthly monetary limits for specific categories or globally (`budgets.rs`).
    * **Real-time Calculation:** The backend dynamically calculates `spent` vs `remaining` amounts based on the current month's transaction history.
    * **Visual Alerts:** The TUI dashboard automatically highlights budgets in **Red** if they are exceeded, providing immediate visual feedback on financial health.

### 6. Interactive TUI (Command-Line Interface)


* **Implementation:** Built using **Ratatui** and **Crossterm**.
* **Dashboard View:** A unified "Single Pane of Glass" dashboard displaying Accounts, Recent Transactions, and Budget statuses simultaneously.
* **Keyboard-First Workflow:**
    * **Popup Forms:** Modal windows for data entry (Add Transaction, Transfer, etc.) without losing context.
    * **Navigation:** Intuitive `Tab` / `Arrow Key` navigation and shortcuts (`t` for Transaction, `x` for Transfer) designed for power users.
    * **Asynchronous UI:** The frontend uses `tokio` to perform non-blocking API calls, ensuring the interface remains responsive during data synchronization.




---

# User Guide

> **Pre-configurataion:**

```bash

cd frontend

cargo build

cargo run

```

> This is all you need to test the app.

## Step 1: User Onboarding & Authentication

### 1. Launch Screen

When the application starts, it automatically enters the **LOGIN** screen.

### 2. Switch to Registration Mode

**Action:** Press `Ctrl + r`.

### 3. Register a New User

**Action:**

* **Username:** Enter `demo_user`
* Press `Tab` to switch to the password field
* **Password:** Enter `password123`
* Press `Enter` to submit

**Result:**
A green message appears in the bottom status bar:
**"Success: ... Please Login."**

### 4. Login

**Action:**
Press `Ctrl + r` again to switch back to login mode.
Enter the newly created username and password, then press `Enter`.
The app navigates to the **Dashboard** main screen.


https://github.com/user-attachments/assets/19441314-6df0-4115-b859-2402e02cf2b1


---

## Step 2: Account Management


### 1. Add a Checking Account

**Action:** Press `a` (Add Account).

**Popup Operation:**

* **Name:** Enter `Chase Checking` → press `Down`
* **Currency:** Enter `USD`
* Press `Enter` to submit


### 2. Add a Savings Account

**Action:** Press `a` again.

**Popup Operation:**

`Name: Savings` → `Currency: USD` → `Enter`

If the account name already exits, it will trigger an Error.


https://github.com/user-attachments/assets/ee17f23d-eb26-4ec8-82a4-a87165d578de



## Step 3: Category Management


### 1. Switch Focus

Press **Right Arrow (→)** to move the highlight to the rightmost **"Categories"** panel.


### 2. Add an Expense Category

**Action:** Press `c` again.

**Popup Operation:**

* **Name:** Enter `Car` → `Enter`

If the Category name already exits, it will trigger an Error.

### 3. Verification

The **Categories** table now displays the newly added categories and their IDs.



https://github.com/user-attachments/assets/b60f7211-3089-4807-b730-c21c4179e649




## Step 4: Transaction Logging


### 1. Select an Account

Press **Left Arrow (←)** to return to the **Accounts** panel.
Use **Up/Down** to select **Chase Checking**.



### 2. Scenario A: Log an Income Transaction

**Action:** Press `t` (Transaction).

**Popup Operation** (follow TUI prompts):

1. **Amount:** Enter positive `5000` (represents income) → `Down`
2. **Desc:** Enter `Monthly Salary` → `Down`
3. **Category:** Enter `Salary` (or its ID) → `Enter`

**Result:**
A green income record appears in the middle **Transactions** panel.
`Amount` is shown as **5000.00**, and the account balance increases by 5000.


### 3. Scenario B: Log an Expense Transaction

**Action:** Press `t` again.

**Popup Operation:**

1. **Amount:** Enter negative `-50` (represents expense) → `Down`
2. **Desc:** Enter `Lunch` → `Down`
3. **Category:** Enter `Food` → `Enter`

**Result:**
A red expense record appears in the **Transactions** panel showing **-50.00**.
The account balance automatically decreases from 5000 to **4950**.


https://github.com/user-attachments/assets/28e688de-8ae0-4bd6-bd0a-c42ba53f5329





## Step 5: Budgeting


### 1. Switch Focus

Press **Right Arrow (→)** to move to the **Budgets** panel.



### 2. Set a Budget

**Action:** Press `b` (Budget).

**Popup Operation:**

1. **Amount:** Enter `100` (set a Food budget of 100) → `Down`
2. **Category ID:** Enter the ID of the `Food` category (e.g., `2`) → `Enter`



### 3. Verify Budget Status

* The list now shows **Food 50/100**
* The entry is **green**, because the expense (50) is still below the budget (100)



### 4. Trigger Overspending (Optional Demonstration)

* Go back to Accounts and log another Food expense of **-60**
* The Budgets panel will update to **Food 110/100**, and the text automatically turns **red**, clearly indicating overspending



https://github.com/user-attachments/assets/096f8a39-fb9c-4b2e-b701-b9558ecec3dd


## Step 6: Transfers


### 1. Initiate a Transfer

**Action:** Press `x` (Transfer).

**Popup Operation:**

1. **From ID:** Enter the ID of the Checking account (e.g., `1`) → `Down`
2. **To ID:** Enter the ID of the Savings account (e.g., `2`) → `Down`
3. **Amount:** Enter `1000` (positive) → `Enter`



### 2. Verify the Result

* The **Checking** account balance decreases by **1000**
* The **Savings** account balance increases by **1000**
* The transaction list automatically generates **two records**:
  • one transfer-out
  • one transfer-in


https://github.com/user-attachments/assets/cb2c8b19-d9eb-49ba-ad0c-8fd00f294ea9




## Step 7: Delete & Cleanup


### 1. Delete an Item

* Select the previously added Transfer record or any Budget entry.
* **Action:** Press `d` (Delete).



### 2. Confirmation Popup

* A red dialog titled **"DELETE CONFIRMATION"** appears.
* The system asks you to type the **exact name** of the target (e.g., account name or category name) for secondary confirmation.
* Once the correct name is entered, the item is permanently removed.



https://github.com/user-attachments/assets/88e3962d-07d3-4b32-9221-e5096bdc770f




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
   - TUI Architecture & Event Loop.Designed and implemented the asynchronous terminal user interface using Ratatui and Tokio.Built a robust state management system using the AppState enum to handle seamless transitions between Login, Dashboard, and various Modal Popups.
   - Robust API Client Integration.Developed the ApiClient module using Reqwest to communicate with the backend REST API.
   - Interactive Modal System.Engineered a flexible PopupType system to handle complex multi-step data entry forms (AddTransaction, Transfer) directly within the terminal.
   - Data Visualization & UX.Designed the "Single Pane of Glass" dashboard layout using Ratatui constraints to display Accounts, Transactions, and Budgets simultaneously.Implemented dynamic visual feedback, such as conditional coloring (Red/Green) for expenses vs. income and highlighting over-budget categories to provide immediate financial insights.

### Yiyang Wang

- Backend:
  - Designed and implemented PostgreSQL database schema with 6 core tables supporting user isolation and data integrity
  - Implemented user authentication and authorization using Argon2 password hashing and token-based session management
  - Developed RESTful API endpoints using Axum framework for accounts, transactions, categories, budgets, and transfers
  - Implemented input validation and error handling with balance checks for transfers and user permission verification
  - Created Docker Compose configuration to orchestrate database and backend services for local development

> **Note:** Large lines-of-code contributions were caused by accidentally committing the /target directory earlier in the project. This has now been resolved by adding it to .gitignore.

---

# Concluding Remarks

## Lessons Learned

Through developing this project in Rust, we learned that Rust's strong type system and ownership model help catch bugs at compile time, particularly around memory safety. Working with async Rust (Tokio) and the Axum web framework demonstrated how Rust achieves high performance while maintaining code safety. The comprehensive error handling with `Result` types made our code more robust, and building a TUI frontend with Ratatui showcased Rust's versatility beyond web development.

This project demonstrates how Rust can be used to build a full-stack application with both a RESTful API backend and a terminal-based user interface. The combination of type safety, performance, and developer experience makes Rust an excellent choice for building reliable financial applications.





