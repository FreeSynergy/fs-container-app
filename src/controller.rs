// controller.rs — ContainerAppController: MVC controller.
//
// Knows only the ContainerEngine trait — never QuadletManager directly.

use std::sync::{Arc, Mutex};

use fs_container::{ContainerEngine, ServiceStatus};

use crate::model::{ContainerAppModel, ContainerEntry};

// ── ContainerAppController ────────────────────────────────────────────────────

/// Shared controller — cheaply cloneable (Arc-backed).
pub struct ContainerAppController<E: ContainerEngine> {
    engine: Arc<E>,
    state: Arc<Mutex<ContainerAppModel>>,
}

impl<E: ContainerEngine + Send + Sync + 'static> ContainerAppController<E> {
    #[must_use]
    pub fn new(engine: E) -> Self {
        Self {
            engine: Arc::new(engine),
            state: Arc::new(Mutex::new(ContainerAppModel::new())),
        }
    }

    /// Snapshot of the current model.
    #[must_use]
    pub fn snapshot(&self) -> ContainerAppModel {
        self.state.lock().unwrap().clone()
    }

    /// Reload the container list from the engine.
    pub fn refresh(&self) {
        let services = self.engine.list().unwrap_or_default();
        let containers = services
            .into_iter()
            .map(|s| ContainerEntry {
                name: s.name.clone(),
                state_label: format!("{:?}", s.state),
            })
            .collect();
        let mut guard = self.state.lock().unwrap();
        guard.containers = containers;
        guard.loading = false;
    }

    /// Start a container service by name.
    pub fn start(&self, name: &str) -> Result<(), String> {
        self.engine.start(name).map_err(|e| e.to_string())
    }

    /// Stop a container service by name.
    pub fn stop(&self, name: &str) -> Result<(), String> {
        self.engine.stop(name).map_err(|e| e.to_string())
    }

    /// Retrieve log lines for a container.
    pub fn logs(&self, name: &str, lines: usize) -> Vec<String> {
        self.engine.logs(name, lines).unwrap_or_default()
    }
}

impl<E: ContainerEngine + Send + Sync + 'static> Clone for ContainerAppController<E> {
    fn clone(&self) -> Self {
        Self {
            engine: Arc::clone(&self.engine),
            state: Arc::clone(&self.state),
        }
    }
}

// ── StubEngine for tests ──────────────────────────────────────────────────────

#[cfg(test)]
pub(crate) struct StubEngine;

#[cfg(test)]
impl ContainerEngine for StubEngine {
    type Error = String;

    fn list(&self) -> Result<Vec<ServiceStatus>, Self::Error> {
        Ok(vec![ServiceStatus {
            name: "test-svc".into(),
            state: fs_container::systemctl::UnitActiveState::Active,
            description: String::new(),
        }])
    }
    fn start(&self, _name: &str) -> Result<(), Self::Error> {
        Ok(())
    }
    fn stop(&self, _name: &str) -> Result<(), Self::Error> {
        Ok(())
    }
    fn logs(&self, _name: &str, _lines: usize) -> Result<Vec<String>, Self::Error> {
        Ok(vec!["log line 1".into()])
    }
    fn deploy(&self, _config: &fs_container::ServiceConfig) -> Result<(), Self::Error> {
        Ok(())
    }
    fn remove(&self, _name: &str) -> Result<(), Self::Error> {
        Ok(())
    }
    fn status(&self, _name: &str) -> Result<Option<ServiceStatus>, Self::Error> {
        Ok(None)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn ctrl() -> ContainerAppController<StubEngine> {
        ContainerAppController::new(StubEngine)
    }

    #[test]
    fn refresh_populates_list() {
        let ctrl = ctrl();
        ctrl.refresh();
        assert_eq!(ctrl.snapshot().containers.len(), 1);
    }

    #[test]
    fn start_returns_ok() {
        assert!(ctrl().start("test-svc").is_ok());
    }

    #[test]
    fn stop_returns_ok() {
        assert!(ctrl().stop("test-svc").is_ok());
    }

    #[test]
    fn logs_returns_lines() {
        let lines = ctrl().logs("test-svc", 10);
        assert!(!lines.is_empty());
    }

    #[test]
    fn snapshot_after_refresh_has_containers() {
        let ctrl = ctrl();
        ctrl.refresh();
        let snap = ctrl.snapshot();
        assert!(!snap.containers.is_empty());
        assert_eq!(snap.containers[0].name, "test-svc");
    }
}
