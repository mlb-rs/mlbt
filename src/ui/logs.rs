use tui::prelude::Color;
use tui::widgets::Block;
use tui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};
use tui_logger::{TuiLoggerLevelOutput, TuiLoggerWidget};

pub struct LogWidget {}

impl Widget for LogWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        TuiLoggerWidget::default()
            .block(Block::bordered().title("logs"))
            .style_error(Style::default().fg(Color::Red))
            .style_warn(Style::default().fg(Color::Yellow))
            .style_info(Style::default().fg(Color::Cyan))
            .style_debug(Style::default().fg(Color::Green))
            .style_trace(Style::default().fg(Color::Magenta))
            .output_separator('|')
            .output_timestamp(Some("%F %H:%M:%S%.3f".to_string()))
            .output_level(Some(TuiLoggerLevelOutput::Long))
            .output_target(false)
            .output_file(false)
            .output_line(false)
            .style(Style::default().fg(Color::White))
            .render(area, buf);
    }
}
