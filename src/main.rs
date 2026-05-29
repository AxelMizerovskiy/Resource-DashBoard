use sysinfo::{System, Disks, Networks};
use std::{io, time::{Duration, Instant}};
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    Terminal, backend::CrosstermBackend, layout::{Constraint, Direction, Layout}, style::{Color, Style}, symbols, widgets::{Axis, Block, Borders, Chart, Dataset, Gauge, GraphType, Paragraph}
};


// argument structs
#[derive(Parser)]
#[command(name = "rdb")]
#[command(about = "A Rust resource dashboard")]
struct Cli {
    /// Polling interval in seconds
    #[arg(short, long, default_value_t = 2)]
    timeout: u64,
}


fn main() -> Result<(), io::Error>{
    // parse args
    let args = Cli::parse();
    
    // setup the terminal takeover
    enable_raw_mode()?; // default error
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?; // default error
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?; // default error

    // setup System and timer
    let mut sys = System::new_all();
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_secs(args.timeout);

    // setup history chart
    let mut cpu_history: Vec<f64> = vec![];
    let mut dataset_data: Vec<(f64, f64)> = vec![]; // storing data for graph
    let max_history_bars = (60 / args.timeout.max(1)) as usize; // for minute history
                                                                 
    // state variables
    let mut current_cpu: f32 = 0.0;
    let mut used_memory:  u64 = 0;
    let mut total_memory: u64 = 0;

    // network state stats
    let mut networks = Networks::new_with_refreshed_list();
    let mut rx_bytes: u64 = 0;
    let mut tx_bytes: u64 = 0;

    //disk state stats
    let mut disks = Disks::new_with_refreshed_list();
    let mut disk_state: Vec<(String, u64, u64)> = vec![];

    // main monitoring loop
    loop {
        // Refresh sys info for latest stats after timeout only
        if last_tick.elapsed() >= tick_rate {
            sys.refresh_all(); // could refresh cpu and mem instead 

            // update state variables
            current_cpu = sys.global_cpu_info().cpu_usage();
            cpu_history.push(current_cpu as f64);
            used_memory = sys.used_memory() as u64 / 1024 / 1024;
            total_memory= sys.total_memory() as u64 / 1024 / 1024;
            
            // get tick after grabbing it
            last_tick = Instant::now();

            if cpu_history.len() > max_history_bars {
                cpu_history.remove(0);
            }

            // network update
            networks.refresh();
            rx_bytes = networks.iter().map(|(_, data)| data.received()).sum();
            tx_bytes = networks.iter().map(|(_, data)| data.transmitted()).sum();


            // dataset calculation
            dataset_data = cpu_history
                .iter()
                .enumerate()
                .map(|(x, &y)| (x as f64, y))
                .collect();

            // disk update
            disks.refresh();
            disk_state.clear();
            for disk in disks.list() {
                let total = disk.total_space();
                let availeble = disk.available_space();

                if total > 0 {
                    let used = total.saturating_sub(availeble);
                    let mount = disk.mount_point().to_string_lossy().to_string();
                    disk_state.push((mount, used, total));
                }
            }

            disk_state.truncate(2);
        }
        

        // draw UI
        terminal.draw(|f| {
            // split terminal in half
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(3), // Summary
                    Constraint::Length(12), // graph
                    Constraint::Length(6), // disk and network
                    Constraint::Min(0), // for OPNsense Logs later
                ])
                .split(f.size());
            let split_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(40), // network left side
                    Constraint::Percentage(60), // disks right side
                ]).split(chunks[2]);
           
            // Summary widget
            let summary_text = format!(" CPU: {:.1}%   |   RAM: {} MB / {} MB", current_cpu, used_memory, total_memory);
            let summary_widget = Paragraph::new(summary_text).block(
                Block::default()
                    .title(" Status Summary ")
                    .borders(Borders::ALL),
            );
            f.render_widget(summary_widget, chunks[0]);

            let datasets = vec![
                Dataset::default()
                    .name(" CPU % ")
                    .marker(symbols::Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(Color::Yellow))
                    .data(&dataset_data)
            ];

            // Graph widget
            let cpu_graph = Chart::new(datasets)
                .block(Block::default().title(" CPU Usage History ").borders(Borders::ALL))
                .x_axis(
                    Axis::default()
                        .title(" Time ")
                        .style(Style::default().fg(Color::Gray))
                        .bounds([0.0, max_history_bars as f64])
                        .labels(vec!["-60s".into(), "-30s".into(), "Now".into()]),
                )
                .y_axis(
                    Axis::default()
                        .title(" Usage ")
                        .style(Style::default().fg(Color::Gray))
                        .bounds([0.0, 100.0])
                        .labels(vec!["0%".into(), "50%".into(), "100%".into()]),
                );
            f.render_widget(cpu_graph, chunks[1]);
            
            // network widget
            let rx_kbs = (rx_bytes as f64 / 1024.0) / args.timeout as f64;
            let tx_kbs = (tx_bytes as f64 / 1024.0) / args.timeout as f64;
            
            let net_text = format!("\n Download (Rx): {rx_kbs:.1} KB/s\n Upload (Tx): {tx_kbs:.1} KB/s");
            let net_widget = Paragraph::new(net_text)
                .block(Block::default().title(" Network I/O ").borders(Borders::ALL));
            f.render_widget(net_widget, split_chunks[0]);

            // Disk gauges
            let disk_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(split_chunks[1]);
            for (i, (mount, used, total)) in disk_state.iter().enumerate() {
                if i >= disk_chunks.len() {break;}
                
                let used_gb = *used as f64 / 1_073_741_824.0;
                let total_gb = *total as f64 / 1_073_741_824.0;

                let percent = ((*used as f64/ *total as f64) * 100.0) as u16;
                let label = format!("{used_gb:.1} GB / {total_gb:.1} GB");

                let gauge = Gauge::default()
                    .block(Block::default().title(format!(" Disk: {mount} ")).borders(Borders::ALL))
                    .gauge_style(Style::default().fg(Color::Cyan))
                    .percent(percent.clamp(0, 100))
                    .label(label);
                f.render_widget(gauge, disk_chunks[i]);
            } 
            
            // chunks[3] is empty for now
            
        })?; // default error
        // polls for keypress
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break; // exits when q is pressed
                }
            }
        }
    } 
    // terminal clean up 
    disable_raw_mode()?; //default error
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?; //default error
                                                             
    Ok(())
}
