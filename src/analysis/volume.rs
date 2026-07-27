use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FunctionMetric {
    pub file_path: PathBuf,
    pub function_name: String,
    pub lines_of_code: usize,
    pub parameter_count: usize,
}

pub struct VolumeEngine {
    // We will construct this as a deep module.
    // In later phases, this will use rayon to parallelize over files.
}

impl VolumeEngine {
    pub fn new() -> Self {
        Self {}
    }

    pub fn analyze_file(
        &self,
        path: &std::path::Path,
        source_code: &str,
    ) -> anyhow::Result<Vec<FunctionMetric>> {
        let mut parser = tree_sitter::Parser::new();
        // The tree-sitter v0.26.x and tree-sitter-rust v0.24.2 syntax
        parser.set_language(&tree_sitter_rust::LANGUAGE.into())?;

        let tree = parser
            .parse(source_code, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse file: {:?}", path))?;

        let mut metrics = Vec::new();
        let root_node = tree.root_node();

        // Let's use a tree cursor to find all function_item nodes.
        let mut cursor = root_node.walk();
        let mut needs_visit = true;

        while needs_visit {
            let node = cursor.node();

            if node.kind() == "function_item" {
                let start_row = node.start_position().row;
                let end_row = node.end_position().row;
                let loc = (end_row - start_row) + 1; // Basic LOC heuristic

                let mut func_name = "unknown".to_string();
                let mut param_count = 0;

                for child in node.children(&mut tree.walk()) {
                    if child.kind() == "identifier" {
                        func_name = child
                            .utf8_text(source_code.as_bytes())
                            .unwrap_or("unknown")
                            .to_string();
                    }
                    if child.kind() == "parameters" {
                        // Count named parameters. In tree-sitter-rust, `parameters` contains things like `(`, `)`, `,` and `parameter`.
                        // A rough parameter count is counting actual `parameter` nodes, or taking named children count if available.
                        let mut param_cursor = child.walk();
                        let mut has_next = param_cursor.goto_first_child();
                        while has_next {
                            let p_node = param_cursor.node();
                            // In rust grammar, the actual parameters are called "parameter", "self_parameter", etc.
                            if p_node.kind() == "parameter" || p_node.kind() == "self_parameter" {
                                param_count += 1;
                            }
                            has_next = param_cursor.goto_next_sibling();
                        }
                    }
                }

                metrics.push(FunctionMetric {
                    file_path: path.to_path_buf(),
                    function_name: func_name,
                    lines_of_code: loc,
                    parameter_count: param_count,
                });
            }

            // Standard preorder traversal
            if cursor.goto_first_child() {
                continue;
            }
            if cursor.goto_next_sibling() {
                continue;
            }

            // Ascend
            loop {
                if !cursor.goto_parent() {
                    needs_visit = false;
                    break;
                }
                if cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        Ok(metrics)
    }
}
