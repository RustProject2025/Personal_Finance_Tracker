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
use models::{LoginRequest, AccountResponse, TransactionResponse, BudgetResponse};

enum AppState {
    Login,
    Dashboard,
}

#[derive(PartialEq)] // 为了方便比较，加上 PartialEq
enum InputMode {
    Username,
    Password,
}

struct App {
    state: AppState,
    api: ApiClient,
    
    // 登录/注册共用输入框
    input_username: String,
    input_password: String,
    input_mode: InputMode,
    
    // 新增：是否处于注册模式
    is_register_mode: bool,
    
    // 用于显示错误或成功消息
    message: Option<(String, Color)>, // (文本, 颜色)

    // 数据缓存
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
            is_register_mode: false, // 默认为登录模式
            message: None,
            accounts: vec![],
            transactions: vec![],
            budgets: vec![],
        }
    }

    // 切换 登录/注册 模式
    fn toggle_auth_mode(&mut self) {
        self.is_register_mode = !self.is_register_mode;
        self.message = None; // 切换时清空消息
    }

    // 处理注册
    async fn try_register(&mut self) {
        let req = LoginRequest { // 复用结构体
            username: self.input_username.clone(),
            password: self.input_password.clone(),
        };

        match self.api.register(req).await {
            Ok(msg) => {
                // 注册成功，切回登录模式，并显示绿色成功消息
                self.is_register_mode = false;
                self.message = Some((format!("Success: {}. Please Login.", msg), Color::Green));
            }
            Err(e) => {
                self.message = Some((format!("Register Error: {}", e), Color::Red));
            }
        }
    }

    // 处理登录
    async fn try_login(&mut self) {
        let req = LoginRequest {
            username: self.input_username.clone(),
            password: self.input_password.clone(),
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
    // 1. 设置终端
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 2. 初始化 App
    let mut app = App::new();

    // 3. 主循环
    loop {
        terminal.draw(|f| ui(f, &app))?;

        // 简单的事件轮询
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match app.state {
                    AppState::Login => {
                        match key.code {
                            // 切换输入框焦点 (Tab)
                            KeyCode::Tab => {
                                app.input_mode = match app.input_mode {
                                    InputMode::Username => InputMode::Password,
                                    InputMode::Password => InputMode::Username,
                                };
                            }
                            // 切换 注册/登录 模式 (Ctrl + r)
                            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                app.toggle_auth_mode();
                            }
                            // 输入字符
                            KeyCode::Char(c) => {
                                // 只有没有按 Control 键时才输入字符
                                if !key.modifiers.contains(KeyModifiers::CONTROL) {
                                    match app.input_mode {
                                        InputMode::Username => app.input_username.push(c),
                                        InputMode::Password => app.input_password.push(c),
                                    }
                                }
                            }
                            KeyCode::Backspace => match app.input_mode {
                                InputMode::Username => { app.input_username.pop(); }
                                InputMode::Password => { app.input_password.pop(); }
                            },
                            // 提交 (Enter)
                            KeyCode::Enter => {
                                if app.is_register_mode {
                                    app.try_register().await;
                                } else {
                                    app.try_login().await;
                                }
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

    // 4. 恢复终端
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
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Username
            Constraint::Length(3), // Password
            Constraint::Length(3), // Instructions
            Constraint::Min(1)     // Messages
        ])
        .split(area);

    // 1. 标题 (根据模式变化)
    let (title_text, title_color) = if app.is_register_mode {
        ("REGISTER NEW ACCOUNT", Color::Magenta)
    } else {
        ("PERSONAL FINANCE TRACKER - LOGIN", Color::Cyan)
    };

    let title = Paragraph::new(title_text)
        .style(Style::default().fg(title_color).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    // 2. 输入框样式 (高亮当前选中的)
    let username_border_style = if app.input_mode == InputMode::Username { Style::default().fg(Color::Yellow) } else { Style::default() };
    let password_border_style = if app.input_mode == InputMode::Password { Style::default().fg(Color::Yellow) } else { Style::default() };

    let username_block = Paragraph::new(app.input_username.as_str())
        .style(Style::default())
        .block(Block::default().borders(Borders::ALL).title("Username").border_style(username_border_style));
    f.render_widget(username_block, chunks[1]);
    
    let pass_mask: String = app.input_password.chars().map(|_| '*').collect();
    let password_block = Paragraph::new(pass_mask.as_str())
        .style(Style::default())
        .block(Block::default().borders(Borders::ALL).title("Password").border_style(password_border_style));
    f.render_widget(password_block, chunks[2]);

    // 3. 操作指引
    let mode_switch_text = if app.is_register_mode {
        "Switch to Login (Ctrl+r)"
    } else {
        "Switch to Register (Ctrl+r)"
    };
    
    let instructions = Paragraph::new(Line::from(vec![
        Span::raw("Press "),
        Span::styled("Tab", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to switch fields, "),
        Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to submit. "),
        Span::styled(mode_switch_text, Style::default().fg(Color::LightBlue)),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(instructions, chunks[3]);

    // 4. 消息提示 (错误或成功)
    if let Some((msg, color)) = &app.message {
        let msg_widget = Paragraph::new(msg.as_str())
            .style(Style::default().fg(*color))
            .alignment(Alignment::Center);
        f.render_widget(msg_widget, chunks[4]);
    }
}

// ... render_dashboard 代码保持不变，使用你之前提供的版本即可 ...
fn render_dashboard(f: &mut Frame, app: &App, area: Rect) {
    // 顶部标题栏
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let header = Paragraph::new("Dashboard (Press 'r' to refresh, 'Esc' to quit)")
        .style(Style::default().fg(Color::White).bg(Color::Blue))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, vertical_chunks[0]);

    // 主体三列布局：账户(20%) | 交易(50%) | 预算(30%)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20), // Left: Accounts
            Constraint::Percentage(50), // Center: Transactions
            Constraint::Percentage(30), // Right: Budgets
        ])
        .split(vertical_chunks[1]);

    // 1. 左侧：账户列表
    let account_items: Vec<ListItem> = app.accounts.iter().map(|acc| {
        let content = format!("{}\n  {} {}", acc.name, acc.currency, acc.balance);
        ListItem::new(content).style(Style::default().fg(Color::Cyan))
    }).collect();
    
    let accounts_list = List::new(account_items)
        .block(Block::default().borders(Borders::ALL).title("Accounts"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));
    f.render_widget(accounts_list, main_chunks[0]);

    // 2. 中间：交易记录表格
    // 这里的 header 和 row 需要对齐
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

    // 3. 右侧：预算情况 (使用 Gauge 进度条)
    // Ratatui 的 List 不能直接包含 Gauge，我们需要手动切割区域
    let budget_block = Block::default().borders(Borders::ALL).title("Budgets");
    f.render_widget(budget_block, main_chunks[2]);

    // 在右侧区域内，为每个预算切分出一小块区域
    let budget_area = main_chunks[2].inner(&Margin { vertical: 1, horizontal: 1 });
    let budget_constraints: Vec<Constraint> = app.budgets.iter()
        .map(|_| Constraint::Length(3)) // 每个预算占3行
        .collect();
    
    let budget_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(budget_constraints)
        .split(budget_area);

    for (i, budget) in app.budgets.iter().enumerate() {
        if i >= budget_chunks.len() { break; } // 防止溢出

        let spent = budget.spent.parse::<f64>().unwrap_or(0.0);
        let amount = budget.amount.parse::<f64>().unwrap_or(1.0); // 避免除以0
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