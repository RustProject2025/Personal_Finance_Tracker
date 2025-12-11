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
use models::{LoginRequest, AccountResponse};


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
                self.fetch_data().await;
            }
            Err(e) => self.login_error = Some(e.to_string()),
        }
    }

    async fn fetch_data(&mut self) {
        if let Ok(accounts) = self.api.get_accounts().await {
            self.accounts = accounts;
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
                            app.fetch_data().await; 
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
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Min(1)])
                .split(size);

            let title = Paragraph::new("Personal Finance Tracker - Login")
                .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center);
            f.render_widget(title, chunks[0]);

            let username_style = if let InputMode::Username = app.input_mode { Style::default().fg(Color::Yellow) } else { Style::default() };
            let password_style = if let InputMode::Password = app.input_mode { Style::default().fg(Color::Yellow) } else { Style::default() };

            let username_block = Paragraph::new(app.input_username.as_str())
                .style(username_style)
                .block(Block::default().borders(Borders::ALL).title("Username"));
            f.render_widget(username_block, chunks[1]);
            
        
            let pass_mask: String = app.input_password.chars().map(|_| '*').collect();
            let password_block = Paragraph::new(pass_mask.as_str())
                .style(password_style)
                .block(Block::default().borders(Borders::ALL).title("Password (Press TAB to switch, ENTER to login)"));
            f.render_widget(password_block, chunks[2]); 
        }
        AppState::Dashboard => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(size);

            let title = Paragraph::new("Dashboard (Press 'r' to refresh, 'Esc' to quit)")
                .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(title, chunks[0]);

  
            let items: Vec<ListItem> = app.accounts.iter().map(|acc| {
                let content = format!("{} ({}): ${}", acc.name, acc.currency, acc.balance);
                ListItem::new(content)
            }).collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Accounts"));
            f.render_widget(list, chunks[1]);
        }
    }
}