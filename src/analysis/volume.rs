use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FunctionMetric {
    #[allow(dead_code)]
    pub file_path: PathBuf,
    #[allow(dead_code)]
    pub function_name: String,
    pub lines_of_code: usize,
    pub parameter_count: usize,
    pub cyclomatic_complexity: usize,
}

pub struct VolumeEngine {}

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

                // Base complexity is 1 (the function itself)
                let mut complexity = 1;

                // We need to walk the entire subtree of the function to count parameters and complexity
                let mut func_cursor = node.walk();
                let mut func_needs_visit = true;
                
                while func_needs_visit {
                    let child = func_cursor.node();
                    let kind = child.kind();
                    
                    // Identify function name
                    if kind == "identifier" && child.parent().map(|p| p.kind()) == Some("function_item") {
                        func_name = child
                            .utf8_text(source_code.as_bytes())
                            .unwrap_or("unknown")
                            .to_string();
                    }
                    
                    // Identify parameters
                    if kind == "parameter" || kind == "self_parameter" {
                        param_count += 1;
                    }
                    
                    // Cyclomatic complexity branches
                    match kind {
                        "if_expression" | "while_expression" | "for_expression" | "loop_expression" | "match_arm" => {
                            complexity += 1;
                        }
                        "binary_expression" => {
                            // Check if operator is && or ||
                            // In tree-sitter-rust, the operator is a child of the binary_expression
                            // But for simplicity and zero-bloat, we just count binary expressions that contain && or || text
                            if let Ok(text) = child.utf8_text(source_code.as_bytes()) {
                                if text.contains("&&") || text.contains("||") {
                                    // This is a naive heuristic that works reasonably well for complexity estimation
                                    // without doing deep operator introspection
                                    complexity += 1;
                                }
                            }
                        }
                        _ => {}
                    }

                    // Preorder traversal within the function subtree
                    if func_cursor.goto_first_child() {
                        continue;
                    }
                    if func_cursor.goto_next_sibling() {
                        continue;
                    }
                    
                    // Ascend
                    loop {
                        if !func_cursor.goto_parent() {
                            func_needs_visit = false;
                            break;
                        }
                        // Stop if we've ascended back to the function_item itself
                        if func_cursor.node() == node {
                            func_needs_visit = false;
                            break;
                        }
                        if func_cursor.goto_next_sibling() {
                            break;
                        }
                    }
                }

                metrics.push(FunctionMetric {
                    file_path: path.to_path_buf(),
                    function_name: func_name,
                    lines_of_code: loc,
                    parameter_count: param_count,
                    cyclomatic_complexity: complexity,
                });
            }

            // Standard preorder traversal (skipping internals of function_item since we handled them)
            if node.kind() != "function_item" && cursor.goto_first_child() {
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
