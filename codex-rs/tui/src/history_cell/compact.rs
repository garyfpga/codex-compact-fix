//! Context compaction lifecycle history cells.

use super::HistoryCell;
use chrono::TimeZone;
use ratatui::text::Line;
use std::time::Duration;
use std::time::Instant;

#[derive(Debug)]
pub(crate) struct CompactHistoryCell {
    id: String,
    started_at_ms: Option<i64>,
    completed_at_ms: Option<i64>,
    completed: bool,
    local_start: Instant,
}

impl CompactHistoryCell {
    pub(crate) fn new_active(id: String, started_at_ms: Option<i64>) -> Self {
        Self {
            id,
            started_at_ms,
            completed_at_ms: None,
            completed: false,
            local_start: Instant::now(),
        }
    }

    pub(crate) fn new_completed(
        id: String,
        started_at_ms: Option<i64>,
        completed_at_ms: Option<i64>,
    ) -> Self {
        Self {
            id,
            started_at_ms,
            completed_at_ms,
            completed: true,
            local_start: Instant::now(),
        }
    }

    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn complete(&mut self, completed_at_ms: i64) {
        self.completed_at_ms = Some(completed_at_ms);
        self.completed = true;
    }

    fn line_text(&self) -> String {
        if self.completed {
            self.completed_line_text()
        } else {
            self.active_line_text()
        }
    }

    fn active_line_text(&self) -> String {
        let mut parts = vec!["Compacting context".to_string()];
        if let Some(started) = self.started_at_ms.and_then(format_local_time) {
            parts.push(format!("started {started}"));
        }
        parts.push(format!(
            "elapsed {}",
            crate::status_indicator_widget::fmt_elapsed_compact(self.active_elapsed_secs())
        ));
        parts.join(" · ")
    }

    fn completed_line_text(&self) -> String {
        let started = self.started_at_ms.and_then(format_local_time);
        let finished = self.completed_at_ms.and_then(format_local_time);
        let duration = match (&started, &finished) {
            (Some(_), Some(_)) => self.completed_duration_secs(),
            _ => None,
        };

        let mut parts = vec!["Context compacted".to_string()];
        if let Some(started) = started {
            parts.push(format!("started {started}"));
        }
        if let Some(finished) = finished {
            parts.push(format!("finished {finished}"));
        }
        if let Some(duration) = duration {
            parts.push(format!(
                "took {}",
                crate::status_indicator_widget::fmt_elapsed_compact(duration)
            ));
        }
        parts.join(" · ")
    }

    fn active_elapsed_secs(&self) -> u64 {
        if let Some(started_at_ms) = self.started_at_ms
            && timestamp_is_valid(started_at_ms)
        {
            let elapsed_ms = chrono::Local::now()
                .timestamp_millis()
                .saturating_sub(started_at_ms)
                .max(0);
            return Duration::from_millis(elapsed_ms as u64).as_secs();
        }

        self.local_start.elapsed().as_secs()
    }

    fn completed_duration_secs(&self) -> Option<u64> {
        let started_at_ms = self.started_at_ms?;
        let completed_at_ms = self.completed_at_ms?;
        let duration_ms = completed_at_ms.saturating_sub(started_at_ms).max(0);
        Some(Duration::from_millis(duration_ms as u64).as_secs())
    }
}

impl HistoryCell for CompactHistoryCell {
    fn display_lines(&self, _width: u16) -> Vec<Line<'static>> {
        vec![Line::from(self.line_text())]
    }

    fn raw_lines(&self) -> Vec<Line<'static>> {
        vec![Line::from(self.line_text())]
    }

    fn transcript_animation_tick(&self) -> Option<u64> {
        (!self.completed).then(|| self.active_elapsed_secs())
    }
}

fn format_local_time(ms: i64) -> Option<String> {
    if ms <= 0 {
        return None;
    }

    chrono::Local
        .timestamp_millis_opt(ms)
        .single()
        .map(|dt| dt.format("%H:%M:%S").to_string())
}

fn timestamp_is_valid(ms: i64) -> bool {
    ms > 0 && chrono::Local.timestamp_millis_opt(ms).single().is_some()
}

pub(crate) fn new_active_context_compaction(
    id: String,
    started_at_ms: Option<i64>,
) -> CompactHistoryCell {
    CompactHistoryCell::new_active(id, started_at_ms)
}

pub(crate) fn new_completed_context_compaction(
    id: String,
    started_at_ms: Option<i64>,
    completed_at_ms: Option<i64>,
) -> CompactHistoryCell {
    CompactHistoryCell::new_completed(id, started_at_ms, completed_at_ms)
}
