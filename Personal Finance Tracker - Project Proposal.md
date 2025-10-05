# Personal Finance Tracker - Project Proposal

| Team Member | Student Number |
| --- | --- |
| Shiyao Sun | 1006769793 |
| Yiyang Wang | 1010033278 |



<!-- !toc (minlevel=2 omit="Table of Contents") -->
- [1. Motivation](#1-motivation)
    - [1.1 Target Users](#11-target-users)
    - [1.2 Existing Solutions & Limitations](#12-existing-solutions--limitations)
    - [1.3 How Our Project Solves These Issues](#13-how-our-project-solves-these-issues)
    - [1.4 Why This Project Is Worth Pursuing](#14-why-this-project-is-worth-pursuing)
- [2. Objective](#2-objective)
- [3. Architecture Approach](#3-Architecture-Approach)
- [4. Key Features](#4-key-features)
    - [4.1 User Authentication and Role Management](#41-user-authentication-and-role-management)
    - [4.2 Payment System](#42-payment-system)
    - [4.3 Seat Selection & Booking System](#43-seat-selection--booking-system)
    - [4.4 Order Management System](#44-order-management-system)
    - [4.5 Event Management](#45-event-management)
    - [4.6 QR Code Ticketing & Check-in](#46-qr-code-ticketing--check-in)
    - [4.7 Analytics & Reporting System](#47-analytics--reporting-system)
    - [4.8 Automated Email Confirmations](#48-automated-email-confirmations)
    - [4.9 Waitlist Management](#49-waitlist-management)
    - [4.10 Cloud Storage & File Management](#410-cloud-storage--file-management)
    - [4.11 Advanced Features](#411-advanced-features)
    - [4.12 Search Algorithm](#412-search-algorithm)
- [5. Main Database Schema and Relationships](#5-main-database-schema-and-relationships)
- [6. Tentative Plan](#6-tentative-plan)
    - [6.1 Backend Task](#61-backend-task)
    - [6.2 Frontend Task](#62-frontend-task)
    - [6.3 Collaboration Approach](#63-collaboration-approach)
- [7. Schedule](#7-schedule)
- [8. Conclusion](#8-conclusion)

<!-- toc! -->




## **1. Motivation**

For students and early-career professionals, managing money across checking accounts, credit cards, cash, and e-wallets often leads to scattered records, mistakes, and painful reconciliation. Many apps charge subscriptions, create privacy risks, and rarely allow self-hosting or meaningful automation.

This project will build a Rust-based, terminal-oriented personal finance tracker with a secure HTTPS backend and a relational database. It will support quick transaction capture, consistent categories and tags, split transactions, account transfers, statement reconciliation, budgeting, and flexible reports. The design prioritizes full control of data, reliable local use, and an architecture that is easy to extend.

### **1.1 Target Users**

- Students & early professionals: Those who want a zero cost tool to build budgeting habits and control cash flow.
- Privacy-conscious and technical users : prefer local-first and self-hosting, with exportability, and scriptable imports (CSV/OFX).
- Multi-account OR multi-currency users: manage debit, credit, cash, and e-wallets and need per-currency stats and rollups.

### **1.2 Existing Solutions & Limitations**

- Commercial Financial Management Apps: Comprehensive functionality, excellent mobile experience.But it has high subscription costs, difficulty in fully exporting data, lack of privacy control; inconsistent support for complex allocations and multi-currency; limited automation and custom rules.
- Official Bank Apps: They provide accurate accounting and timely notifications. However, they operate independently within each bank, lacking unified classification and cross-bank aggregation. They also have no zero-based or envelope budgeting features and offer limited reporting and data export capabilities.
- Third-Party Account Aggregation Services: They can automatically pulls bank statements from multiple banks.However, the coverage varies by region, and frequent interface changes lead to instability; requires additional authorization, raising compliance and security concerns; speed and quota restrictions impact user experience.
- Open-source, self-hosted tool: They provide data controllability and offline usability. However, they often have high deployment thresholds, inconsistent documentation and ecosystems; poor command-line and terminal user experiences; and fragmented importer and rule engines, resulting in high subsequent expansion costs.

### **1.3 How Our Project Solves These Issues**

| key Problem | Our Solution |
| --- | --- |
| Personal data is fragmented and insecure | Use self-hosted HTTPS backend and database centralized storage, transmission encryption, controllable access and backup policies |
| Bookkeeping relies on manual operations, which is inefficient | Command line and TUI quick entry, support templates and batch editing, reduce repeated input |
| The category system is confusing and difficult to reuse | Unified classification and multi-label model, supporting search and regular classification, maintaining consistent caliber |
| A single purchase often involves multiple types of expenses, making it difficult to accurately split them. | Transactions use a detailed line structure and can be split into multiple categories by amount or ratio, with automatic verification of the total amount |
| Using multiple accounts in parallel makes it difficult to align balances and transactions | Unified account model, covering current and credit card types, providing inter-account transfers and real-time balance aggregation |
| Reconciliation is complicated and prone to omissions or duplications | Import CSV or OFX bank statements, semi-automatically match amounts and dates, and support reversals and status tracking |
| Lack of budget and reporting makes it difficult to form a closed loop | Provides monthly and quarterly budgets, cash flow and category expenditure reports, net change and trend views, and supports export |
| Lack of scalability, difficulty in integrating new sources | Design a pluggable importer and rule engine, retain the command line batch processing capabilities, and facilitate the subsequent integration of more billing sources |

### **1.4 Why This Project Is Worth Pursuing**

- Real demand: Many users want a self-hosted, privacy-respecting finance tool without subscriptions or vendor lock-in.
- Tangible impact: It helps users gain control of cash flow, reduce manual bookkeeping, and improve the accuracy of monthly budgets and reports.
- Room to grow: The design invites plug-in importers, a rules engine for auto-categorization, email invoice parsing, and a lightweight mobile client in later iterations.
- Differentiation: Local-first storage and optional self-hosting give users full data ownership, better reliability offline, and lower long-term cost.

## **2. Objective**

Build a Rust-based personal finance tracker that exposes a secure HTTPS REST API and an interactive terminal UI. The system stores data in a relational database and lets users manage multiple account types, record income and expenses with categories and tags, create split transactions, transfer between accounts, reconcile statements, and optionally run budgets and financial reports. The solution is lightweight, focusing privacy, self-hosted, and designed for easy extension.

## 3. **Architecture Approach**

### 3.1 Frontend (Ratatui TUI and CLI)

- Interaction: keyboard-first forms, lists, reconciliation flows, and report views
- Core tasks: fast entry, category and tag selection, split input, account transfers, search and filtering
- Networking: talks to the backend through the REST API; small local cache keeps lists and searches responsive
- Config: stores server address and token in a local config file with protected secrets
- Resilience: allows offline entry and retries sync when a connection is available

### 3.2 Backend (Axum HTTPS service)

- Expose a secure HTTPS REST API with unified routing and middleware
- Routing & middleware: Axum on top of Tower layers for auth, logging, timeouts, compression, and rate-limits.
- Domain modules: Accounts, Categories, Transactions, Reconciliation, Budgets, Reports, Imports
- Layers: a service layer enforces business rules and transaction boundaries; a repository layer uses SQLx for data access.
- Auth: token-based authentication (JWT or similar) with simple roles such as read-only and editor.
- API ergonomics: consistent pagination, filtering, sorting; typed errors the TUI can map to user actions.
- Structured logs and metrics, health and readiness probes, generated OpenAPI spec
- Background work: async jobs for CSV/OFX import, auto-categorization hints, reconciliation suggestions, and report pre-computation; idempotent with retries.

### 3.3 Database (PostgreSQL with SQLx)

- Create a transaction and its splits in one database transaction; validate split totals on both service and database sides.
- Money safety: store monetary amounts as numeric to avoid floating-point errors; avoid the money type for primary storage due to locale and precision caveats.
- Transfers write two transactions and a pairing link; any failure rolls back the whole unit
- Indexing: composite indexes for common filters like user with date range and account with status; index all foreign keys.
- Index all foreign keys; use materialized views or periodic pre-computation for heavy reports
- First-class export and backup so users keep full ownership of their data

### 3.4 Deployment (Docker and Docker Compose, DigitalOcean)

- One-command dev: compose file brings up Postgres and the API with seed data; the TUI targets the local server.
- Production: reverse proxy with TLS, environment-based secrets or a secret manager, least-privilege tokens.
- Run database migrations on startup; scheduled backups and log rotation
- Pinned versions and reproducible builds so local and cloud environments match
- Monitoring and alerts for API availability, DB connections, and background job queues

## **4. Key Features**

### **4.1 Account Management**

- User registration and authentication
    - Implement secure password hashing using bcrypt.
    - Store user credentials and sessions securely in the database.
- Multiple accounts per user
    - Support checking, savings, and credit accounts.
    - Each account maintains its balance, currency type, and transaction history.
- Account summary and queries
    - returns current balance and metadata.
    - Real-time aggregation of balances across all accounts.
- Auditing and Traceability
    - All write operations are logged in the audit log for easy traceability and compliance checks.

### 4.2 Transaction Recording and Categories

- Transaction model
    - Each transaction has an amount, category, date, and description.
    - Supports income, expense, and transfer types.
- Category management
    - Predefined categories (Bonus, Salary, Stock, Food, Rent, Travel, Utilities).
    - Users can create custom categories and tags.
- REST API Endpoints
    - add a new transaction.
    - list transactions by date range, category, or account.
- Query and Export
    - Filter by time window, account, category, and keyword; export to CSV is supported.

### 4.3 Reconciliation and Imports

- Reconciliation Process
    - Import CSV or OFX bank statements. The system provides matching suggestions based on amount and date, supporting confirmation, rejection, and difference marking.
- Import Jobs
    - Import tasks are processed asynchronously in the background, with retry options for failed tasks. Unmatched items are placed in a to-do list for user reconfirmation.

### 4.4 Budgeting (Enhancement)

- Budget tracking
    - Users can set monthly or category-level budgets.
    - The system calculates progress and remaining allowance.
- Reports
    - Views include cash flow, expenditure by category, net asset changes, and income-to-expense ratio; detailed reports can be exported.
- TUI Display
    - Budget progress and key indicators are intuitively displayed on the terminal dashboard.

### 4.5 Data Storage and Security

- Database: PostgreSQL (accessed via SQLx).
    - Ensures ACID compliance and efficient relational querying.
- Secure communications
    - All backend APIs served over HTTPS.
    - Input validation and parameterized queries to prevent SQL injection.

### 4.6 Command-Line User Interface (CLI)

- Built with Ratatui, providing a text-based dashboard.
- Layout
    - Left: account list and balances.
    - Center: recent transactions.
    - Right: budget summaries.
- Interactive navigation using arrow keys or commands.
- Communicates with backend REST API through HTTPS requests.

### 4.7 Deployment (If Time Allows)

- The system can run entirely locally, or optionally deploy the HTTPS backend to a cloud platform (e.g., DigitalOcean) for multi-device access.
- This architecture supports both local-first and self-hosted modes.

## **5 Database Schema and Relationships**

**USERS**

| ID | email | passwordHash | createdAt | updatedAt |
| --- | --- | --- | --- | --- |

**APIToken**

| ID | userID | tokenHash | expiresAt | revokedAt | createdAt |
| --- | --- | --- | --- | --- | --- |

**ACCOUNTS**

| ID | userID | name | type | currency | openingBalance | archived | createdAt |
| --- | --- | --- | --- | --- | --- | --- | --- |

**CATEGORIES**

| ID | userID | name | parentID | kind | createdAt |
| --- | --- | --- | --- | --- | --- |

**TRANSACTIONS**

| ID | userID | accountID | postedAt | amountTotal | payee | memo | status | externalRef | createdAt | updatedAt |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |

**TRANSACTION_SPLITS**

| ID | transactionID | categoryID | amount | memo |
| --- | --- | --- | --- | --- |

**TRANSFER_LINKS**

| ID | fromTransactionID | toTransactionID | exchangeRate | note |
| --- | --- | --- | --- | --- |

**STATEMENTS**

| ID | accountID | periodStart | periodEnd | openingBalance | closingBalance | sourceFile | createdAt |
| --- | --- | --- | --- | --- | --- | --- | --- |

**RECONCILIATION_MATCHES**

| ID | statementID | transactionID | matchScore | matchedAmount | matchedDate | matchedDescHash | status | createdAt |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |

**BUDGETS**

| ID | userID | name | periodStart | periodEnd | currency | createdAt |
| --- | --- | --- | --- | --- | --- | --- |

**BUDGET_ITEMS**

| ID | budgetID | categoryID | amount |
| --- | --- | --- | --- |

**EXCHANGE_RATES**

| ID | baseCurrency | quoteCurrency | rate | asOf |
| --- | --- | --- | --- | --- |

**AUDIT_LOGS**

| ID | userID | entityType | entityID | action | beforeJSON | afterJSON | createdAt |
| --- | --- | --- | --- | --- | --- | --- | --- |

**IMPORT_JOBS**

| ID | userID | accountID | source | status | startedAt | finishedAt |
| --- | --- | --- | --- | --- | --- | --- |

**IMPORT_ROWS**

| ID | jobID | rawJSON | parsedAmount | parsedDate | parsedDesc | transactionID | status | error |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |

**Primary and Foreign Keys**

| Table | Primary Key (PK) | Foreign Keys (FK) |
| --- | --- | --- |
| Users | ID | – |
| APIToken | ID | userID → Users(ID) |
| Accounts | ID | userID → Users(ID) |
| Categories | ID | userID → Users(ID), parentID → Categories(ID) |
| Transactions | ID | userID → Users(ID), accountID → Accounts(ID) |
| Transaction_Splits | ID | transactionID → Transactions(ID), categoryID → Categories(ID) |
| Transfer_Links | ID | fromTransactionID → Transactions(ID), toTransactionID → Transactions(ID) |
| Statements | ID | accountID → Accounts(ID) |
| Reconciliation_Matches | ID | statementID → Statements(ID), transactionID → Transactions(ID) |
| Budgets | ID | userID → Users(ID) |
| Budget_Items | ID | budgetID → Budgets(ID), categoryID → Categories(ID) |
| Exchange_Rates | ID | – |
| Audit_Logs | ID | userID → Users(ID) |
| Import_Jobs | ID | userID → Users(ID), accountID → Accounts(ID) |
| Import_Rows | ID | jobID → Import_Jobs(ID), transactionID → Transactions(ID) |

## **6. Tentative Plan**

### **6.1 Team Workflow**

The team will begin by jointly designing the overall system architecture, including the database schema, API specifications, and command-line interface layout.

Once the design is finalized, development will proceed in parallel across backend and frontend components with continuous collaboration via GitHub commits and pull requests.

---

### **6.2 Division of Responsibilities**

**Yiyang Wang — Backend and Database Development**

- Design and create the PostgreSQL database schema.
- Implement RESTful API routes in Axum for accounts, transactions, and budgets.
- Integrate SQLx for database interactions and schema migrations.
- Handle authentication, input validation, and backend testing.

**Shiyao Sun — Frontend, Integration, and Deployment**

- Implement the Ratatui-based command-line interface to interact with backend APIs.
- Test and debug HTTPS and JSON communication with the backend.
- Prepare Docker configurations for reproducible builds and testing.
- Manage optional deployment to a cloud environment such as DigitalOcean or Render.

---

### **6.3 Development Process**

1. **System Design (Joint Task)**
    
    Define database schema, API endpoints, and CLI layout.
    
    Document the structure and data flow between modules.
    
2. **Parallel Implementation (Github Version Control)**
    
    Backend & Database (Yiyang Wang): Implement Axum REST API, SQLx models, and migrations.
    
    Frontend & Integration (Shiyao Sun): Develop the Ratatui interface and connect it to the API.
    
3. **Integration and Testing**
    
    Combine backend and frontend, perform end-to-end testing for transaction creation, querying, and reporting.
    
    Validate correctness of database operations and CLI display.
    
4. **Containerization and Deployment**
    
    Build Docker images for reproducibility.
    
    May deploy the HTTPS backend to a lightweight cloud environment.
    
5. **Documentation and Submission**
    
    Write usage instructions, API reference, and build guide in the final README.
    
    Ensure reproducibility for grading without external assistance.
    
