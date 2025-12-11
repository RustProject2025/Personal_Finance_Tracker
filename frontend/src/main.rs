mod models;
mod api;

use std::{io, time::Duration};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, SetSize},
};
use ratatui::{prelude::*, widgets::*};
use api::ApiClient;
use models::*;


enum AppState {
    Login,
    Dashboard,
    InputPopup(PopupType),
}

#[derive(PartialEq)]
enum Focus {
    Accounts,
    Budgets,
    Categories,
}


enum PopupType {
    AddAccount { step: usize, name: String, currency: String },
    AddTransaction { step: usize, amount: String, desc: String, category_input: String }, 
    Transfer { step: usize, from_id: String, to_id: String, amount: String },
    AddCategory { name: String },
    AddBudget { step: usize, amount: String, category_id: String },
    
 
    DeleteConfirm { 
        type_label: String, 
        target_id: i32, 
        verify_name: String, 
        input_name: String 
    },
}

#[derive(PartialEq)]
enum InputMode {
    Username,
    Password,
}

struct App {
    state: AppState,
    api: ApiClient,
    
    // Auth
    input_username: String,
    input_password: String,
    input_mode: InputMode,
    is_register_mode: bool,
    message: Option<(String, Color)>, 

    // Data
    accounts: Vec<AccountResponse>,
    transactions: Vec<TransactionResponse>,
    budgets: Vec<BudgetResponse>,
    categories: Vec<CategoryResponse>, 

    // UI Navigation State
    focus: Focus, 
    account_list_state: ListState, 
    budget_list_state: ListState,
    category_list_state: ListState,
}

impl App {
    fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0)); 

        Self {
            state: AppState::Login,
            api: ApiClient::new(),
            input_username: String::new(),
            input_password: String::new(),
            input_mode: InputMode::Username,
            is_register_mode: false,
            message: None,
            
            accounts: vec![],
            transactions: vec![],
            budgets: vec![],
            categories: vec![],
            
            // 初始化焦点和列表状态
            focus: Focus::Accounts, 
            account_list_state: list_state.clone(),
            budget_list_state: list_state.clone(),
            category_list_state: list_state,
        }
    }

    // === Helpers ===
    fn get_selected_account(&self) -> Option<&AccountResponse> {
        self.account_list_state.selected().and_then(|i| self.accounts.get(i))
    }
    
    fn get_selected_budget(&self) -> Option<&BudgetResponse> {
        self.budget_list_state.selected().and_then(|i| self.budgets.get(i))
    }

    fn get_selected_category(&self) -> Option<&CategoryResponse> {
        self.category_list_state.selected().and_then(|i| self.categories.get(i))
    }

    fn resolve_category(&self, input: &str) -> Option<(i32, String)> {
        let input = input.trim();
        if input.is_empty() { return None; }
        if let Ok(id) = input.parse::<i32>() {
            if let Some(cat) = self.categories.iter().find(|c| c.id == id) {
                return Some((cat.id, cat.name.clone()));
            }
        }
        if let Some(cat) = self.categories.iter().find(|c| c.name.eq_ignore_ascii_case(input)) {
            return Some((cat.id, cat.name.clone()));
        }
        None 
    }

    fn next_item(&mut self) {
        match self.focus {
            Focus::Accounts => {
                let i = next_index(self.account_list_state.selected(), self.accounts.len());
                self.account_list_state.select(Some(i));
            }
            Focus::Budgets => {
                let i = next_index(self.budget_list_state.selected(), self.budgets.len());
                self.budget_list_state.select(Some(i));
            }
            Focus::Categories => {
                let i = next_index(self.category_list_state.selected(), self.categories.len());
                self.category_list_state.select(Some(i));
            }
        }
    }

    fn previous_item(&mut self) {
        match self.focus {
            Focus::Accounts => {
                let i = prev_index(self.account_list_state.selected(), self.accounts.len());
                self.account_list_state.select(Some(i));
            }
            Focus::Budgets => {
                let i = prev_index(self.budget_list_state.selected(), self.budgets.len());
                self.budget_list_state.select(Some(i));
            }
            Focus::Categories => {
                let i = prev_index(self.category_list_state.selected(), self.categories.len());
                self.category_list_state.select(Some(i));
            }
        }
    }

    fn toggle_auth_mode(&mut self) {
        self.is_register_mode = !self.is_register_mode;
        self.message = None;
    }

    async fn try_register(&mut self) {
        let req = LoginRequest { username: self.input_username.clone(), password: self.input_password.clone() };
        match self.api.register(req).await {
            Ok(msg) => { self.is_register_mode = false; self.message = Some((format!("Success: {}. Please Login.", msg), Color::Green)); }
            Err(e) => self.message = Some((format!("Register Error: {}", e), Color::Red)),
        }
    }

    async fn try_login(&mut self) {
        let req = LoginRequest { username: self.input_username.clone(), password: self.input_password.clone() };
        match self.api.login(req).await {
            Ok(_) => {
                self.state = AppState::Dashboard;
                self.message = None;
                self.refresh_all_data().await;
            }
            Err(e) => self.message = Some((e.to_string(), Color::Red)),
        }
    }

    async fn refresh_all_data(&mut self) {
        if let Ok(data) = self.api.get_accounts().await { self.accounts = data; }
        if let Ok(data) = self.api.get_budgets().await { self.budgets = data; }
        if let Ok(data) = self.api.get_categories().await { self.categories = data; }
        self.refresh_transactions().await;
    }

    async fn refresh_transactions(&mut self) {
        let selected_id = self.get_selected_account().map(|a| a.id);
        match self.api.get_transactions(selected_id).await {
            Ok(data) => self.transactions = data,
            Err(_) => self.transactions = vec![],
        }
    }

   
    fn init_delete(&mut self) {
        match self.focus {
            Focus::Accounts => {
                if let Some(acc) = self.get_selected_account() {
                    self.state = AppState::InputPopup(PopupType::DeleteConfirm {
                        type_label: "Account".to_string(),
                        target_id: acc.id,
                        verify_name: acc.name.clone(),
                        input_name: String::new(),
                    });
                }
            },
            Focus::Budgets => {
                if let Some(b) = self.get_selected_budget() {
                 
                    let name = b.category_name.clone().unwrap_or("Global".to_string());
                    self.state = AppState::InputPopup(PopupType::DeleteConfirm {
                        type_label: "Budget".to_string(),
                        target_id: b.id,
                        verify_name: name,
                        input_name: String::new(),
                    });
                }
            },
            Focus::Categories => {
                if let Some(c) = self.get_selected_category() {
                    self.state = AppState::InputPopup(PopupType::DeleteConfirm {
                        type_label: "Category".to_string(),
                        target_id: c.id,
                        verify_name: c.name.clone(),
                        input_name: String::new(),
                    });
                }
            }
        }
    }


    async fn submit_popup(&mut self) {
        if let AppState::InputPopup(ref popup) = self.state {
            let result = match popup {
                PopupType::AddAccount { name, currency, .. } => {
                    let name_trim = name.trim();
                    if name_trim.is_empty() {
                        Err(anyhow::anyhow!("Account name cannot be empty!"))
                    } else if name_trim.len() > 50 {
                        Err(anyhow::anyhow!("Account name must be 50 characters or less!"))
                    } else if self.accounts.iter().any(|a| a.name.eq_ignore_ascii_case(name_trim)) {
                        Err(anyhow::anyhow!("Account '{}' already exists!", name_trim))
                    } else {
                        self.api.create_account(CreateAccountRequest { name: name_trim.to_string(), currency: Some(currency.clone()) }).await
                    }
                },
                PopupType::AddTransaction { amount, desc, category_input, .. } => {
                    let acc_id = self.get_selected_account().map(|a| a.id);
                    if acc_id.is_none() {
                        Err(anyhow::anyhow!("Select an account first!"))
                    } else {
                        let amount_trim = amount.trim();
                        if amount_trim.is_empty() {
                            Err(anyhow::anyhow!("Amount cannot be empty!"))
                        } else {
                            let amount_val = match amount_trim.parse::<f64>() {
                                Ok(v) => v,
                                Err(_) => {
                                    self.message = Some(("Invalid amount format! Use numbers only.".to_string(), Color::Red));
                                    return;
                                }
                            };
                            if amount_val == 0.0 {
                                self.message = Some(("Amount cannot be zero!".to_string(), Color::Red));
                                return;
                            } else {
                                let input_trim = category_input.trim();
                                let mut final_cat_id = None;
                                if !input_trim.is_empty() {
                                    if let Some((id, _)) = self.resolve_category(input_trim) {
                                        final_cat_id = Some(id);
                                    } else {
                                        self.message = Some((format!("Invalid Category: '{}'", input_trim), Color::Red));
                                        return;
                                    }
                                }
                                self.api.create_transaction(CreateTransactionRequest {
                                    account_id: acc_id,
                                    account_name: None, category_id: final_cat_id,
                                    amount: amount_trim.to_string(),
                                    r#type: if amount_val >= 0.0 { "income".to_string() } else { "expense".to_string() },
                                    date: chrono::Local::now().format("%Y-%m-%d").to_string(),
                                    description: Some(desc.clone()),
                                }).await
                            }
                        }
                    }
                },
                PopupType::Transfer { from_id, to_id, amount, .. } => {
                    let from_id_trim = from_id.trim();
                    let to_id_trim = to_id.trim();
                    let amount_trim = amount.trim();
                    
                    if from_id_trim.is_empty() {
                        Err(anyhow::anyhow!("From account ID cannot be empty!"))
                    } else if to_id_trim.is_empty() {
                        Err(anyhow::anyhow!("To account ID cannot be empty!"))
                    } else if amount_trim.is_empty() {
                        Err(anyhow::anyhow!("Amount cannot be empty!"))
                    } else {
                        let f = match from_id_trim.parse::<i32>() {
                            Ok(id) => id,
                            Err(_) => {
                                self.message = Some(("Invalid from account ID! Use numbers only.".to_string(), Color::Red));
                                return;
                            }
                        };
                        let t = match to_id_trim.parse::<i32>() {
                            Ok(id) => id,
                            Err(_) => {
                                self.message = Some(("Invalid to account ID! Use numbers only.".to_string(), Color::Red));
                                return;
                            }
                        };
                        
                        if f == t {
                            Err(anyhow::anyhow!("Cannot transfer to the same account!"))
                        } else if !self.accounts.iter().any(|a| a.id == f) {
                            Err(anyhow::anyhow!("From account #{} not found!", f))
                        } else if !self.accounts.iter().any(|a| a.id == t) {
                            Err(anyhow::anyhow!("To account #{} not found!", t))
                        } else {
                            let amount_val = match amount_trim.parse::<f64>() {
                                Ok(v) => v,
                                Err(_) => {
                                    self.message = Some(("Invalid amount format! Use numbers only.".to_string(), Color::Red));
                                    return;
                                }
                            };
                            if amount_val <= 0.0 {
                                self.message = Some(("Transfer amount must be positive!".to_string(), Color::Red));
                                return;
                            } else {
                                self.api.transfer(TransferRequest { from_account_id: f, to_account_id: t, amount: amount_trim.to_string(), date: None, description: Some("TUI Transfer".to_string()) }).await
                            }
                        }
                    }
                },
                PopupType::AddCategory { name, .. } => {
                    let name_trim = name.trim();
                    if name_trim.is_empty() {
                        Err(anyhow::anyhow!("Category name cannot be empty!"))
                    } else if name_trim.len() > 50 {
                        Err(anyhow::anyhow!("Category name must be 50 characters or less!"))
                    } else if self.categories.iter().any(|c| c.name.eq_ignore_ascii_case(name_trim)) {
                        Err(anyhow::anyhow!("Category '{}' already exists!", name_trim))
                    } else {
                        self.api.create_category(CreateCategoryRequest { name: name_trim.to_string(), parent_id: None }).await
                    }
                },
                PopupType::AddBudget { amount, category_id, .. } => {
                    let amount_trim = amount.trim();
                    if amount_trim.is_empty() {
                        Err(anyhow::anyhow!("Budget amount cannot be empty!"))
                    } else {
                        let amount_val = match amount_trim.parse::<f64>() {
                            Ok(v) => v,
                            Err(_) => {
                                self.message = Some(("Invalid amount format! Use numbers only.".to_string(), Color::Red));
                                return;
                            }
                        };
                        if amount_val <= 0.0 {
                            self.message = Some(("Budget amount must be positive!".to_string(), Color::Red));
                            return;
                        } else {
                            let cat_id = if category_id.trim().is_empty() { 
                                None 
                            } else { 
                                let parsed_id = match category_id.trim().parse::<i32>() {
                                    Ok(id) => id,
                                    Err(_) => {
                                        self.message = Some(("Invalid category ID! Use numbers only.".to_string(), Color::Red));
                                        return;
                                    }
                                };
                                if !self.categories.iter().any(|c| c.id == parsed_id) {
                                    self.message = Some((format!("Category #{} not found!", parsed_id), Color::Red));
                                    return;
                                } else {
                                    Some(parsed_id)
                                }
                            };
                            self.api.create_budget(CreateBudgetRequest { category_id: cat_id, amount: amount_trim.to_string(), period: Some("monthly".to_string()), start_date: None }).await
                        }
                    }
                },
                
              
                PopupType::DeleteConfirm { type_label, target_id, verify_name, input_name } => {
                    if input_name != verify_name {
                        Err(anyhow::anyhow!("Name mismatch! Cancelled."))
                    } else {
                        match type_label.as_str() {
                            "Account" => self.api.delete_account(*target_id).await,
                            "Category" => self.api.delete_category(*target_id).await,
                            "Budget" => self.api.delete_budget(*target_id).await,
                            _ => Ok(()),
                        }
                    }
                }
            };

            match result {
                Ok(_) => {
                    self.message = Some(("Action Successful!".to_string(), Color::Green));
                    self.refresh_all_data().await;
                    self.state = AppState::Dashboard;
                },
                Err(e) => self.message = Some((format!("Error: {}", e), Color::Red)),
            }
        }
    }
}


fn next_index(curr: Option<usize>, len: usize) -> usize {
    if len == 0 { return 0; }
    match curr {
        Some(i) => if i >= len - 1 { 0 } else { i + 1 },
        None => 0,
    }
}
fn prev_index(curr: Option<usize>, len: usize) -> usize {
    if len == 0 { return 0; }
    match curr {
        Some(i) => if i == 0 { len - 1 } else { i - 1 },
        None => 0,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    let _ = execute!(stdout, EnterAlternateScreen, EnableMouseCapture, SetSize(140, 70));
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match app.state {
                    AppState::Login => {
                        
                        match key.code {
                            KeyCode::Tab => app.input_mode = match app.input_mode { InputMode::Username => InputMode::Password, InputMode::Password => InputMode::Username },
                            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => app.toggle_auth_mode(),
                            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                                match app.input_mode { InputMode::Username => app.input_username.push(c), InputMode::Password => app.input_password.push(c) }
                            },
                            KeyCode::Backspace => match app.input_mode { InputMode::Username => { app.input_username.pop(); }, InputMode::Password => { app.input_password.pop(); } },
                            KeyCode::Enter => { if app.is_register_mode { app.try_register().await; } else { app.try_login().await; } },
                            KeyCode::Esc => break, 
                            _ => {}
                        }
                    }
                    
                    AppState::Dashboard => {
                        match key.code {
                            KeyCode::Esc => break,
                            KeyCode::Char('r') => app.refresh_all_data().await,
                            
                            
                            KeyCode::Left => {
                                app.focus = match app.focus {
                                    Focus::Categories => Focus::Budgets,
                                    Focus::Budgets => Focus::Accounts,
                                    Focus::Accounts => Focus::Accounts,
                                };
                            },
                            KeyCode::Right => {
                                app.focus = match app.focus {
                                    Focus::Accounts => Focus::Budgets,
                                    Focus::Budgets => Focus::Categories,
                                    Focus::Categories => Focus::Categories,
                                };
                            },
                           
                            KeyCode::Down => { 
                                app.next_item(); 
                                if app.focus == Focus::Accounts { app.refresh_transactions().await; }
                            },
                            KeyCode::Up => { 
                                app.previous_item(); 
                                if app.focus == Focus::Accounts { app.refresh_transactions().await; }
                            },

                           
                            KeyCode::Char('a') => app.state = AppState::InputPopup(PopupType::AddAccount { step: 0, name: String::new(), currency: "USD".to_string() }),
                            KeyCode::Char('t') => {
                                if app.accounts.is_empty() { app.message = Some(("Create an account first!".to_string(), Color::Red)); }
                                else { app.state = AppState::InputPopup(PopupType::AddTransaction { step: 0, amount: String::new(), desc: String::new(), category_input: String::new() }); }
                            },
                            KeyCode::Char('x') => app.state = AppState::InputPopup(PopupType::Transfer { step: 0, from_id: String::new(), to_id: String::new(), amount: String::new() }),
                            KeyCode::Char('c') => app.state = AppState::InputPopup(PopupType::AddCategory { name: String::new() }),
                            KeyCode::Char('b') => app.state = AppState::InputPopup(PopupType::AddBudget { step: 0, amount: String::new(), category_id: String::new() }),
                            
                            
                            KeyCode::Char('d') => app.init_delete(),
                            
                            _ => {}
                        }
                    }

                    AppState::InputPopup(ref mut popup) => {
                        match key.code {
                            KeyCode::Esc => app.state = AppState::Dashboard,
                            KeyCode::Enter => app.submit_popup().await,
                            
                            KeyCode::Down => {
                                match popup {
                                    PopupType::AddAccount { step, .. } => *step = (*step + 1) % 2,
                                    PopupType::AddTransaction { step, .. } => *step = (*step + 1) % 3,
                                    PopupType::Transfer { step, .. } => *step = (*step + 1) % 3,
                                    PopupType::AddBudget { step, .. } => *step = (*step + 1) % 2,
                                    _ => {}
                                }
                            },
                            KeyCode::Up => {
                                match popup {
                                    PopupType::AddAccount { step, .. } => *step = if *step == 0 { 1 } else { *step - 1 },
                                    PopupType::AddTransaction { step, .. } => *step = if *step == 0 { 2 } else { *step - 1 },
                                    PopupType::Transfer { step, .. } => *step = if *step == 0 { 2 } else { *step - 1 },
                                    PopupType::AddBudget { step, .. } => *step = if *step == 0 { 1 } else { *step - 1 },
                                    _ => {}
                                }
                            },
                          
                            KeyCode::Char(c) => {
                                match popup {
                                    PopupType::AddAccount { step, name, currency } => if *step == 0 { name.push(c) } else { currency.push(c) },
                                    PopupType::AddTransaction { step, amount, desc, category_input } => { match step { 0 => amount.push(c), 1 => desc.push(c), 2 => category_input.push(c), _ => {} } },
                                    PopupType::Transfer { step, from_id, to_id, amount } => { match step { 0 => from_id.push(c), 1 => to_id.push(c), 2 => amount.push(c), _ => {} } },
                                    PopupType::AddCategory { name, .. } => name.push(c),
                                    PopupType::AddBudget { step, amount, category_id } => if *step == 0 { amount.push(c) } else { category_id.push(c) },
                                    PopupType::DeleteConfirm { input_name, .. } => input_name.push(c),
                                }
                            },
                            KeyCode::Backspace => {
                                match popup {
                                    PopupType::AddAccount { step, name, currency } => if *step == 0 { name.pop(); } else { currency.pop(); },
                                    PopupType::AddTransaction { step, amount, desc, category_input } => { match step { 0 => {amount.pop();}, 1 => {desc.pop();}, 2 => {category_input.pop();}, _ => {} } },
                                    PopupType::Transfer { step, from_id, to_id, amount } => { match step { 0 => {from_id.pop();}, 1 => {to_id.pop();}, 2 => {amount.pop();}, _ => {} } },
                                    PopupType::AddCategory { name, .. } => { name.pop(); },
                                    PopupType::AddBudget { step, amount, category_id } => if *step == 0 { amount.pop(); } else { category_id.pop(); },
                                    PopupType::DeleteConfirm { input_name, .. } => { input_name.pop(); },
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let size = f.size();
    match app.state {
        AppState::Login => render_login(f, app, size),
        AppState::Dashboard | AppState::InputPopup(_) => {
            render_dashboard(f, app, size);
            if let AppState::InputPopup(ref popup) = app.state {
                render_popup(f, popup, size, app);
            }
        }
    }
}


fn render_login(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default().direction(Direction::Vertical).margin(2).constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Length(3), Constraint::Length(3), Constraint::Min(1)]).split(area);
    let (title_text, title_color) = if app.is_register_mode { ("REGISTER NEW ACCOUNT", Color::Magenta) } else { ("PERSONAL FINANCE TRACKER - LOGIN", Color::Cyan) };
    f.render_widget(Paragraph::new(title_text).style(Style::default().fg(title_color).add_modifier(Modifier::BOLD)).alignment(Alignment::Center), chunks[0]);
    f.render_widget(Paragraph::new(app.input_username.as_str()).block(Block::default().borders(Borders::ALL).title("Username").border_style(if app.input_mode == InputMode::Username { Style::default().fg(Color::Yellow) } else { Style::default() })), chunks[1]);
    f.render_widget(Paragraph::new(app.input_password.chars().map(|_| '*').collect::<String>()).block(Block::default().borders(Borders::ALL).title("Password").border_style(if app.input_mode == InputMode::Password { Style::default().fg(Color::Yellow) } else { Style::default() })), chunks[2]);
    let mode_txt = if app.is_register_mode { "Switch to Login (Ctrl+r)" } else { "Use (Tab) to switch line.Use (Enter) to submit. Switch to Register (Ctrl+r)" };
    f.render_widget(Paragraph::new(mode_txt).alignment(Alignment::Center), chunks[3]);
    if let Some((msg, color)) = &app.message { f.render_widget(Paragraph::new(msg.as_str()).style(Style::default().fg(*color)).alignment(Alignment::Center), chunks[4]); }
}

fn render_dashboard(f: &mut Frame, app: &App, area: Rect) {
    let vertical_chunks = Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(3), Constraint::Min(0)]).split(area);
    let help_text = "Nav: ←/→ Switch Panel | ↑/↓ Select | 'd' Delete | 't' Tx | 'a' Acc | 'x' Transfer | 'c' Cat | 'b' Budget";
    f.render_widget(Paragraph::new(help_text).style(Style::default().fg(Color::White).bg(Color::Blue)).alignment(Alignment::Center).block(Block::default().borders(Borders::ALL)), vertical_chunks[0]);

    let main_chunks = Layout::default().direction(Direction::Horizontal).constraints([Constraint::Percentage(25), Constraint::Percentage(45), Constraint::Percentage(30)]).split(vertical_chunks[1]);

   
    let border_style = |active: bool| if active { Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) } else { Style::default() };

    
    let account_items: Vec<ListItem> = app.accounts.iter().map(|acc| {
        let content = format!("[#{}] {}", acc.id, acc.name);
        ListItem::new(content).style(Style::default().fg(Color::Cyan))
    }).collect();
    let accounts_list = List::new(account_items)
        .block(Block::default().borders(Borders::ALL).title("Accounts").border_style(border_style(app.focus == Focus::Accounts)))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)).highlight_symbol(">> ");
    f.render_stateful_widget(accounts_list, main_chunks[0], &mut app.account_list_state.clone());

   
    let tx_title = if let Some(acc) = app.get_selected_account() { 
        format!("Transactions (Account #{} | Balance: {} {})", acc.id, acc.balance, acc.currency) 
    } else { 
        "Transactions (All)".to_string() 
    };
    let header_cells = ["Date", "Desc", "Cat", "Amt"].iter().map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);
    let rows = app.transactions.iter().map(|t| {
        let amount_style = if t.r#type == "expense" { Style::default().fg(Color::Red) } else { Style::default().fg(Color::Green) };
        let cells = vec![Cell::from(t.date.clone()), Cell::from(t.description.clone().unwrap_or_default()), Cell::from(t.category_name.clone().unwrap_or_default()), Cell::from(t.amount.clone()).style(amount_style)];
        Row::new(cells).height(1)
    });
    f.render_widget(Table::new(rows, [
        Constraint::Min(10),
        Constraint::Percentage(40),
        Constraint::Percentage(20),
        Constraint::Min(10),
    ]).header(header).block(Block::default().borders(Borders::ALL).title(tx_title)), main_chunks[1]);

   
    let right_chunks = Layout::default().direction(Direction::Vertical).constraints([Constraint::Percentage(50), Constraint::Percentage(50)]).split(main_chunks[2]);
    
    let budget_items: Vec<ListItem> = app.budgets.iter().map(|b| {
        let name = b.category_name.as_deref().unwrap_or("Global");
        let content = format!("{} {}/{}", name, b.spent, b.amount);
        let color = if b.is_over_budget { Color::Red } else { Color::Green };
        ListItem::new(content).style(Style::default().fg(color))
    }).collect();
    let budget_list = List::new(budget_items)
        .block(Block::default().borders(Borders::ALL).title("Budgets").border_style(border_style(app.focus == Focus::Budgets)))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)).highlight_symbol(">> ");
    f.render_stateful_widget(budget_list, right_chunks[0], &mut app.budget_list_state.clone());

    
    let cat_items: Vec<ListItem> = app.categories.iter().map(|c| ListItem::new(format!("{}: {}", c.id, c.name))).collect();
    let cat_list = List::new(cat_items)
        .block(Block::default().borders(Borders::ALL).title("Categories").border_style(border_style(app.focus == Focus::Categories)))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)).highlight_symbol(">> ");
    f.render_stateful_widget(cat_list, right_chunks[1], &mut app.category_list_state.clone());

    if let Some((msg, color)) = &app.message {
        let msg_area = Rect { x: area.x, y: area.height.saturating_sub(1), width: area.width, height: 1 };
        f.render_widget(Paragraph::new(msg.as_str()).style(Style::default().bg(*color).fg(Color::Black)), msg_area);
    }
}

fn render_popup(f: &mut Frame, popup: &PopupType, area: Rect, app: &App) {
    let width_percent = (area.width * 60 / 100).min(80).max(50);
    let height_percent = (area.height * 30 / 100).min(30).max(15);
    let area = centered_rect_percent(width_percent, height_percent, area);
    f.render_widget(Clear, area);
    let block = Block::default().borders(Borders::ALL).style(Style::default().bg(Color::DarkGray));
    let st = |s: usize, target: usize| if s == target { Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) } else { Style::default() };

    let has_error = app.message.is_some();
    let constraints = match popup {
        PopupType::AddAccount { .. } => vec![Constraint::Min(3), Constraint::Min(3), if has_error { Constraint::Length(3) } else { Constraint::Length(0) }],
        PopupType::AddTransaction { .. } => vec![Constraint::Min(3), Constraint::Min(3), Constraint::Min(3), Constraint::Length(1), if has_error { Constraint::Length(3) } else { Constraint::Length(0) }],
        PopupType::Transfer { .. } => vec![Constraint::Min(3), Constraint::Min(3), Constraint::Min(3), if has_error { Constraint::Length(3) } else { Constraint::Length(0) }],
        PopupType::AddCategory { .. } => vec![Constraint::Min(3), if has_error { Constraint::Length(3) } else { Constraint::Length(0) }],
        PopupType::AddBudget { .. } => vec![Constraint::Min(3), Constraint::Min(3), if has_error { Constraint::Length(3) } else { Constraint::Length(0) }],
        PopupType::DeleteConfirm { .. } => vec![Constraint::Min(2), Constraint::Min(3), if has_error { Constraint::Length(3) } else { Constraint::Length(0) }],
    };
    
    let layout = Layout::default().direction(Direction::Vertical).margin(2).constraints(constraints).split(area);
    let mut layout_idx = 0;

    match popup {
        PopupType::AddAccount { step, name, currency } => {
            f.render_widget(block.title("New Account"), area);
            f.render_widget(Paragraph::new(name.as_str()).block(Block::default().borders(Borders::ALL).title("Name")).style(st(*step, 0)), layout[layout_idx]);
            layout_idx += 1;
            f.render_widget(Paragraph::new(currency.as_str()).block(Block::default().borders(Borders::ALL).title("Currency")).style(st(*step, 1)), layout[layout_idx]);
            layout_idx += 1;
        },
        PopupType::AddTransaction { step, amount, desc, category_input } => {
            let match_hint = if let Some((id, name)) = app.resolve_category(category_input) { format!("Matched: [{}] {}", id, name) } else if category_input.trim().is_empty() { "(Optional) Leave empty".to_string() } else { "No match found".to_string() };
            let acc_name = if let Some(acc) = app.get_selected_account() { acc.name.clone() } else { "None".to_string() };
            f.render_widget(block.title(format!("New Tx for: {}", acc_name)), area);
            f.render_widget(Paragraph::new(amount.as_str()).block(Block::default().borders(Borders::ALL).title("Amount")).style(st(*step, 0)), layout[layout_idx]);
            layout_idx += 1;
            f.render_widget(Paragraph::new(desc.as_str()).block(Block::default().borders(Borders::ALL).title("Desc")).style(st(*step, 1)), layout[layout_idx]);
            layout_idx += 1;
            f.render_widget(Paragraph::new(category_input.as_str()).block(Block::default().borders(Borders::ALL).title("Category (ID or Name)")).style(st(*step, 2)), layout[layout_idx]);
            layout_idx += 1;
            f.render_widget(Paragraph::new(match_hint).style(Style::default().fg(Color::Cyan)), layout[layout_idx]);
            layout_idx += 1;
        },
        PopupType::Transfer { step, from_id, to_id, amount } => {
            f.render_widget(block.title("Transfer"), area);
            f.render_widget(Paragraph::new(from_id.as_str()).block(Block::default().borders(Borders::ALL).title("From ID")).style(st(*step, 0)), layout[layout_idx]);
            layout_idx += 1;
            f.render_widget(Paragraph::new(to_id.as_str()).block(Block::default().borders(Borders::ALL).title("To ID")).style(st(*step, 1)), layout[layout_idx]);
            layout_idx += 1;
            f.render_widget(Paragraph::new(amount.as_str()).block(Block::default().borders(Borders::ALL).title("Amount")).style(st(*step, 2)), layout[layout_idx]);
            layout_idx += 1;
        },
        PopupType::AddCategory { name, .. } => {
             f.render_widget(block.title("New Category"), area);
             f.render_widget(Paragraph::new(name.as_str()).block(Block::default().borders(Borders::ALL).title("Name")).style(st(0, 0)), layout[layout_idx]);
             layout_idx += 1;
        },
        PopupType::AddBudget { step, amount, category_id } => {
            f.render_widget(block.title("New Budget"), area);
            f.render_widget(Paragraph::new(amount.as_str()).block(Block::default().borders(Borders::ALL).title("Amount")).style(st(*step, 0)), layout[layout_idx]);
            layout_idx += 1;
            f.render_widget(Paragraph::new(category_id.as_str()).block(Block::default().borders(Borders::ALL).title("Category ID (ID only)")).style(st(*step, 1)), layout[layout_idx]);
            layout_idx += 1;
        },
        PopupType::DeleteConfirm { type_label, target_id: _, verify_name, input_name } => {
            f.render_widget(block.title(Span::styled(format!("DELETE {}", type_label.to_uppercase()), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))), area);
            let warn_text = format!("Type name '{}' to confirm:", verify_name);
            f.render_widget(Paragraph::new(warn_text).style(Style::default().fg(Color::Red)), layout[layout_idx]);
            layout_idx += 1;
            f.render_widget(Paragraph::new(input_name.as_str()).block(Block::default().borders(Borders::ALL).title("Confirmation")).style(Style::default().fg(Color::Red)), layout[layout_idx]);
            layout_idx += 1;
        }
    };

    if has_error && layout_idx < layout.len() {
        if let Some((msg, color)) = &app.message {
            f.render_widget(
                Paragraph::new(msg.as_str())
                    .style(Style::default().bg(*color).fg(Color::White).add_modifier(Modifier::BOLD))
                    .alignment(Alignment::Center)
                    .block(Block::default().borders(Borders::ALL).title("Error")),
                layout[layout_idx]
            );
        }
    }
}

fn centered_rect_percent(width: u16, height: u16, r: Rect) -> Rect {
    let vertical_margin = (r.height.saturating_sub(height)) / 2;
    let horizontal_margin = (r.width.saturating_sub(width)) / 2;
    
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(vertical_margin),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(horizontal_margin),
            Constraint::Length(width),
            Constraint::Min(0),
        ])
        .split(popup_layout[1])[1]
}
