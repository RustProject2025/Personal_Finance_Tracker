mod models;
mod api;

use std::{io, time::Duration};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use api::ApiClient;
use models::{LoginRequest, AccountResponse, TransactionResponse, BudgetResponse};


enum AppState {
    Login,
    Dashboard,
}


enum InputMode {
    Username,
    Password,
}

struct App {
    state: AppState,
    api: ApiClient,
    
 
    input_username: String,
    input_password: String,
    input_mode: InputMode,
    login_error: Option<String>,

  
    accounts: Vec<AccountResponse>,
    transactions: Vec<TransactionResponse>,
    budgets: Vec<BudgetResponse>,
}

impl App {
    fn new() -> Self {
        Self {
            state: AppState::Login,
            api: ApiClient::new(),
            input_username: String::new(),
            input_password: String::new(),
            input_mode: InputMode::Username,
            login_error: None,
            accounts: vec![],
            transactions: vec![],
            budgets: vec![],
        }
    }


    async fn try_login(&mut self) {
        let req = LoginRequest {
            username: self.input_username.clone(),
            password: self.input_password.clone(),
        };

        match self.api.login(req).await {
            Ok(_) => {
                self.state = AppState::Dashboard;
                self.refresh_all_data().await;
            }
            Err(e) => self.login_error = Some(e.to_string()),
        }
    }


    async fn refresh_all_data(&mut self) {
     
        if let Ok(data) = self.api.get_accounts().await {
            self.accounts = data;
        }
        if let Ok(data) = self.api.get_transactions().await {
            self.transactions = data;
        }
        if let Ok(data) = self.api.get_budgets().await {
            self.budgets = data;
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
                            KeyCode::Char(c) => match app.input_mode {
                                InputMode::Username => app.input_username.push(c),
                                InputMode::Password => app.input_password.push(c),
                            },
                            KeyCode::Backspace => match app.input_mode {
                                InputMode::Username => { app.input_username.pop(); }
                                InputMode::Password => { app.input_password.pop(); }
                            },
                            KeyCode::Tab => {
                                app.input_mode = match app.input_mode {
                                    InputMode::Username => InputMode::Password,
                                    InputMode::Password => InputMode::Username,
                                };
                            }
                            KeyCode::Enter => {
                                app.try_login().await;
                            }
                            KeyCode::Esc => break, 
                            _ => {}
                        }
                    }
                    AppState::Dashboard => {
                        if let KeyCode::Esc = key.code {
                            break;
                        }
                        if let KeyCode::Char('r') = key.code {
                            app.refresh_all_data().await; 
                        }
                    }
                }
            }
        }
    }


    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let size = f.size();

    match app.state {
        AppState::Login => {
            render_login(f, app, size);
        }
        AppState::Dashboard => {
            render_dashboard(f, app, size);
        }
    }
}

fn render_login(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Length(3), Constraint::Min(1)])
        .split(area);

    let title = Paragraph::new("Personal Finance Tracker")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    let username_block = Paragraph::new(app.input_username.as_str())
        .style(if let InputMode::Username = app.input_mode { Style::default().fg(Color::Yellow) } else { Style::default() })
        .block(Block::default().borders(Borders::ALL).title("Username"));
    f.render_widget(username_block, chunks[1]);
    
    let pass_mask: String = app.input_password.chars().map(|_| '*').collect();
    let password_block = Paragraph::new(pass_mask.as_str())
        .style(if let InputMode::Password = app.input_mode { Style::default().fg(Color::Yellow) } else { Style::default() })
        .block(Block::default().borders(Borders::ALL).title("Password"));
    f.render_widget(password_block, chunks[2]);

    if let Some(err) = &app.login_error {
        let err_msg = Paragraph::new(format!("Error: {}", err))
            .style(Style::default().fg(Color::Red));
        f.render_widget(err_msg, chunks[3]);
    }
}

fn render_dashboard(f: &mut Frame, app: &App, area: Rect) {
  
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let header = Paragraph::new("Dashboard (Press 'r' to refresh, 'Esc' to quit)")
        .style(Style::default().fg(Color::White).bg(Color::Blue))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, vertical_chunks[0]);

 
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20), // Left: Accounts
            Constraint::Percentage(50), // Center: Transactions
            Constraint::Percentage(30), // Right: Budgets
        ])
        .split(vertical_chunks[1]);

    
    let account_items: Vec<ListItem> = app.accounts.iter().map(|acc| {
        let content = format!("{}\n  {} {}", acc.name, acc.currency, acc.balance);
        ListItem::new(content).style(Style::default().fg(Color::Cyan))
    }).collect();
    
    let accounts_list = List::new(account_items)
        .block(Block::default().borders(Borders::ALL).title("Accounts"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));
    f.render_widget(accounts_list, main_chunks[0]);

 
    let header_cells = ["Date", "Desc", "Category", "Amount"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let rows = app.transactions.iter().map(|t| {
        let amount_style = if t.r#type == "expense" { 
            Style::default().fg(Color::Red) 
        } else { 
            Style::default().fg(Color::Green) 
        };
        
        let cells = vec![
            Cell::from(t.date.clone()),
            Cell::from(t.description.clone().unwrap_or_default()),
            Cell::from(t.category_name.clone().unwrap_or_default()),
            Cell::from(t.amount.clone()).style(amount_style),
        ];
        Row::new(cells).height(1)
    });

    let t_table = Table::new(rows, [
            Constraint::Length(12), // Date
            Constraint::Percentage(40), // Desc
            Constraint::Percentage(30), // Category
            Constraint::Length(10), // Amount
        ])
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Recent Transactions"));
    f.render_widget(t_table, main_chunks[1]);


    let budget_block = Block::default().borders(Borders::ALL).title("Budgets");
    f.render_widget(budget_block, main_chunks[2]);


    let budget_area = main_chunks[2].inner(&Margin { vertical: 1, horizontal: 1 });
    let budget_constraints: Vec<Constraint> = app.budgets.iter()
        .map(|_| Constraint::Length(3)) 
        .collect();
    
    let budget_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(budget_constraints)
        .split(budget_area);

    for (i, budget) in app.budgets.iter().enumerate() {
        if i >= budget_chunks.len() { break; } 

        let spent = budget.spent.parse::<f64>().unwrap_or(0.0);
        let amount = budget.amount.parse::<f64>().unwrap_or(1.0); 
        let mut ratio = spent / amount;
        if ratio > 1.0 { ratio = 1.0; }

        let label = format!("{}: {} / {}", 
            budget.category_name.clone().unwrap_or("Total".to_string()), 
            budget.spent, 
            budget.amount
        );

        let color = if budget.is_over_budget { Color::Red } else { Color::Green };

        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::NONE))
            .gauge_style(Style::default().fg(color))
            .ratio(ratio)
            .label(label);
        
        f.render_widget(gauge, budget_chunks[i]);
    }
}