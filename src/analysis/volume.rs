use std::path::{Path, PathBuf};
use tree_sitter::Node;

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

struct Ctx<'a> {
    code: &'a str,
    name: String,
    params: usize,
    comp: usize,
}

impl VolumeEngine {
    pub fn new() -> Self {
        Self {}
    }

    pub fn analyze_tree(
        &self,
        path: &Path,
        code: &str,
        tree: &tree_sitter::Tree,
    ) -> Vec<FunctionMetric> {
        let mut metrics = Vec::new();
        let mut cursor = tree.root_node().walk();
        while self.visit_node(&mut cursor, path, code, &mut metrics) {}
        metrics
    }

    fn visit_node(
        &self,
        cur: &mut tree_sitter::TreeCursor,
        p: &Path,
        c: &str,
        m: &mut Vec<FunctionMetric>,
    ) -> bool {
        let node = cur.node();
        let is_func = node.kind() == "function_item";
        if is_func {
            m.push(self.extract_metric(node, p, c));
        }
        self.advance_cursor(cur, is_func)
    }

    fn advance_cursor(&self, cursor: &mut tree_sitter::TreeCursor, is_func: bool) -> bool {
        if !is_func && cursor.goto_first_child() {
            return true;
        }
        if cursor.goto_next_sibling() {
            return true;
        }
        self.ascend(cursor)
    }

    fn extract_metric(&self, node: Node, path: &Path, code: &str) -> FunctionMetric {
        let mut ctx = Ctx {
            code,
            name: "unknown".to_string(),
            params: 0,
            comp: 1,
        };
        let mut cursor = node.walk();
        while self.eval_func_node(&mut cursor, node, &mut ctx) {}
        FunctionMetric {
            file_path: path.to_path_buf(),
            function_name: ctx.name,
            lines_of_code: (node.end_position().row - node.start_position().row) + 1,
            parameter_count: ctx.params,
            cyclomatic_complexity: ctx.comp,
        }
    }

    fn eval_func_node(&self, cur: &mut tree_sitter::TreeCursor, root: Node, ctx: &mut Ctx) -> bool {
        self.eval_node(cur.node(), root, ctx);
        if cur.goto_first_child() {
            return true;
        }
        if cur.goto_next_sibling() {
            return true;
        }
        self.ascend_func(cur, root)
    }

    fn eval_node(&self, child: Node, root: Node, ctx: &mut Ctx) {
        let kind = child.kind();
        if kind == "identifier" && child.parent() == Some(root) {
            ctx.name = child
                .utf8_text(ctx.code.as_bytes())
                .unwrap_or("unknown")
                .to_string();
        } else if kind == "parameter" || kind == "self_parameter" {
            ctx.params += 1;
        } else if self.is_branch(kind) {
            ctx.comp += 1;
        } else if kind == "binary_expression" {
            let txt = child.utf8_text(ctx.code.as_bytes()).unwrap_or("");
            if txt.contains("&&") || txt.contains("||") {
                ctx.comp += 1;
            }
        }
    }

    fn is_branch(&self, kind: &str) -> bool {
        matches!(
            kind,
            "if_expression"
                | "while_expression"
                | "for_expression"
                | "loop_expression"
                | "match_arm"
        )
    }

    fn ascend(&self, cursor: &mut tree_sitter::TreeCursor) -> bool {
        loop {
            if !cursor.goto_parent() {
                return false;
            }
            if cursor.goto_next_sibling() {
                return true;
            }
        }
    }

    fn ascend_func(&self, cursor: &mut tree_sitter::TreeCursor, root: Node) -> bool {
        loop {
            if !cursor.goto_parent() || cursor.node() == root {
                return false;
            }
            if cursor.goto_next_sibling() {
                return true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_extraction() {
        let code = r#"
        fn example_func(a: i32, b: i32) {
            if a > 0 && b > 0 {
                println!("test");
            }
        }
        "#;

        let engine = VolumeEngine::new();
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .unwrap();
        let tree = parser.parse(code, None).unwrap();

        let metrics = engine.analyze_tree(Path::new("dummy.rs"), code, &tree);
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].function_name, "example_func");
        assert_eq!(metrics[0].parameter_count, 2);
        // Base complexity 1 + 1 (if) + 1 (&&) = 3
        assert_eq!(metrics[0].cyclomatic_complexity, 3);
        assert_eq!(metrics[0].lines_of_code, 5);
    }
}
