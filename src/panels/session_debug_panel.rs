//! Session Debug Panel - Inspect and debug agent session states
//!
//! This panel provides real-time visibility into:
//! - Active sessions and their metadata
//! - Session lifecycle events
//! - ACP protocol communication
//! - Session errors and warnings

use gpui::*;
use gpui_component::{
    button::Button,
    divider::Divider,
    h_flex, v_flex,
    label::Label,
    theme::ActiveTheme,
    Sizable,
};

use crate::{
    app::app_state::AppState,
    core::services::{AgentSessionInfo, SessionStatus},
    panels::dock_panel::DockPanel,
};

use chrono::Local;

pub struct SessionDebugPanel {
    focus_handle: FocusHandle,
    sessions: Vec<SessionInfoDisplay>,
}

#[derive(Clone, Debug)]
struct SessionInfoDisplay {
    session_id: String,
    agent_name: String,
    created_at: String,
    last_active: String,
    idle_duration: String,
    status: SessionStatus,
}

impl SessionDebugPanel {
    pub fn new(_window: &mut Window, cx: &mut App) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            sessions: Vec::new(),
        }
    }

    fn refresh_sessions(&mut self, cx: &mut Context<Self>) {
        let agent_service = match AppState::global(cx).agent_service() {
            Some(service) => service,
            None => {
                log::warn!("AgentService not available for session refresh");
                return;
            }
        };

        let all_sessions = agent_service.list_sessions();
        let now = chrono::Utc::now();

        self.sessions = all_sessions
            .into_iter()
            .map(|info: AgentSessionInfo| {
                let idle_duration = now.signed_duration_since(info.last_active);
                SessionInfoDisplay {
                    session_id: info.session_id.clone(),
                    agent_name: info.agent_name.clone(),
                    created_at: info.created_at.with_timezone(&Local).format("%H:%M:%S").to_string(),
                    last_active: info.last_active.with_timezone(&Local).format("%H:%M:%S").to_string(),
                    idle_duration: format_duration(idle_duration.num_seconds()),
                    status: info.status,
                }
            })
            .collect();

        cx.notify();
    }

    fn test_session(&mut self, session_id: String, cx: &mut Context<Self>) {
        let agent_service = match AppState::global(cx).agent_service() {
            Some(service) => service.clone(),
            None => {
                log::error!("AgentService not available");
                return;
            }
        };

        let session_id_clone = session_id.clone();
        cx.spawn(|_this, mut _cx| async move {
            // Try to find the session
            match agent_service.get_session_by_id(&session_id_clone) {
                Some(info) => {
                    log::info!(
                        "✅ Session {} found - Agent: {}, Status: {:?}, Last active: {}",
                        session_id_clone,
                        info.agent_name,
                        info.status,
                        info.last_active
                    );

                    // Try to send a test prompt
                    let test_result = agent_service
                        .send_prompt(&info.agent_name, &session_id_clone, vec!["ping".to_string()])
                        .await;

                    match test_result {
                        Ok(_) => {
                            log::info!("✅ Test prompt sent successfully to session {}", session_id_clone);
                        }
                        Err(e) => {
                            log::error!("❌ Failed to send test prompt to session {}: {}", session_id_clone, e);
                        }
                    }
                }
                None => {
                    log::error!("❌ Session {} not found in AgentService", session_id_clone);
                }
            }
            Ok::<(), anyhow::Error>(())
        })
        .detach();
    }

    fn create_test_session(&mut self, agent_name: String, cx: &mut Context<Self>) {
        let agent_service = match AppState::global(cx).agent_service() {
            Some(service) => service.clone(),
            None => {
                log::error!("AgentService not available");
                return;
            }
        };

        cx.spawn(|this, mut cx| async move {
            match agent_service.create_session(&agent_name).await {
                Ok(session_id) => {
                    log::info!("✅ Created test session {} for agent {}", session_id, agent_name);

                    // Refresh the panel
                    cx.update(|cx| {
                        this.update(cx, |this, cx| {
                            this.refresh_sessions(cx);
                        })
                    }).ok();
                }
                Err(e) => {
                    log::error!("❌ Failed to create test session for agent {}: {}", agent_name, e);
                }
            }
            Ok::<(), anyhow::Error>(())
        })
        .detach();
    }
}

impl DockPanel for SessionDebugPanel {
    fn title() -> &'static str {
        "Session Debugger"
    }

    fn description() -> &'static str {
        "Debug agent session states and lifecycle"
    }

    fn new_view(_window: &mut Window, cx: &mut App) -> Entity<impl Render> {
        cx.new(|cx| Self::new(_window, cx))
    }
}

impl Focusable for SessionDebugPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for SessionDebugPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let sessions_count = self.sessions.len();

        // Auto-refresh on render
        self.refresh_sessions(cx);

        v_flex()
            .size_full()
            .gap_3()
            .p_4()
            .bg(theme.background)
            .child(
                // Header
                v_flex()
                    .gap_2()
                    .child(
                        h_flex()
                            .items_center()
                            .justify_between()
                            .child(
                                Label::new(format!("Session Debugger ({} sessions)", sessions_count))
                                    .text_color(theme.foreground),
                            )
                            .child(
                                Button::new("refresh")
                                    .label("Refresh")
                                    .small()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.refresh_sessions(cx);
                                    })),
                            ),
                    )
                    .child(Divider::horizontal()),
            )
            .child(
                // Controls
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        Label::new("Test actions:")
                            .text_color(theme.muted_foreground),
                    )
                    .child(
                        Button::new("create-test-iflow")
                            .label("Create Iflow Session")
                            .small()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.create_test_session("Iflow".to_string(), cx);
                            })),
                    ),
            )
            .child(Divider::horizontal())
            .child(
                // Session list
                v_flex()
                    .flex_1()
                    .gap_2()
                    .children(if self.sessions.is_empty() {
                        vec![
                            div()
                                .p_4()
                                .child(
                                    Label::new("No active sessions")
                                        .text_color(theme.muted_foreground),
                                ).into_any_element()
                        ]
                    } else {
                        self.sessions.iter().map(|session| {
                            render_session_card(session.clone(), &theme, cx).into_any_element()
                        }).collect()
                    }),
            )
    }
}

fn render_session_card(session: SessionInfoDisplay, theme: &gpui_component::theme::Theme, cx: &mut Context<SessionDebugPanel>) -> Div {
    let session_id_clone = session.session_id.clone();

    let status_color = match session.status {
        SessionStatus::Active => theme.success,
        SessionStatus::Idle => theme.warning,
        SessionStatus::Closed => theme.muted_foreground,
    };

    let status_text = format!("{:?}", session.status);

    v_flex()
        .gap_2()
        .p_3()
        .bg(theme.muted)
        .border_1()
        .border_color(theme.border)
        .rounded_md()
        .child(
            // Session header
            h_flex()
                .items_center()
                .justify_between()
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(
                            div()
                                .w_2()
                                .h_2()
                                .rounded_full()
                                .bg(status_color),
                        )
                        .child(
                            Label::new(format!("Session: {}", &session.session_id[..8]))
                                .text_color(theme.foreground),
                        )
                        .child(
                            Label::new(status_text)
                                .text_color(status_color),
                        ),
                )
                .child(
                    Button::new(ElementId::Name(session.session_id.clone().into()))
                        .label("Test")
                        .xsmall()
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.test_session(session_id_clone.clone(), cx);
                        })),
                ),
        )
        .child(
            // Session details
            v_flex()
                .gap_1()
                .child(
                    h_flex()
                        .gap_2()
                        .child(
                            Label::new("Agent:")
                                .text_color(theme.muted_foreground),
                        )
                        .child(
                            Label::new(&session.agent_name)
                                .text_color(theme.foreground),
                        ),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .child(
                            Label::new("Created:")
                                .text_color(theme.muted_foreground),
                        )
                        .child(
                            Label::new(&session.created_at)
                                .text_color(theme.foreground),
                        ),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .child(
                            Label::new("Last Active:")
                                .text_color(theme.muted_foreground),
                        )
                        .child(
                            Label::new(&session.last_active)
                                .text_color(theme.foreground),
                        )
                        .child(
                            Label::new(format!("({})", session.idle_duration))
                                .text_color(theme.muted_foreground),
                        ),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .child(
                            Label::new("Session ID:")
                                .text_color(theme.muted_foreground),
                        )
                        .child(
                            Label::new(&session.session_id)
                                .text_color(theme.muted_foreground),
                        ),
                ),
        )
}

fn format_duration(seconds: i64) -> String {
    if seconds < 60 {
        format!("{}s ago", seconds)
    } else if seconds < 3600 {
        format!("{}m ago", seconds / 60)
    } else {
        format!("{}h ago", seconds / 3600)
    }
}
