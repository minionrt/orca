use crate::openai;
use crate::tools_interface::ToolInstance;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct DirEntry {
    pub name: String,
    pub is_dir: bool,
    pub children: Option<Vec<DirEntry>>,
}

pub struct DirTreeTool;

impl ToolInstance for DirTreeTool {
    type Args = ();
    type Out = DirEntry;

    /// Runs the directory tree tool
    /// # Parameters
    /// * `params`: serde_json::Value with optional "path" String (default: ".")
    /// # Return
    /// Returns a JSON representation of the directory tree or an error.
    fn run(_: ()) -> Result<Self::Out, Box<dyn Error>> {
        Ok(Self::read_dir_tree()?)
    }

    /// Returns the tool definition for this tool, including parameters and descriptions
    fn return_choice() -> openai::Tool {
        openai::Tool::function(
            "dir_tree".to_owned(),
            "Recursively returns the directory structure as JSON.".to_owned(),
            HashMap::new(),
            HashMap::new(),
        )
    }
}

impl DirTreeTool {
    pub fn new() -> Self {
        DirTreeTool
    }

    pub fn read_dir_tree() -> std::io::Result<DirEntry>{
        Self::read_dir_tree_inner(Path::new("/workspace"))
    }

    /// Recursively reads a directory and builds the tree structure.
    pub fn read_dir_tree_inner<P: AsRef<Path>>(path: P) -> std::io::Result<DirEntry> {
        let path = path.as_ref();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string());
        if path.is_dir() {
            let children = fs::read_dir(path)?
                .filter_map(|e| e.ok())
                .map(|e| DirTreeTool::read_dir_tree_inner(e.path()))
                .filter_map(Result::ok)
                .collect();
            Ok(DirEntry {
                name,
                is_dir: true,
                children: Some(children),
            })
        } else {
            Ok(DirEntry {
                name,
                is_dir: false,
                children: None,
            })
        }
    }
}

// Implements Default trait for DirTreeTool
impl Default for DirTreeTool {
    fn default() -> Self {
        DirTreeTool::new()
    }
}
