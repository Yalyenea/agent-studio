//! Tests for PersistenceService with chunk merging optimization
//!
//! This test suite validates the chunk merging logic and ensures
//! that consecutive text chunks are properly merged before writing to disk.

#[cfg(test)]
mod tests {
    use agent_client_protocol_schema::{ContentBlock, ContentChunk, SessionUpdate};
    use std::path::PathBuf;
    use std::time::Duration;

    use crate::core::services::persistence_service::PersistenceService;

    /// Helper to create a temporary test directory
    fn create_temp_dir() -> PathBuf {
        let temp_dir = std::env::temp_dir().join(format!(
            "agentx_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));
        std::fs::create_dir_all(&temp_dir).unwrap();
        temp_dir
    }

    /// Helper to cleanup temp directory
    fn cleanup_temp_dir(dir: &PathBuf) {
        if dir.exists() {
            std::fs::remove_dir_all(dir).ok();
        }
    }

    #[test]
    fn test_chunk_merging_basic() {
        smol::block_on(async {
            let temp_dir = create_temp_dir();
            let service = PersistenceService::with_buffer_timeout(
                temp_dir.clone(),
                Duration::from_millis(50),
            );

            let session_id = "test-merge-basic";

            // Send 5 consecutive agent message chunks
            for i in 0..5 {
                let chunk = ContentChunk::new(ContentBlock::from(format!("Chunk{} ", i)));
                let update = SessionUpdate::AgentMessageChunk(chunk);
                service.save_update(session_id, update).await.unwrap();
            }

            // Wait for buffer to flush
            smol::Timer::after(Duration::from_millis(100)).await;

            // Load messages
            let messages = service.load_messages(session_id).await.unwrap();

            // Should have 1 merged message instead of 5
            assert_eq!(messages.len(), 1, "Expected 1 merged message");

            // Verify merged content
            if let SessionUpdate::AgentMessageChunk(chunk) = &messages[0].update {
                if let ContentBlock::Text(text) = &chunk.content {
                    assert_eq!(
                        text.text, "Chunk0 Chunk1 Chunk2 Chunk3 Chunk4 ",
                        "Text should be merged"
                    );
                } else {
                    panic!("Expected text content");
                }
            } else {
                panic!("Expected AgentMessageChunk");
            }

            cleanup_temp_dir(&temp_dir);
        });
    }

    #[test]
    fn test_chunk_type_change_flushes_buffer() {
        smol::block_on(async {
            let temp_dir = create_temp_dir();
            let service = PersistenceService::with_buffer_timeout(
                temp_dir.clone(),
                Duration::from_millis(50),
            );

            let session_id = "test-type-change";

            // Send 3 agent chunks
            for i in 0..3 {
                let chunk = ContentChunk::new(ContentBlock::from(format!("Agent{} ", i)));
                let update = SessionUpdate::AgentMessageChunk(chunk);
                service.save_update(session_id, update).await.unwrap();
            }

            // Send a user chunk (different type, should flush agent buffer)
            let user_chunk = ContentChunk::new(ContentBlock::from("User message".to_string()));
            service
                .save_update(session_id, SessionUpdate::UserMessageChunk(user_chunk))
                .await
                .unwrap();

            // Wait for buffers to flush
            smol::Timer::after(Duration::from_millis(100)).await;

            // Load messages
            let messages = service.load_messages(session_id).await.unwrap();

            // Should have 2 messages: 1 merged agent, 1 user
            assert_eq!(messages.len(), 2, "Expected 2 messages");

            // Verify first message (merged agent chunks)
            if let SessionUpdate::AgentMessageChunk(chunk) = &messages[0].update {
                if let ContentBlock::Text(text) = &chunk.content {
                    assert_eq!(text.text, "Agent0 Agent1 Agent2 ");
                }
            }

            // Verify second message (user chunk)
            if let SessionUpdate::UserMessageChunk(chunk) = &messages[1].update {
                if let ContentBlock::Text(text) = &chunk.content {
                    assert_eq!(text.text, "User message");
                }
            }

            cleanup_temp_dir(&temp_dir);
        });
    }

    #[test]
    fn test_non_chunk_message_flushes_buffer() {
        smol::block_on(async {
            let temp_dir = create_temp_dir();
            let service = PersistenceService::with_buffer_timeout(
                temp_dir.clone(),
                Duration::from_millis(50),
            );

            let session_id = "test-non-chunk";

            // Send 3 agent chunks
            for i in 0..3 {
                let chunk = ContentChunk::new(ContentBlock::from(format!("Agent{} ", i)));
                let update = SessionUpdate::AgentMessageChunk(chunk);
                service.save_update(session_id, update).await.unwrap();
            }

            // Send a non-chunk message (should flush buffer immediately)
            service
                .save_update(
                    session_id,
                    SessionUpdate::AgentMessageComplete {
                        is_error: false,
                        error_description: None,
                        meta: None,
                    },
                )
                .await
                .unwrap();

            // No need to wait for timeout - should be flushed immediately

            // Load messages
            let messages = service.load_messages(session_id).await.unwrap();

            // Should have 2 messages: 1 merged agent chunks, 1 complete message
            assert_eq!(messages.len(), 2, "Expected 2 messages");

            cleanup_temp_dir(&temp_dir);
        });
    }

    #[test]
    fn test_explicit_flush_session() {
        smol::block_on(async {
            let temp_dir = create_temp_dir();
            let service = PersistenceService::with_buffer_timeout(
                temp_dir.clone(),
                Duration::from_secs(10), // Long timeout to test explicit flush
            );

            let session_id = "test-explicit-flush";

            // Send 3 chunks
            for i in 0..3 {
                let chunk = ContentChunk::new(ContentBlock::from(format!("Chunk{} ", i)));
                let update = SessionUpdate::AgentMessageChunk(chunk);
                service.save_update(session_id, update).await.unwrap();
            }

            // Explicitly flush without waiting for timeout
            service.flush_session(session_id).await.unwrap();

            // Load messages
            let messages = service.load_messages(session_id).await.unwrap();

            // Should have 1 merged message
            assert_eq!(messages.len(), 1, "Expected 1 merged message");

            if let SessionUpdate::AgentMessageChunk(chunk) = &messages[0].update {
                if let ContentBlock::Text(text) = &chunk.content {
                    assert_eq!(text.text, "Chunk0 Chunk1 Chunk2 ");
                }
            }

            cleanup_temp_dir(&temp_dir);
        });
    }

    #[test]
    fn test_flush_all_multiple_sessions() {
        smol::block_on(async {
            let temp_dir = create_temp_dir();
            let service = PersistenceService::with_buffer_timeout(
                temp_dir.clone(),
                Duration::from_secs(10), // Long timeout to test explicit flush
            );

            // Create 3 sessions with buffered chunks
            for session_num in 0..3 {
                let session_id = format!("test-flush-all-{}", session_num);

                for i in 0..3 {
                    let chunk = ContentChunk::new(ContentBlock::from(format!("Chunk{} ", i)));
                    let update = SessionUpdate::AgentMessageChunk(chunk);
                    service.save_update(&session_id, update).await.unwrap();
                }
            }

            // Flush all sessions at once
            service.flush_all().await.unwrap();

            // Verify all sessions have 1 merged message each
            for session_num in 0..3 {
                let session_id = format!("test-flush-all-{}", session_num);
                let messages = service.load_messages(&session_id).await.unwrap();
                assert_eq!(messages.len(), 1, "Expected 1 merged message per session");
            }

            cleanup_temp_dir(&temp_dir);
        });
    }

    #[test]
    fn test_mixed_message_types() {
        smol::block_on(async {
            let temp_dir = create_temp_dir();
            let service = PersistenceService::with_buffer_timeout(
                temp_dir.clone(),
                Duration::from_millis(50),
            );

            let session_id = "test-mixed-types";

            // Agent chunks
            for i in 0..3 {
                let chunk = ContentChunk::new(ContentBlock::from(format!("Agent{} ", i)));
                service
                    .save_update(session_id, SessionUpdate::AgentMessageChunk(chunk))
                    .await
                    .unwrap();
            }

            // User chunks
            for i in 0..2 {
                let chunk = ContentChunk::new(ContentBlock::from(format!("User{} ", i)));
                service
                    .save_update(session_id, SessionUpdate::UserMessageChunk(chunk))
                    .await
                    .unwrap();
            }

            // Thought chunks
            for i in 0..2 {
                let chunk = ContentChunk::new(ContentBlock::from(format!("Thought{} ", i)));
                service
                    .save_update(session_id, SessionUpdate::AgentThoughtChunk(chunk))
                    .await
                    .unwrap();
            }

            // Wait for buffers to flush
            smol::Timer::after(Duration::from_millis(100)).await;

            // Load messages
            let messages = service.load_messages(session_id).await.unwrap();

            // Should have 3 messages: 1 agent (merged), 1 user (merged), 1 thought (merged)
            assert_eq!(messages.len(), 3, "Expected 3 merged messages");

            cleanup_temp_dir(&temp_dir);
        });
    }

    #[test]
    fn test_rapid_chunk_arrival() {
        smol::block_on(async {
            let temp_dir = create_temp_dir();
            let service = PersistenceService::with_buffer_timeout(
                temp_dir.clone(),
                Duration::from_millis(100),
            );

            let session_id = "test-rapid-arrival";

            // Simulate rapid streaming (100 chunks)
            for i in 0..100 {
                let chunk = ContentChunk::new(ContentBlock::from(format!("{}", i % 10)));
                service
                    .save_update(session_id, SessionUpdate::AgentMessageChunk(chunk))
                    .await
                    .unwrap();
            }

            // Wait for buffer to flush
            smol::Timer::after(Duration::from_millis(150)).await;

            // Load messages
            let messages = service.load_messages(session_id).await.unwrap();

            // Should have significantly fewer messages than 100 (likely 1-5 depending on timing)
            assert!(
                messages.len() < 10,
                "Expected fewer than 10 messages due to merging, got {}",
                messages.len()
            );

            // Verify total content length
            let mut total_content = String::new();
            for msg in &messages {
                if let SessionUpdate::AgentMessageChunk(chunk) = &msg.update {
                    if let ContentBlock::Text(text) = &chunk.content {
                        total_content.push_str(&text.text);
                    }
                }
            }

            // Should contain all 100 digits
            assert_eq!(
                total_content.len(),
                100,
                "Total content length should be 100"
            );

            cleanup_temp_dir(&temp_dir);
        });
    }
}
