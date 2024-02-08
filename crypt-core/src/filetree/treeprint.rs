use super::tree::*;

pub fn print_tree(root: &str, dir: &Directory) {
    const OTHER_CHILD: &str = "│   "; // prefix: pipe
    const OTHER_ENTRY: &str = "├── "; // connector: tee
    const FINAL_CHILD: &str = "    "; // prefix: no siblings
    const FINAL_ENTRY: &str = "└── "; // connector: elbow

    println!("{}", root);
    let (d, f) = visit(dir, "");
    println!("\n{} directories, {} files", d, f);

    fn visit(node: &Directory, prefix: &str) -> (usize, usize) {
        let mut dirs: usize = 1; // counting this directory
        let mut files: usize = 0;
        let mut count = node.entries.len();
        for entry in &node.entries {
            count -= 1;
            let connector = if count == 0 { FINAL_ENTRY } else { OTHER_ENTRY };
            match entry {
                FileTree::DirNode(sub_dir) => {
                    println!("{}{}{}", prefix, connector, sub_dir.name);
                    let new_prefix = format!(
                        "{}{}",
                        prefix,
                        if count == 0 { FINAL_CHILD } else { OTHER_CHILD }
                    );
                    let (d, f) = visit(sub_dir, &new_prefix);
                    dirs += d;
                    files += f;
                }
                FileTree::LinkNode(symlink) => {
                    println!(
                        "{}{}{} -> {}",
                        prefix, connector, symlink.name, symlink.target
                    );
                    files += 1;
                }
                FileTree::FileNode(file) => {
                    println!("{}{}{}", prefix, connector, file.name);
                    files += 1;
                }
            }
        }
        (dirs, files)
    }
}
