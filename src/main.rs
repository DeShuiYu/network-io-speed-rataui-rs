
use std::time::{Duration, Instant};
use color_eyre::Result;
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols::{self},
    text::{ Span},
    widgets::{Axis, Block, Chart, Dataset},
    DefaultTerminal, Frame,
};

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = App::new().run(terminal);
    ratatui::restore();
    app_result
}

struct App {
    data1: Vec<(f64, f64)>,
    data2: Vec<(f64, f64)>,
    window: [f64; 2],
    network: sysinfo::Networks,
}
impl App {
    fn new() -> Self {
        let network = sysinfo::Networks::new_with_refreshed_list();
        let data1 = (0..60).map(|x| (x as f64, 0f64)).collect::<Vec< _>>();
        let data2 = (0..60).map(|x| (x as f64, 0f64)).collect::<Vec< _>>();
        Self {
            data1,
            data2,
            window: [0.0, 60.0],
            network,
        }
    }

    fn get_download_and_upload_speed(&mut self)-> (f64, f64){
        self.network.refresh(true);
        for (interface, stats) in self.network.iter() {
            if !interface.to_lowercase().starts_with("en0"){
                continue;
            }
           return (stats.received() as f64 / 1024f64 / 1024f64, stats.transmitted() as f64 / 1024f64 / 1024f64);
        }
        (0.0, 0.0)
    }

    fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        let tick_rate = Duration::from_secs(1);
        let mut last_tick = Instant::now();
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if key.code == KeyCode::Char('q') {
                        return Ok(());
                    }
                }
            }
            if last_tick.elapsed() >= tick_rate {
                self.on_tick();
                last_tick = Instant::now();
            }
        }
    }

    fn on_tick(&mut self) {
        let (download_speed, upload_speed) = self.get_download_and_upload_speed();
        self.window[0] += 1.0;
        self.window[1] += 1.0;
        self.data1.drain(0..1);
        self.data1.push((self.window[1],download_speed));
        self.data2.drain(0..1);
        self.data2.push((self.window[1],upload_speed));

    }

    fn draw(&self, frame: &mut Frame) {
        let [animated_chart] = Layout::vertical([Constraint::Fill(1); 1]).areas(frame.area());
        self.render_animated_chart(frame, animated_chart);
    }

    fn render_animated_chart(&self, frame: &mut Frame, area: Rect) {
        let x_labels = vec![
            Span::styled(
                format!("{}", self.window[0]),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("{}", (self.window[0] + self.window[1]) / 2.0)),
            Span::styled(
                format!("{}", self.window[1]),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ];
        let datasets = vec![
            Dataset::default()
                .name("DSpeed")
                .marker(symbols::Marker::Dot)
                .graph_type(ratatui::widgets::GraphType::Line)
                .style(Style::default().fg(Color::Cyan))
                .data(&self.data1),
            Dataset::default()
                .name("USpeed")
                .marker(symbols::Marker::Braille)
                .graph_type(ratatui::widgets::GraphType::Line)
                .style(Style::default().fg(Color::Yellow))
                .data(&self.data2),
        ];

        let chart = Chart::new(datasets)
            .block(Block::bordered())
            .x_axis(
                Axis::default()
                    .title("X Axis")
                    .style(Style::default().fg(Color::Gray))
                    .labels(x_labels)
                    .bounds(self.window),
            )
            .y_axis(
                Axis::default()
                    .title("Y Axis")
                    .style(Style::default().fg(Color::Gray))
                    .labels(["0M/s".bold(), "5M/s".into(), "10M/s".bold()])
                    .bounds([0.0, 10.0]),
            );

        frame.render_widget(chart, area);
    }
}




