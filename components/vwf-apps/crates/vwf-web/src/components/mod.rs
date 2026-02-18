//! UI components for VWF web interface.

mod run_status_viewer;
mod service_panel;
mod var_editor;
mod workdir_input;
mod workflow_editor;

pub use run_status_viewer::RunStatusViewer;
pub use service_panel::ServicePanel;
pub use var_editor::VarEditor;
pub use workdir_input::WorkdirInput;
pub use workflow_editor::WorkflowEditor;
