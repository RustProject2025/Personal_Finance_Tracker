mod models;
mod api;

use std::{io, time::Duration};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use api::ApiClient;
use models::*;


enum AppState {
    Login,
    Dashboard,
    InputPopup(PopupType),
}


enum PopupType {
    AddAccount { step: usize, name: String, currency: String },
    AddTransaction { step: usize, amount: String, desc: String, category_name: String }, 
    Transfer { step: usize, from_id: String, to_id: String, amount: String },
    AddCategory { step: usize, name: String },
    AddBudget { step: usize, amount: String, category_id: String },
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

    // UI State
    account_list_state: ListState, 
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
            account_list_state: list_state,
        }
    }


    fn get_selected_account_id(&self) -> Option<i32> {
        if let Some(index) = self.account_list_state.selected() {
            self.accounts.get(index).map(|acc| acc.id)
        } else {
            None
        }
    }

 
    fn find_category_id_by_name(&self, name: &str) -> Option<i32> {
     
        self.categories.iter()
            .find(|c| c.name.eq_ignore_ascii_case(name))
            .map(|c| c.id)
    }

    
    fn toggle_auth_mode(&mut self) {
        self.is_register_mode = !self.is_register_mode;
        self.message = None;
    }

    async fn try_register(&mut self) {
        let req = LoginRequest { 
            username: self.input_username.clone(), 
            password: self.input_password.clone() 
        };
        match self.api.register(req).await {
            Ok(msg) => {
                self.is_register_mode = false;
                self.message = Some((format!("Success: {}. Please Login.", msg), Color::Green));
            }
            Err(e) => self.message = Some((format!("Register Error: {}", e), Color::Red)),
        }
    }

    async fn try_login(&mut self) {
        let req = LoginRequest { 
            username: self.input_username.clone(), 
            password: self.input_password.clone() 
        };
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
        
        let selected_id = self.get_selected_account_id();
        
        match self.api.get_transactions(selected_id).await {
            Ok(data) => self.transactions = data,
            Err(_) => self.transactions = vec![],
        }
    }

  
    fn next_account(&mut self) {
        if self.accounts.is_empty() { return; }
        let i = match self.account_list_state.selected() {
            Some(i) => {
                if i >= self.accounts.len() - 1 { 0 } else { i + 1 }
            }
            None => 0,
        };
        self.account_list_state.select(Some(i));
    }

    fn previous_account(&mut self) {
        if self.accounts.is_empty() { return; }
        let i = match self.account_list_state.selected() {
            Some(i) => {
                if i == 0 { self.accounts.len() - 1 } else { i - 1 }
            }
            None => 0,
        };
        self.account_list_state.select(Some(i));
    }


    async fn submit_popup(&mut self) {
        if let AppState::InputPopup(ref popup) = self.state {
            let result = match popup {
                PopupType::AddAccount { name, currency, .. } => {
                    self.api.create_account(CreateAccountRequest { 
                        name: name.clone(), 
                        currency: Some(currency.clone()) 
                    }).await
                },
                
              
                PopupType::AddTransaction { amount, desc, category_name, .. } => {
                  
                    let acc_id = self.get_selected_account_id();
                    
                    if acc_id.is_none() {
                        Err(anyhow::anyhow!("Please select an account on the left first!"))
                    } else {
                       
                        let cat_id = self.find_category_id_by_name(category_name);

                        let amount_val = amount.parse::<f64>().unwrap_or(0.0);
                        let is_expense = amount_val < 0.0;

                        self.api.create_transaction(CreateTransactionRequest {
                            account_id: acc_id,
                            account_name: None, 
                            category_id: cat_id, 
                            amount: amount.clone(),
                            r#type: if !is_expense { "income".to_string() } else { "expense".to_string() },
                            date: chrono::Local::now().format("%Y-%m-%d").to_string(),
                            description: Some(desc.clone()),
                        }).await
                    }
                },
                
                PopupType::Transfer { from_id, to_id, amount, .. } => {
                    let f_id = from_id.parse().unwrap_or(0);
                    let t_id = to_id.parse().unwrap_or(0);
                    self.api.transfer(TransferRequest {
                        from_account_id: f_id,
                        to_account_id: t_id,
                        amount: amount.clone(),
                        date: None,
                        description: Some("TUI Transfer".to_string()),
                    }).await
                },
                PopupType::AddCategory { name, .. } => {
                    self.api.create_category(CreateCategoryRequest {
                        name: name.clone(),
                        parent_id: None
                    }).await
                },
                PopupType::AddBudget { amount, category_id, .. } => {
                    let cat_id = if category_id.is_empty() { None } else { Some(category_id.parse().unwrap_or(0)) };
                    self.api.create_budget(CreateBudgetRequest {
                        category_id: cat_id,
                        amount: amount.clone(),
                        period: Some("monthly".to_string()),
                        start_date: None
                    }).await
                }
            };

            match result {
                Ok(_) => self.message = Some(("Action Successful!".to_string(), Color::Green)),
                Err(e) => self.message = Some((format!("Error: {}", e), Color::Red)),
            }
       
            self.refresh_all_data().await;
            self.state = AppState::Dashboard;
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
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
                            
                          
                            KeyCode::Down => {
                                app.next_account();
                                app.refresh_transactions().await; 
                            },
                            KeyCode::Up => {
                                app.previous_account();
                                app.refresh_transactions().await;
                            },

                            
                            KeyCode::Char('a') => app.state = AppState::InputPopup(PopupType::AddAccount { step: 0, name: String::new(), currency: "USD".to_string() }),
                            
                            
                            KeyCode::Char('t') => {
                                if app.accounts.is_empty() {
                                    app.message = Some(("Please create an account first!".to_string(), Color::Red));
                                } else {
                                    app.state = AppState::InputPopup(PopupType::AddTransaction { 
                                        step: 0, 
                                        amount: String::new(), 
                                        desc: String::new(), 
                                        category_name: String::new() 
                                    });
                                }
                            },
                            
                            KeyCode::Char('x') => app.state = AppState::InputPopup(PopupType::Transfer { step: 0, from_id: String::new(), to_id: String::new(), amount: String::new() }),
                            KeyCode::Char('c') => app.state = AppState::InputPopup(PopupType::AddCategory { step: 0, name: String::new() }),
                            KeyCode::Char('b') => app.state = AppState::InputPopup(PopupType::AddBudget { step: 0, amount: String::new(), category_id: String::new() }),
                            _ => {}
                        }
                    }

                    AppState::InputPopup(ref mut popup) => {
                        match key.code {
                            KeyCode::Esc => app.state = AppState::Dashboard,
                            KeyCode::Enter => app.submit_popup().await,
                            KeyCode::Tab => {
                                match popup {
                                    PopupType::AddAccount { step, .. } => *step = (*step + 1) % 2,
                                   
                                    PopupType::AddTransaction { step, .. } => *step = (*step + 1) % 3,
                                    PopupType::Transfer { step, .. } => *step = (*step + 1) % 3,
                                    PopupType::AddBudget { step, .. } => *step = (*step + 1) % 2,
                                    _ => {}
                                }
                            },
                            KeyCode::Char(c) => {
                                match popup {
                                    PopupType::AddAccount { step, name, currency } => if *step == 0 { name.push(c) } else { currency.push(c) },
                                    
                                  
                                    PopupType::AddTransaction { step, amount, desc, category_name } => {
                                        match step {
                                            0 => amount.push(c),
                                            1 => desc.push(c),
                                            2 => category_name.push(c),
                                            _ => {}
                                        }
                                    },
                                    
                                    PopupType::Transfer { step, from_id, to_id, amount } => { match step { 0 => from_id.push(c), 1 => to_id.push(c), 2 => amount.push(c), _ => {} } },
                                    PopupType::AddCategory { name, .. } => name.push(c),
                                    PopupType::AddBudget { step, amount, category_id } => if *step == 0 { amount.push(c) } else { category_id.push(c) },
                                }
                            },
                            KeyCode::Backspace => {
                                match popup {
                                    PopupType::AddAccount { step, name, currency } => if *step == 0 { name.pop(); } else { currency.pop(); },
                                    PopupType::AddTransaction { step, amount, desc, category_name } => { match step { 0 => {amount.pop();}, 1 => {desc.pop();}, 2 => {category_name.pop();}, _ => {} } },
                                    PopupType::Transfer { step, from_id, to_id, amount } => { match step { 0 => {from_id.pop();}, 1 => {to_id.pop();}, 2 => {amount.pop();}, _ => {} } },
                                    PopupType::AddCategory { name, .. } => { name.pop(); },
                                    PopupType::AddBudget { step, amount, category_id } => if *step == 0 { amount.pop(); } else { category_id.pop(); },
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
    let username_block = Paragraph::new(app.input_username.as_str()).block(Block::default().borders(Borders::ALL).title("Username").border_style(if app.input_mode == InputMode::Username { Style::default().fg(Color::Yellow) } else { Style::default() }));
    f.render_widget(username_block, chunks[1]);
    let pass_mask: String = app.input_password.chars().map(|_| '*').collect();
    let password_block = Paragraph::new(pass_mask.as_str()).block(Block::default().borders(Borders::ALL).title("Password").border_style(if app.input_mode == InputMode::Password { Style::default().fg(Color::Yellow) } else { Style::default() }));
    f.render_widget(password_block, chunks[2]);
    let mode_txt = if app.is_register_mode { "Switch to Login (Ctrl+r)" } else { "Switch to Register (Ctrl+r)" };
    f.render_widget(Paragraph::new(mode_txt).alignment(Alignment::Center), chunks[3]);
    if let Some((msg, color)) = &app.message { f.render_widget(Paragraph::new(msg.as_str()).style(Style::default().fg(*color)).alignment(Alignment::Center), chunks[4]); }
}

fn render_dashboard(f: &mut Frame, app: &App, area: Rect) {
    let vertical_chunks = Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(3), Constraint::Min(0)]).split(area);

    let help_text = "Nav: ↑/↓ | 't' Add Tx | 'a' Add Acc | 'x' Transfer | 'c' Category | 'b' Budget | 'r' Refresh";
    let header = Paragraph::new(help_text).style(Style::default().fg(Color::White).bg(Color::Blue)).alignment(Alignment::Center).block(Block::default().borders(Borders::ALL));
    f.render_widget(header, vertical_chunks[0]);

    let main_chunks = Layout::default().direction(Direction::Horizontal).constraints([Constraint::Percentage(25), Constraint::Percentage(45), Constraint::Percentage(30)]).split(vertical_chunks[1]);

   
    let account_items: Vec<ListItem> = app.accounts.iter().map(|acc| {
        let content = format!("[#{}] {} : {}", acc.id, acc.name, acc.balance);
        ListItem::new(content).style(Style::default().fg(Color::Cyan))
    }).collect();
    
  
    let accounts_list = List::new(account_items)
        .block(Block::default().borders(Borders::ALL).title("Accounts (Select)"))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)) // 高亮样式
        .highlight_symbol(">> "); // 选中箭头
        
    f.render_stateful_widget(accounts_list, main_chunks[0], &mut app.account_list_state.clone()); // 渲染选中状态

 
    let tx_title = if let Some(id) = app.get_selected_account_id() {
        format!("Transactions (Account #{})", id)
    } else {
        "Transactions (All)".to_string()
    };

    let header_cells = ["Date", "Description", "Category", "Amt"].iter().map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);
    let rows = app.transactions.iter().map(|t| {
        let amount_style = if t.r#type == "expense" { Style::default().fg(Color::Red) } else { Style::default().fg(Color::Green) };
        let cells = vec![
            Cell::from(t.date.clone()),
            Cell::from(t.description.clone().unwrap_or_default()),
            Cell::from(t.category_name.clone().unwrap_or_default()),
            Cell::from(t.amount.clone()).style(amount_style),
        ];
        Row::new(cells).height(1)
    });
    let t_table = Table::new(rows, [Constraint::Length(10), Constraint::Percentage(40), Constraint::Percentage(20), Constraint::Length(10)])
        .header(header).block(Block::default().borders(Borders::ALL).title(tx_title));
    f.render_widget(t_table, main_chunks[1]);

  
    let right_chunks = Layout::default().direction(Direction::Vertical).constraints([Constraint::Percentage(50), Constraint::Percentage(50)]).split(main_chunks[2]);
    
    // Budgets
    let b_constraints: Vec<Constraint> = app.budgets.iter().map(|_| Constraint::Length(2)).collect();
    let b_layout = Layout::default().direction(Direction::Vertical).constraints(b_constraints).split(right_chunks[0]);
    f.render_widget(Block::default().borders(Borders::ALL).title("Budgets"), right_chunks[0]);
    
    for (i, budget) in app.budgets.iter().enumerate() {
        if i >= b_layout.len() { break; }
        let spent = budget.spent.parse::<f64>().unwrap_or(0.0);
        let amount = budget.amount.parse::<f64>().unwrap_or(1.0);
        let ratio = (spent / amount).min(1.0);
        let label = format!("{} {}/{}", budget.category_name.clone().unwrap_or("Total".to_string()), budget.spent, budget.amount);
        let color = if budget.is_over_budget { Color::Red } else { Color::Green };
        let gauge_area = b_layout[i].inner(&Margin{horizontal:1, vertical:0});
        f.render_widget(Gauge::default().gauge_style(Style::default().fg(color)).ratio(ratio).label(label), gauge_area);
    }

   
    let cat_items: Vec<ListItem> = app.categories.iter().map(|c| ListItem::new(format!("{}: {}", c.id, c.name))).collect();
    f.render_widget(List::new(cat_items).block(Block::default().borders(Borders::ALL).title("Available Categories")), right_chunks[1]);

    // Message
    if let Some((msg, color)) = &app.message {
        let msg_area = Rect { x: area.x, y: area.height.saturating_sub(1), width: area.width, height: 1 };
        f.render_widget(Paragraph::new(msg.as_str()).style(Style::default().bg(*color).fg(Color::Black)), msg_area);
    }
}

fn render_popup(f: &mut Frame, popup: &PopupType, area: Rect, app: &App) {
    let area = centered_rect(60, 25, area);
    f.render_widget(Clear, area);
    let block = Block::default().borders(Borders::ALL).style(Style::default().bg(Color::DarkGray));

    let (title, content) = match popup {
        PopupType::AddAccount { step, name, currency } => {
            ("New Account", format!("Name: {} {}\nCurrency: {} {}", name, if *step==0 {"<"} else {""}, currency, if *step==1 {"<"} else {""}))
        },
        
        
        PopupType::AddTransaction { step, amount, desc, category_name, .. } => {
           
            let acc_name = if let Some(idx) = app.account_list_state.selected() {
                app.accounts.get(idx).map(|a| a.name.clone()).unwrap_or("Unknown".to_string())
            } else { "None".to_string() };

            (
                "New Transaction",
                format!("Account: {} (Locked)\nAmount: {} {}\nDescription: {} {}\nCategory Name: {} {} \n(Type 'Food' etc.)", 
                    acc_name,
                    amount, if *step==0 {"<"} else {""},
                    desc, if *step==1 {"<"} else {""},
                    category_name, if *step==2 {"<"} else {""})
            )
        },
        
        PopupType::Transfer { step, from_id, to_id, amount } => {
            ("Transfer", format!("From ID: {} {}\nTo ID: {} {}\nAmount: {} {}", from_id, if *step==0 {"<"} else {""}, to_id, if *step==1 {"<"} else {""}, amount, if *step==2 {"<"} else {""}))
        },
        PopupType::AddCategory { name, .. } => ("New Category", format!("Name: {} <", name)),
        PopupType::AddBudget { step, amount, category_id } => {
            ("New Budget", format!("Amount: {} {}\nCategory ID: {} {}", amount, if *step==0 {"<"} else {""}, category_id, if *step==1 {"<"} else {""}))
        }
    };

    let p = Paragraph::new(content).block(block.title(title)).alignment(Alignment::Center);
    f.render_widget(p, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default().direction(Direction::Vertical).constraints([Constraint::Percentage((100 - percent_y) / 2), Constraint::Percentage(percent_y), Constraint::Percentage((100 - percent_y) / 2)]).split(r);
    Layout::default().direction(Direction::Horizontal).constraints([Constraint::Percentage((100 - percent_x) / 2), Constraint::Percentage(percent_x), Constraint::Percentage((100 - percent_x) / 2)]).split(popup_layout[1])[1]
}