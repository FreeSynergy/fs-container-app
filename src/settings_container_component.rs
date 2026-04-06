// settings_container_component.rs — SettingsContainerComponent: Pod-YAML container config.
//
// Design Pattern: MVC (existing pattern in fs-container-app)
//   This component renders the container configuration for a running service.
//   Actions that require a container restart show an explicit restart-required badge.
//   Permission check: only users with the "container.admin" right may reconfigure.
//
// Data source: ContainerEngine trait (via gRPC stub in this file).
// Writes: Bus-Events ("container.reconfigure" topic) — triggers restart.

use fs_render::component::{ButtonStyle, ComponentCtx, ComponentTrait, LayoutElement, TextSize};
use fs_render::layout::SlotKind;

// ── RestartPolicy ─────────────────────────────────────────────────────────────

/// Whether an action requires a container restart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RestartPolicy {
    /// Change applied live, no restart needed.
    Live,
    /// Container must be restarted for the change to take effect.
    Required,
}

impl RestartPolicy {
    fn badge(self) -> Option<LayoutElement> {
        match self {
            Self::Live => None,
            Self::Required => Some(LayoutElement::Badge {
                content: "settings-container-restart-required".into(),
                color: None,
            }),
        }
    }
}

// ── Action row helpers ────────────────────────────────────────────────────────

fn action_row(label_key: &str, action: &str, restart: RestartPolicy) -> LayoutElement {
    let mut children = vec![LayoutElement::Button {
        label_key: label_key.into(),
        action: action.into(),
        style: ButtonStyle::Ghost,
    }];
    if let Some(badge) = restart.badge() {
        children.push(badge);
    }
    LayoutElement::Row { children, gap: 8 }
}

// ── SettingsContainerComponent ────────────────────────────────────────────────

/// Shows Pod-YAML configuration for a container-managed service.
///
/// # Wiring (ComponentCtx.config)
///
/// | key | value |
/// |-----|-------|
/// | `"instance_id"` | Container instance id (e.g. `"kanidm-main"`) |
/// | `"instance_name"` | Display name (e.g. `"Main IAM"`) |
/// | `"has_permission"` | `"true"` if the current user may reconfigure |
/// | `"status"` | `"running"` \| `"stopped"` \| `"error"` |
///
/// When `has_permission` is not `"true"`, all action buttons are replaced with
/// a permission-denied notice.
pub struct SettingsContainerComponent {
    id: &'static str,
}

impl SettingsContainerComponent {
    /// Create a new settings container component.
    pub fn new() -> Self {
        Self {
            id: "settings-container",
        }
    }

    fn render_no_permission() -> LayoutElement {
        LayoutElement::Text {
            content: "settings-container-no-permission".into(),
            size: TextSize::Body,
            color: None,
        }
    }

    fn render_actions(instance_id: &str, status: &str) -> Vec<LayoutElement> {
        let is_running = status == "running";

        vec![
            // Instance management
            LayoutElement::Separator {
                label_key: Some("settings-container-section-instance".into()),
            },
            action_row(
                "settings-container-action-start",
                &format!("container.start:{instance_id}"),
                RestartPolicy::Live,
            ),
            action_row(
                "settings-container-action-stop",
                &format!("container.stop:{instance_id}"),
                RestartPolicy::Live,
            ),
            action_row(
                "settings-container-action-restart",
                &format!("container.restart:{instance_id}"),
                RestartPolicy::Live,
            ),
            // Pod-YAML reconfiguration (requires restart)
            LayoutElement::Separator {
                label_key: Some("settings-container-section-config".into()),
            },
            action_row(
                "settings-container-action-edit-yaml",
                &format!("container.edit_yaml:{instance_id}"),
                RestartPolicy::Required,
            ),
            action_row(
                "settings-container-action-copy-instance",
                &format!("container.copy_instance:{instance_id}"),
                RestartPolicy::Live,
            ),
            // Danger zone
            LayoutElement::Separator {
                label_key: Some("settings-container-section-danger".into()),
            },
            LayoutElement::Button {
                label_key: "settings-container-action-delete".into(),
                action: format!("container.delete:{instance_id}"),
                style: ButtonStyle::Danger,
            },
            // Status badge
            if is_running {
                LayoutElement::Badge {
                    content: "settings-container-status-running".into(),
                    color: None,
                }
            } else {
                LayoutElement::Badge {
                    content: "settings-container-status-stopped".into(),
                    color: None,
                }
            },
        ]
    }
}

impl Default for SettingsContainerComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl ComponentTrait for SettingsContainerComponent {
    fn component_id(&self) -> &str {
        self.id
    }

    fn name_key(&self) -> &'static str {
        "component-settings-container-name"
    }

    fn description_key(&self) -> &'static str {
        "component-settings-container-desc"
    }

    fn slot_preference(&self) -> SlotKind {
        SlotKind::Fill
    }

    fn min_width(&self) -> u32 {
        240
    }

    fn render(&self, ctx: &ComponentCtx) -> Vec<LayoutElement> {
        let instance_id = ctx.config.get("instance_id").cloned().unwrap_or_default();
        let instance_name = ctx
            .config
            .get("instance_name")
            .cloned()
            .unwrap_or_else(|| instance_id.clone());
        let has_permission = ctx.config.get("has_permission").map(String::as_str) == Some("true");
        let status = ctx.config.get("status").map_or("stopped", String::as_str);

        let mut elements = vec![
            LayoutElement::Text {
                content: "component-settings-container-name".into(),
                size: TextSize::Label,
                color: None,
            },
            LayoutElement::Row {
                children: vec![
                    LayoutElement::Icon {
                        name: "fs:apps/container".into(),
                        size: 20,
                    },
                    LayoutElement::Text {
                        content: instance_name,
                        size: TextSize::Subheading,
                        color: None,
                    },
                ],
                gap: 8,
            },
            LayoutElement::Separator { label_key: None },
        ];

        if has_permission {
            elements.extend(Self::render_actions(&instance_id, status));
        } else {
            elements.push(Self::render_no_permission());
        }

        elements
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use fs_render::layout::{ShellKind, SlotKind};

    fn ctx_with(instance_id: &str, has_permission: bool, status: &str) -> ComponentCtx {
        let mut ctx = ComponentCtx::test(ShellKind::Main, SlotKind::Fill);
        ctx.config.insert("instance_id".into(), instance_id.into());
        ctx.config
            .insert("instance_name".into(), "Test Instance".into());
        ctx.config.insert(
            "has_permission".into(),
            if has_permission { "true" } else { "false" }.into(),
        );
        ctx.config.insert("status".into(), status.into());
        ctx
    }

    #[test]
    fn component_id() {
        let c = SettingsContainerComponent::new();
        assert_eq!(c.component_id(), "settings-container");
    }

    #[test]
    fn slot_preference_is_fill() {
        let c = SettingsContainerComponent::new();
        assert_eq!(c.slot_preference(), SlotKind::Fill);
    }

    #[test]
    fn render_without_permission_shows_notice() {
        let c = SettingsContainerComponent::new();
        let ctx = ctx_with("kanidm-main", false, "running");
        let els = c.render(&ctx);
        let has_notice = els.iter().any(|e| {
            matches!(e, LayoutElement::Text { content, .. }
                if content.contains("no-permission"))
        });
        assert!(has_notice);
    }

    #[test]
    fn render_with_permission_shows_actions() {
        let c = SettingsContainerComponent::new();
        let ctx = ctx_with("kanidm-main", true, "running");
        let els = c.render(&ctx);
        let has_button = els
            .iter()
            .any(|e| matches!(e, LayoutElement::Button { .. }));
        assert!(has_button);
    }

    #[test]
    fn render_with_permission_shows_delete_danger() {
        let c = SettingsContainerComponent::new();
        let ctx = ctx_with("kanidm-main", true, "stopped");
        let els = c.render(&ctx);
        let has_danger = els.iter().any(|e| {
            matches!(e, LayoutElement::Button { style, .. }
                if *style == ButtonStyle::Danger)
        });
        assert!(has_danger);
    }

    #[test]
    fn restart_policy_badge_for_required() {
        let badge = RestartPolicy::Required.badge();
        assert!(badge.is_some());
    }

    #[test]
    fn restart_policy_no_badge_for_live() {
        let badge = RestartPolicy::Live.badge();
        assert!(badge.is_none());
    }
}
