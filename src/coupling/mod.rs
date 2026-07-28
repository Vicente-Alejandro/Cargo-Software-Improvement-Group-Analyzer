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
    let cp = p.split_whitespace().next().unwrap_or(p).trim_matches(|c| c == '{' || c == '}' || c == ';');
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
}
