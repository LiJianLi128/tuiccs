//! Per-frame mouse hit-testing store.
//!
//! The render pass records the exact rectangles it draws interactive elements
//! into, and [`crate::cli::tui::app::App::on_mouse`] reads them back in the same
//! event-loop iteration to translate a terminal mouse coordinate into a UI
//! element. Everything is cleared at the start of each [`super::render`].

use ratatui::layout::Rect;

use crate::app_config::AppType;

use crate::cli::tui::form::FormFocus;

/// A rectangular hit region, tested in terminal (column, row) coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct HitRect {
    pub rect: Rect,
}

impl HitRect {
    pub(crate) fn contains(self, column: u16, row: u16) -> bool {
        column >= self.rect.x
            && column < self.rect.x.saturating_add(self.rect.width)
            && row >= self.rect.y
            && row < self.rect.y.saturating_add(self.rect.height)
    }
}

/// A scrollable list region with enough geometry to map a clicked row back to
/// the absolute row index ratatui rendered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ListHitRegion {
    pub list_rect: Rect,
    /// Number of header rows above the first data row (0 for a table with no
    /// `.header(...)`, 1 otherwise).
    pub header_rows: u16,
    /// Height of each data row in terminal rows (1 for single-line tables, >1
    /// where rows wrap).
    pub row_height: u16,
    /// The list's scroll window offset (`TableState::offset()` / page start)
    /// captured after the widget was rendered.
    pub scroll_offset: u16,
}

impl ListHitRegion {
    /// Maps a terminal row to the absolute list index under the cursor, or
    /// `None` when the row falls outside the list body (borders, header, or
    /// below the last visible row).
    pub(crate) fn row_at(self, row: u16) -> Option<usize> {
        let first_data_row = self.list_rect.y.saturating_add(self.header_rows);
        let row_height = self.row_height.max(1);
        let visible = row.checked_sub(first_data_row)?;
        if visible % row_height != 0 {
            return None;
        }
        let local = visible / row_height;
        let local_max = self.list_rect.height.saturating_sub(self.header_rows) / row_height;
        if local >= local_max {
            return None;
        }
        Some(local as usize + self.scroll_offset as usize)
    }
}

/// Everything the current frame recorded for mouse hit-testing.
///
/// `RefCell` is used because the store lives on [`super::super::app::App`] and
/// is written during `render(&App)` and read during `on_mouse(&mut App)` in the
/// same loop iteration.
#[derive(Clone, Debug, Default)]
pub(crate) struct MouseLayout {
    /// The focused content list (providers, MCP, prompts, config, ...).
    pub content_list: Option<ListHitRegion>,
    /// The active overlay's list (pickers), when one is open.
    pub overlay_list: Option<ListHitRegion>,
    /// Clickable overlay buttons, in display order (key-bar row for confirms;
    /// bottom rows for action dialogs). Each is paired with the key label the
    /// button triggers (e.g. "Enter", "Esc", "c").
    pub overlay_buttons: Vec<(HitRect, String)>,
    /// Header app-type tab chips, each paired with the app type it switches to.
    pub header_chips: Vec<(HitRect, AppType)>,
    /// Editable form fields, each paired with the focus it activates.
    pub form_fields: Vec<(HitRect, FormFocus)>,
    /// The filter input bar, when visible.
    pub filter_bar: Option<Rect>,
}

impl MouseLayout {
    /// Clears every recorded region. Called once at the start of each render.
    pub(crate) fn reset(&mut self) {
        self.content_list = None;
        self.overlay_list = None;
        self.overlay_buttons.clear();
        self.header_chips.clear();
        self.form_fields.clear();
        self.filter_bar = None;
    }
}
