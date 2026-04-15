use tui::prelude::*;
use tui::symbols::scrollbar::DOUBLE_VERTICAL;
use tui::widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState};

#[derive(Clone, Copy)]
pub struct ScrollParams {
    pub scroll_offset: i32,
    pub visible_top: i32,
    pub visible_bottom: i32,
}

/// Adjust area based on scroll position and clip to visible viewport.
/// Returns (clipped_area, rows_clipped_from_top).
pub fn adjust_area_for_scroll(area: Rect, params: ScrollParams) -> Option<(Rect, u16)> {
    let area_top = area.y as i32 - params.scroll_offset + params.visible_top;
    let area_bottom = area_top + area.height as i32;

    if area_bottom <= params.visible_top || area_top >= params.visible_bottom {
        return None;
    }

    let clipped_top = area_top.max(params.visible_top);
    let clipped_bottom = area_bottom.min(params.visible_bottom);
    let clipped_height = clipped_bottom - clipped_top;
    let rows_clipped = (clipped_top - area_top) as u16;

    if clipped_height > 0 {
        Some((
            Rect {
                x: area.x,
                y: clipped_top as u16,
                width: area.width,
                height: clipped_height as u16,
            },
            rows_clipped,
        ))
    } else {
        None
    }
}

pub fn render_scrollbar(
    area: Rect,
    scroll_state: &mut ScrollbarState,
    symbols: &crate::symbols::Symbols,
    buf: &mut Buffer,
) {
    let scrollbar_area = Rect {
        x: area.x + area.width + 1, // +1 to render over the border
        y: area.y,
        width: 1,
        height: area.height,
    };
    StatefulWidget::render(
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .thumb_symbol(DOUBLE_VERTICAL.track)
            .track_symbol(Some(DOUBLE_VERTICAL.track))
            .begin_symbol(Some(symbols.scroll_up()))
            .end_symbol(Some(symbols.scroll_down())),
        scrollbar_area,
        buf,
        scroll_state,
    );
}
