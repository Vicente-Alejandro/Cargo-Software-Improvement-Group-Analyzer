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

#[derive(Default)]
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
        while self.visit_node(&mut cursor, (path, code), &mut metrics) {}
        metrics
    }

    fn visit_node(
        &self,
        cur: &mut tree_sitter::TreeCursor,
        file: (&Path, &str),
        m: &mut Vec<FunctionMetric>,
    ) -> bool {
        let is_func = cur.node().kind() == "function_item";
        if is_func {
            self.process_func(cur.node(), file, m);
        }
        self.advance_cursor(cur, is_func)
    }

    fn process_func(&self, n: Node, file: (&Path, &str), m: &mut Vec<FunctionMetric>) {
        let met = self.extract_metric(n, file.0, file.1);
        if !met.function_name.starts_with("test_") {
            m.push(met);
        }
    }

    fn advance_cursor(&self, cursor: &mut tree_sitter::TreeCursor, is_func: bool) -> bool {
        if !is_func && cursor.goto_first_child() {
            return true;
        }
        if cursor.goto_next_sibling() {
            return true;
        }
        self.ascend(cursor, None)
    }

    fn extract_metric(&self, node: Node, path: &Path, code: &str) -> FunctionMetric {
        let mut ctx = Ctx {
            code,
            name: "unknown".to_string(),
            params: 0,
            comp: 1,
        };
        let mut cur = node.walk();
        while self.eval_func_node(&mut cur, node, &mut ctx) {}
        self.build_metric(node, path, &ctx)
    }

    fn build_metric(&self, node: Node, path: &Path, ctx: &Ctx) -> FunctionMetric {
        let loc = (node.end_position().row - node.start_position().row) + 1;
        FunctionMetric {
            file_path: path.to_path_buf(),
            function_name: ctx.name.clone(),
            lines_of_code: loc,
            parameter_count: ctx.params,
            cyclomatic_complexity: ctx.comp,
        }
    }

    fn eval_func_node(&self, cur: &mut tree_sitter::TreeCursor, root: Node, ctx: &mut Ctx) -> bool {
        self.eval_node(cur.node(), root, ctx);
        if cur.goto_first_child() || cur.goto_next_sibling() {
            return true;
        }
        self.ascend(cur, Some(root))
    }

    fn eval_node(&self, child: Node, root: Node, ctx: &mut Ctx) {
        let kind = child.kind();
        if self.is_branch(kind) {
            ctx.comp += 1;
            return;
        }
        self.eval_non_branch((child, root), ctx, kind);
    }

    fn eval_non_branch(&self, nodes: (Node, Node), ctx: &mut Ctx, k: &str) {
        if self.is_param(k) {
            ctx.params += 1;
            return;
        }
        if self.is_id(nodes.0, nodes.1, k) {
            ctx.name = nodes
                .0
                .utf8_text(ctx.code.as_bytes())
                .unwrap_or("unknown")
                .to_string();
            return;
        }
        self.check_binary(nodes.0, ctx);
    }

    fn is_param(&self, k: &str) -> bool {
        k == "parameter" || k == "self_parameter"
    }

    fn is_id(&self, child: Node, root: Node, k: &str) -> bool {
        k == "identifier" && child.parent() == Some(root)
    }

    fn check_binary(&self, child: Node, ctx: &mut Ctx) {
        if child.kind() == "binary_expression" {
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

    fn ascend(&self, cursor: &mut tree_sitter::TreeCursor, root: Option<Node>) -> bool {
        loop {
            if !cursor.goto_parent() || root.is_some_and(|r| cursor.node() == r) {
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

        let mut metrics = Vec::new();
        let mut cursor = tree.root_node().walk();
        while engine.visit_node(&mut cursor, (Path::new("dummy.rs"), code), &mut metrics) {}

        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].function_name, "example_func");
        assert_eq!(metrics[0].parameter_count, 2);
        assert_eq!(metrics[0].cyclomatic_complexity, 3);
        assert_eq!(metrics[0].lines_of_code, 5);
    }
}
