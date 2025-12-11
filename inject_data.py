import requests
import random
from datetime import datetime, timedelta

# Configuration
BASE_URL = "http://localhost:3000/api"
USERNAME = "demo1"  # You can modify this to create different users
PASSWORD = "password123"

session = requests.Session()

def print_step(msg):
    print(f"[MESSAGE] {msg}")

def print_success(msg):
    print(f"[DONE] {msg}")

def print_error(msg):
    print(f"[ERROR] {msg}")

# 1. Register or Login
def auth():
    print_step(f"Attempting to register user: {USERNAME}...")
    # Attempt registration
    reg_res = session.post(f"{BASE_URL}/auth/register", json={
        "username": USERNAME,
        "password": PASSWORD
    })
    
    if reg_res.status_code == 200:
        print_success("Registration successful")
    elif reg_res.status_code == 400 and "already exists" in reg_res.text:
        print_step("User already exists, logging in directly...")
    else:
        print_error(f"Registration failed: {reg_res.text}")

    # Login
    print_step("Logging in...")
    login_res = session.post(f"{BASE_URL}/auth/login", json={
        "username": USERNAME,
        "password": PASSWORD
    })

    if login_res.status_code == 200:
        token = login_res.json()["token"]
        session.headers.update({"Authorization": f"Bearer {token}"})
        print_success("Login successful, Token retrieved")
        return True
    else:
        print_error(f"Login failed: {login_res.text}")
        return False

# 2. Create Accounts
def create_accounts():
    accounts = [
        {"name": "Chase Checking", "currency": "USD"},
        {"name": "Amex Gold", "currency": "USD"},
        {"name": "Cash Wallet", "currency": "USD"},
        {"name": "Savings", "currency": "USD"}
    ]
    created_ids = {}
    
    print_step("Creating accounts...")
    for acc in accounts:
        res = session.post(f"{BASE_URL}/accounts", json=acc)
        if res.status_code == 200:
            data = res.json()
            # Handle potential nested 'account' object in response
            acc_id = data.get('id') or data.get('account', {}).get('id')
            print_success(f"Account created: {acc['name']} (ID: {acc_id})")
            created_ids[acc['name']] = acc_id
    return created_ids

# 3. Create Categories
def create_categories():
    categories = ["Food", "Rent", "Salary", "Entertainment", "Transport"]
    created_ids = {}

    print_step("Creating categories...")
    # Fetch existing categories to avoid duplicates
    get_res = session.get(f"{BASE_URL}/categories")
    existing = {c['name']: c['id'] for c in get_res.json()} if get_res.status_code == 200 else {}

    for name in categories:
        if name in existing:
            created_ids[name] = existing[name]
            continue

        res = session.post(f"{BASE_URL}/categories", json={"name": name, "parent_id": None})
        if res.status_code == 200:
            # We will refresh the list later to get IDs reliably
            pass
    
    # Refresh to get IDs
    get_res = session.get(f"{BASE_URL}/categories")
    if get_res.status_code == 200:
        for cat in get_res.json():
            created_ids[cat['name']] = cat['id']
            print_success(f"Category ready: {cat['name']} (ID: {cat['id']})")
    
    return created_ids

# 4. Create Transactions
def create_transactions(acc_map, cat_map):
    print_step("Injecting transaction records...")
    
    # Get IDs
    chase_id = acc_map.get("Chase Checking")
    amex_id = acc_map.get("Amex Gold")
    salary_cat = cat_map.get("Salary")
    food_cat = cat_map.get("Food")
    rent_cat = cat_map.get("Rent")

    if not (chase_id and salary_cat):
        print_error("Failed to get Account or Category IDs, skipping transaction injection")
        return

    today = datetime.now().strftime("%Y-%m-%d")
    yesterday = (datetime.now() - timedelta(days=1)).strftime("%Y-%m-%d")

    txs = [
        # Income
        {
            "account_id": chase_id, "category_id": salary_cat,
            "amount": "5000.00", "type": "income", "date": today, "description": "Monthly Salary"
        },
        # Expense - Rent
        {
            "account_id": chase_id, "category_id": rent_cat,
            "amount": "1200.00", "type": "expense", "date": today, "description": "Rent Payment"
        },
        # Expense - Food (multiple)
        {
            "account_id": amex_id, "category_id": food_cat,
            "amount": "25.50", "type": "expense", "date": today, "description": "Lunch at Subway"
        },
        {
            "account_id": amex_id, "category_id": food_cat,
            "amount": "80.00", "type": "expense", "date": yesterday, "description": "Dinner Date"
        }
    ]

    for tx in txs:
        if not tx['account_id'] or not tx['category_id']: continue
        res = session.post(f"{BASE_URL}/transactions", json=tx)
        if res.status_code == 200:
            print_success(f"Transaction success: {tx['type']} ${tx['amount']} -> {tx['description']}")
        else:
            print_error(f"Transaction failed: {res.text}")

# 5. Create Budgets
def create_budgets(cat_map):
    print_step("Setting budget...")
    food_id = cat_map.get("Food")
    
    if food_id:
        req = {
            "category_id": food_id,
            "amount": "500.00",
            "period": "monthly",
            "start_date": datetime.now().strftime("%Y-%m-%d")
        }
        res = session.post(f"{BASE_URL}/budgets", json=req)
        if res.status_code == 200:
            print_success("Food budget set successfully: $500.00")
        else:
            print_error(f"Budget setting failed: {res.text}")

# Main execution flow
if __name__ == "__main__":
    try:
        # Check if backend is running
        try:
            requests.get(f"{BASE_URL.replace('/api', '')}/health")
        except:
            print_error("Cannot connect to backend. Please ensure 'cargo run' is running in the backend directory!")
            exit(1)

        if auth():
            acc_map = create_accounts()
            cat_map = create_categories()
            if acc_map and cat_map:
                create_transactions(acc_map, cat_map)
                create_budgets(cat_map)
                print("\n [DONE] Data injection complete! Now go verify in TUI")
                print(f"Login Username: {USERNAME}")
                print(f"Login Password: {PASSWORD}")
    except Exception as e:
        print_error(f"Script execution error: {e}")