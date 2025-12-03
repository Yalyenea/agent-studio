use std::cell::RefCell;
use std::rc::Rc;

use chrono::{DateTime, Local, Utc};
use gpui::{Hsla, SharedString, WeakEntity};
use gpui_component::list::ListState;
use gpui_component::IndexPath;

use crate::schemas::workspace_schema::WorkspaceTask;

/// Date-based section for task grouping
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateSection {
    Today = 0,
    Yesterday = 1,
    Last30Days = 2,
}

impl DateSection {
    /// Get the label for this section
    pub fn label(&self) -> &'static str {
        match self {
            DateSection::Today => "今天",
            DateSection::Yesterday => "昨天",
            DateSection::Last30Days => "过去30天",
        }
    }

    /// Get all sections in order
    pub fn all() -> [DateSection; 3] {
        [DateSection::Today, DateSection::Yesterday, DateSection::Last30Days]
    }

    /// Convert section index to DateSection
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(DateSection::Today),
            1 => Some(DateSection::Yesterday),
            2 => Some(DateSection::Last30Days),
            _ => None,
        }
    }
}

/// Categorize a timestamp into a DateSection
pub fn categorize_by_date(timestamp: DateTime<Utc>) -> DateSection {
    let now = Local::now();
    let local_timestamp = timestamp.with_timezone(&Local);

    let today = now.date_naive();
    let task_date = local_timestamp.date_naive();

    if task_date == today {
        DateSection::Today
    } else if task_date == today - chrono::Duration::days(1) {
        DateSection::Yesterday
    } else {
        DateSection::Last30Days
    }
}

/// Generate a consistent avatar color from a string
pub fn avatar_color(name: &str) -> Hsla {
    let hash = name.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32));
    let hue = (hash % 360) as f32 / 360.0;
    Hsla {
        h: hue,
        s: 0.6,
        l: 0.5,
        a: 1.0,
    }
}

/// Get the first character of a name for avatar display
pub fn avatar_letter(name: &str) -> String {
    name.chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_else(|| "?".to_string())
}

/// Task panel delegate for ListDelegate implementation
pub struct TaskPanelDelegate {
    /// All tasks from all workspaces
    pub all_tasks: Vec<Rc<WorkspaceTask>>,
    /// Tasks grouped by date section (filtered)
    pub section_tasks: [Vec<Rc<WorkspaceTask>>; 3],
    /// Current search query
    pub query: String,
    /// Selected index
    pub selected_index: Option<IndexPath>,
    /// Confirmed index (for double-click)
    pub confirmed_index: Option<IndexPath>,
    /// Collapsed sections
    pub collapsed_sections: Rc<RefCell<std::collections::HashSet<usize>>>,
    /// Weak reference to list state for notifications
    pub list_state: Option<WeakEntity<ListState<Self>>>,
    /// Loading state
    pub loading: bool,
    /// End of data flag
    pub eof: bool,
}

impl TaskPanelDelegate {
    pub fn new() -> Self {
        Self {
            all_tasks: Vec::new(),
            section_tasks: [Vec::new(), Vec::new(), Vec::new()],
            query: String::new(),
            selected_index: None,
            confirmed_index: None,
            collapsed_sections: Rc::new(RefCell::new(std::collections::HashSet::new())),
            list_state: None,
            loading: false,
            eof: true,
        }
    }

    /// Load tasks from workspace service
    pub fn load_tasks(&mut self, tasks: Vec<WorkspaceTask>) {
        self.all_tasks = tasks.into_iter().map(Rc::new).collect();
        self.categorize_tasks();
    }

    /// Categorize tasks by date section
    fn categorize_tasks(&mut self) {
        // Clear existing categorization
        for section in &mut self.section_tasks {
            section.clear();
        }

        // Filter by query if present
        let query_lower = self.query.to_lowercase();
        let filtered_tasks: Vec<_> = if query_lower.is_empty() {
            self.all_tasks.iter().cloned().collect()
        } else {
            self.all_tasks
                .iter()
                .filter(|task| {
                    task.name.to_lowercase().contains(&query_lower)
                        || task
                            .last_message
                            .as_ref()
                            .map(|m| m.to_lowercase().contains(&query_lower))
                            .unwrap_or(false)
                })
                .cloned()
                .collect()
        };

        // Sort by created_at descending (most recent first)
        // TODO: When WorkspaceTask has a last_updated field, use that instead
        let mut sorted_tasks = filtered_tasks;
        sorted_tasks.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        // Categorize into sections
        for task in sorted_tasks {
            let section = categorize_by_date(task.created_at);
            self.section_tasks[section as usize].push(task);
        }
    }

    /// Perform search/filter
    pub fn prepare(&mut self, query: String) {
        self.query = query;
        self.categorize_tasks();
    }

    /// Check if a section is collapsed
    pub fn is_section_collapsed(&self, section: usize) -> bool {
        self.collapsed_sections.borrow().contains(&section)
    }

    /// Get selected task
    pub fn selected_task(&self) -> Option<Rc<WorkspaceTask>> {
        let ix = self.selected_index?;
        self.section_tasks
            .get(ix.section)
            .and_then(|tasks| tasks.get(ix.row))
            .cloned()
    }

    /// Update task message by session ID
    pub fn update_task_message(&mut self, session_id: &str, message: String) -> bool {
        for task in &mut self.all_tasks {
            if task.session_id.as_ref() == Some(&session_id.to_string()) {
                // Need to get mutable access - use Rc::make_mut or similar
                // For now, we'll need to find and update in place
                // This is a limitation of using Rc<WorkspaceTask>
                if let Some(task) = Rc::get_mut(task) {
                    task.last_message = Some(SharedString::from(message));
                    self.categorize_tasks();
                    return true;
                }
            }
        }
        false
    }

    /// Get section task count (for display)
    pub fn section_count(&self, section: usize) -> usize {
        self.section_tasks
            .get(section)
            .map(|tasks| tasks.len())
            .unwrap_or(0)
    }

    /// Get total task count
    pub fn total_count(&self) -> usize {
        self.section_tasks.iter().map(|s| s.len()).sum()
    }
}
