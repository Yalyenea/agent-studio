use agent_client_protocol::{ContentBlock, EmbeddedResourceResource, SessionUpdate};
// Helper functions for ConversationPanel

/// Get a unique ElementId from a string identifier
pub fn get_element_id(id: &str) -> gpui::ElementId {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    id.hash(&mut hasher);
    gpui::ElementId::from(("item", hasher.finish()))
}

/// Extract text from ContentBlock for display
pub fn extract_text_from_content(content: &ContentBlock) -> String {
    match content {
        ContentBlock::Text(text_content) => text_content.text.clone(),
        ContentBlock::Image(img) => {
            format!("[Image: {}]", img.mime_type)
        }
        ContentBlock::Audio(audio) => {
            format!("[Audio: {}]", audio.mime_type)
        }
        ContentBlock::ResourceLink(link) => {
            format!("[Resource: {}]", link.name)
        }
        ContentBlock::Resource(resource) => match &resource.resource {
            EmbeddedResourceResource::TextResourceContents(text_res) => {
                format!(
                    "[Resource: {}]\n{}",
                    text_res.uri,
                    &text_res.text[..text_res.text.len().min(200)]
                )
            }
            EmbeddedResourceResource::BlobResourceContents(blob_res) => {
                format!("[Binary Resource: {}]", blob_res.uri)
            }
            _ => "[Unknown Resource]".to_string(),
        },
        _ => "[Unknown Content]".to_string(),
    }
}

/// Get a human-readable type name for SessionUpdate (for logging)
pub fn session_update_type_name(update: &SessionUpdate) -> &'static str {
    match update {
        SessionUpdate::UserMessageChunk(_) => "UserMessageChunk",
        SessionUpdate::AgentMessageChunk(_) => "AgentMessageChunk",
        SessionUpdate::AgentThoughtChunk(_) => "AgentThoughtChunk",
        SessionUpdate::ToolCall(_) => "ToolCall",
        SessionUpdate::ToolCallUpdate(_) => "ToolCallUpdate",
        SessionUpdate::Plan(_) => "Plan",
        SessionUpdate::AvailableCommandsUpdate(_) => "AvailableCommandsUpdate",
        SessionUpdate::CurrentModeUpdate(_) => "CurrentModeUpdate",
        _ => "Unknown/Future SessionUpdate Type",
    }
}
