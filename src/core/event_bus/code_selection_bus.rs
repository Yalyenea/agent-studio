use std::sync::{Arc, Mutex};

use crate::app::actions::AddCodeSelection;

/// Event published when code is selected in the editor
#[derive(Clone, Debug)]
pub struct CodeSelectionEvent {
    pub selection: AddCodeSelection,
}

/// Event bus for broadcasting code selection events
pub struct CodeSelectionBus {
    subscribers: Vec<Box<dyn Fn(&CodeSelectionEvent) + Send + Sync>>,
}

impl CodeSelectionBus {
    pub fn new() -> Self {
        Self {
            subscribers: Vec::new(),
        }
    }

    /// Subscribe to code selection events
    pub fn subscribe<F>(&mut self, callback: F)
    where
        F: Fn(&CodeSelectionEvent) + Send + Sync + 'static,
    {
        self.subscribers.push(Box::new(callback));
    }

    /// Publish a code selection event to all subscribers
    pub fn publish(&self, event: CodeSelectionEvent) {
        log::info!(
            "[CodeSelectionBus] Publishing event - file: {}, lines: {}~{}",
            event.selection.file_path,
            event.selection.start_line,
            event.selection.end_line
        );

        for (idx, subscriber) in self.subscribers.iter().enumerate() {
            log::debug!("[CodeSelectionBus] Notifying subscriber {}", idx);
            subscriber(&event);
        }

        log::info!(
            "[CodeSelectionBus] Event published to {} subscribers",
            self.subscribers.len()
        );
    }
}

/// Thread-safe container for CodeSelectionBus
pub type CodeSelectionBusContainer = Arc<Mutex<CodeSelectionBus>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_publish_subscribe() {
        let mut bus = CodeSelectionBus::new();
        let received = Arc::new(Mutex::new(Vec::new()));
        let received_clone = received.clone();

        bus.subscribe(move |event| {
            received_clone
                .lock()
                .unwrap()
                .push(event.selection.file_path.clone());
        });

        bus.publish(CodeSelectionEvent {
            selection: AddCodeSelection {
                file_path: "test.rs".to_string(),
                start_line: 1,
                start_column: 1,
                end_line: 10,
                end_column: 1,
                content: "test content".to_string(),
            },
        });

        assert_eq!(received.lock().unwrap().len(), 1);
    }
}
