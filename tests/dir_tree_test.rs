use std::fs::{self, File};
use teamprojekt_agents::agent_actions::dir_tree::DirTreeTool;

#[test]
fn test_dir_tree_tool_basic_structure() {
    // Creates a temporary directory with files & subdirectories
    let tmp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let root = tmp_dir.path();

    // Creates structure: root/alpha/beta.txt and root/gamma.txt
    let alpha_dir = root.join("alpha");
    fs::create_dir(&alpha_dir).expect("failed to create alpha dir");
    File::create(alpha_dir.join("beta.txt")).expect("failed to create beta.txt");
    File::create(root.join("gamma.txt")).expect("failed to create gamma.txt");

    // Runs tool
    let entry = DirTreeTool::read_dir_tree(root).expect("read_dir_tree_inner failed");

    assert!(entry.is_dir);
    assert!(entry.children.is_some());

    // Flatten children for easier checking
    let mut names: Vec<_> = entry
        .children
        .as_ref()
        .unwrap()
        .iter()
        .map(|c| &c.name)
        .collect();
    names.sort();
    assert_eq!(names, &["alpha", "gamma.txt"]);

    // Check subdirectory structure
    let alpha_entry = entry
        .children
        .as_ref()
        .unwrap()
        .iter()
        .find(|c| c.name == "alpha")
        .unwrap();
    assert!(alpha_entry.is_dir);
    let beta_names: Vec<_> = alpha_entry
        .children
        .as_ref()
        .unwrap()
        .iter()
        .map(|c| &c.name)
        .collect();
    assert_eq!(beta_names, &["beta.txt"]);
}
