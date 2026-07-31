use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Default, Debug)]
pub struct CouplingGraph {
    pub edges: HashMap<PathBuf, HashSet<PathBuf>>,
    pub ignored_externals: usize,
}

impl CouplingGraph {
    #[rustfmt::skip]
    pub fn build(dir: &Path, files: &[PathBuf]) -> Self {
        let mod_map: HashMap<String, PathBuf> = files.iter().filter_map(|f| file_to_mod_name(dir, f).map(|n| (n, f.clone()))).collect();
        let mut ignored_externals = 0;
        let edges = files.iter().map(|f| {
            let (deps, ig) = parse_file_deps(f, &mod_map);
            ignored_externals += ig;
            (f.clone(), deps)
        }).collect();
        Self { edges, ignored_externals }
    }

    #[rustfmt::skip]
    pub fn fan_out(&self, file: &Path) -> usize {
        self.edges.get(file).map(|d| d.len()).unwrap_or(0)
    }

    #[rustfmt::skip]
    pub fn detect_cycles(&self) -> Vec<Vec<PathBuf>> {
        let (mut cycles, mut visited, mut stack) = (Vec::new(), HashSet::new(), Vec::new());
        for node in self.edges.keys() {
            if !visited.contains(node) {
                let mut ctx = DfsCtx { visited: &mut visited, stack: &mut stack, cycles: &mut cycles };
                self.dfs(node, &mut ctx);
            }
        }
        cycles
    }

    #[rustfmt::skip]
    fn dfs(&self, node: &PathBuf, ctx: &mut DfsCtx) {
        ctx.visited.insert(node.clone());
        ctx.stack.push(node.clone());
        if let Some(neighbors) = self.edges.get(node) {
            for n in neighbors {
                if let Some(idx) = ctx.stack.iter().position(|x| x == n) {
                    ctx.cycles.push(ctx.stack[idx..].to_vec());
                } else if !ctx.visited.contains(n) {
                    self.dfs(n, ctx);
                }
            }
        }
        ctx.stack.pop();
    }
}

struct DfsCtx<'a> {
    visited: &'a mut HashSet<PathBuf>,
    stack: &'a mut Vec<PathBuf>,
    cycles: &'a mut Vec<Vec<PathBuf>>,
}

#[rustfmt::skip]
fn parse_file_deps(f: &Path, m: &HashMap<String, PathBuf>) -> (HashSet<PathBuf>, usize) {
    let (mut deps, mut ig) = (HashSet::new(), 0);
    if let Ok(c) = std::fs::read_to_string(f) {
        for l in c.lines().filter_map(|l| l.trim().strip_prefix("use ")) {
            if let Some((d, i)) = parse_use(l, f, m) {
                if let Some(p) = d { deps.insert(p); }
                ig += i;
            }
        }
    }
    (deps, ig)
}

#[rustfmt::skip]
fn parse_use(p: &str, f: &Path, m: &HashMap<String, PathBuf>) -> Option<(Option<PathBuf>, usize)> {
    let cp = p.split_whitespace().next().unwrap_or(p).trim_matches(&['{', '}', ';'][..]);
    if let Some(rem) = cp.strip_prefix("crate::") {
        let mod_name = rem.split("::").next().unwrap_or(rem);
        let t = m.get(mod_name)?;
        return if t != f { Some((Some(t.clone()), 0)) } else { None };
    }
    if cp.starts_with("super::") { return None; }
    if cp.starts_with("self::") { return None; }
    Some((None, 1))
}

#[rustfmt::skip]
fn file_to_mod_name(base: &Path, file: &Path) -> Option<String> {
    let rel = file.strip_prefix(base).ok()?;
    let mut comps = rel.components();
    comps.next(); // skip "src"
    let path = comps.as_path();
    let name = path.file_stem()?.to_string_lossy().to_string();
    if name == "mod" { Some(path.parent()?.file_name()?.to_string_lossy().to_string()) }
    else if name == "main" || name == "lib" { Some("crate".to_string()) }
    else { Some(name) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cycle_detection() {
        let p1 = PathBuf::from("a");
        let p2 = PathBuf::from("b");
        let p3 = PathBuf::from("c");

        let mut edges = HashMap::new();
        edges.insert(p1.clone(), HashSet::from([p2.clone()]));
        edges.insert(p2.clone(), HashSet::from([p3.clone()]));
        edges.insert(p3.clone(), HashSet::from([p1.clone()]));

        let graph = CouplingGraph {
            edges,
            ignored_externals: 0,
        };
        let cycles = graph.detect_cycles();
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].len(), 3);
    }

    #[test]
    fn test_file_to_mod_name() {
        let base = PathBuf::from("my_project");
        let f1 = base.join("src").join("main.rs");
        assert_eq!(file_to_mod_name(&base, &f1), Some("crate".to_string()));

        let f2 = base.join("src").join("analysis").join("mod.rs");
        assert_eq!(file_to_mod_name(&base, &f2), Some("analysis".to_string()));

        let f3 = base.join("src").join("cli.rs");
        assert_eq!(file_to_mod_name(&base, &f3), Some("cli".to_string()));
    }

    #[test]
    fn test_parse_use() {
        let mut m = HashMap::new();
        m.insert("analysis".to_string(), PathBuf::from("src/analysis/mod.rs"));
        m.insert("crate".to_string(), PathBuf::from("src/main.rs"));
        let f = PathBuf::from("src/main.rs");

        // Internal dep
        let (dep, ig) = parse_use("crate::analysis::run;", &f, &m).unwrap();
        assert_eq!(dep, Some(PathBuf::from("src/analysis/mod.rs")));
        assert_eq!(ig, 0);

        // Self referential
        let res = parse_use("crate::main::something", &f, &m);
        assert!(res.is_none());

        // External dep
        let (dep, ig) = parse_use("std::collections::HashMap;", &f, &m).unwrap();
        assert!(dep.is_none());
        assert_eq!(ig, 1);

        // super or self
        assert!(parse_use("super::foo", &f, &m).is_none());
        assert!(parse_use("self::foo", &f, &m).is_none());
    }

    #[test]
    fn test_build_and_fan_out() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src");
        std::fs::create_dir(&src).unwrap();
        
        let a_rs = src.join("a.rs");
        std::fs::write(&a_rs, "use crate::b;\nuse std::fs;").unwrap();
        
        let b_rs = src.join("b.rs");
        std::fs::write(&b_rs, "use crate::a;").unwrap();
        
        let files = vec![a_rs.clone(), b_rs.clone()];
        let graph = CouplingGraph::build(dir.path(), &files);
        
        assert_eq!(graph.ignored_externals, 1);
        assert_eq!(graph.fan_out(&a_rs), 1);
        assert_eq!(graph.fan_out(&b_rs), 1);
        assert_eq!(graph.fan_out(&PathBuf::from("nonexistent")), 0);
        
        let cycles = graph.detect_cycles();
        assert_eq!(cycles.len(), 1);
    }
}
