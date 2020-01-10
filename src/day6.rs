use std::error::Error;
use std::path::PathBuf;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::collections::{HashMap, VecDeque};

pub fn run(filename: &PathBuf, part2: &bool) -> Result<(), Box<dyn Error>> {
    let vec = read(File::open(filename)?)?;
    let pairs = extract_pairs(vec)?;
    let tree = build_tree(pairs)?;
    println!("Orbits: {}", validate(tree));
    Ok(()) 
}

fn read<R: Read>(io: R) -> Result<Vec<String>, std::io::Error> {
    let br = BufReader::new(io);
    br.lines().collect()
}

fn extract_pairs(entries: Vec<String>) -> Result<VecDeque<(String, String)>, String> {
    entries.iter().fold(Ok(VecDeque::new()), |acc, string| {
        let tokens: Vec<&str> = string.split(")").collect();
        acc.and_then(|mut vec| {
            match (tokens.first(), tokens.last()) {
                (Some(&a), Some(&b)) => vec.push_back((String::from(a), String::from(b))),
                _ => return Err(String::from("Warning: failed to add string line to set of pairs."))
            };
            Ok(vec)
        })
    })
}

fn build_tree(pairs: VecDeque<(String, String)>) -> Result<ArenaTree<String>, String> {
    let mut tree = ArenaTree::new();
    let mut known_nodes = HashMap::new();
    known_nodes.insert(String::from("COM"), tree.node(String::from("COM")));
    let mut remaining = pairs.clone();
    while remaining.len() > 0 {
        match remaining.pop_front() {
            Some((parent, child)) => {
                if known_nodes.contains_key(&parent) {
                    let p_idx = tree.node(parent);
                    let c_idx = tree.node(child.clone());
                    tree.arena[p_idx].children.push(c_idx);
                    tree.arena[c_idx].parent = Some(p_idx);
                    known_nodes.insert(child.clone(), c_idx);
                } else {
                    remaining.push_back((parent, child))
                }
            },
            None => return Err(String::from("Unexpectedly empty queue."))
        }
    }
    Ok(tree)
}

#[derive(Debug, Default)]
struct ArenaTree<T>
where
    T: PartialEq
{
    arena: Vec<Node<T>>,
}

#[derive(Debug)]
struct Node<T>
where
    T: PartialEq
{
    idx: usize,
    val: T,
    parent: Option<usize>,
    children: Vec<usize>,
}

impl<T> Node<T>
where
    T: PartialEq
{
    fn new(idx: usize, val: T) -> Self {
        Self {
            idx,
            val,
            parent: None,
            children: vec![]
        }
    }
}

impl<T> ArenaTree<T>
where
    T: PartialEq
{
    fn new() -> Self {
        Self { arena: vec![] }
    }

    fn node(&mut self, val: T) -> usize {
        for node in &self.arena {
            if node.val == val {
                return node.idx;
            }
        }

        let idx = self.arena.len();
        self.arena.push(Node::new(idx, val));
        idx
    }
}

fn validate(tree: ArenaTree<String>) -> i32 {
    let mut known_nodes = HashMap::new();

    // Assuming that the tree nodes are in inner-to-outer order.
    tree.arena.iter().fold(0, |acc, node| {
        match node.parent {
            Some(p_idx) => {
                let dist = known_nodes[&p_idx] + 1;
                known_nodes.insert(node.idx, dist);
                acc + dist
            },
            None => {
                known_nodes.insert(node.idx, 0);
                acc
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_simple() {
        let pairs = extract_pairs(vec![String::from("COM)A")]).unwrap();
        let tree = build_tree(pairs).unwrap();
        assert_eq!(1, validate(tree));
    }

    #[test]
    fn test_validate_two_planets() {
        let pairs = extract_pairs(vec![String::from("COM)A"), 
                                       String::from("COM)B")]).unwrap();
        let tree = build_tree(pairs).unwrap();
        assert_eq!(2, validate(tree));
    }

    #[test]
    fn test_validate_two_planets_cascading() {
        let pairs = extract_pairs(vec![String::from("A)B"), String::from("COM)A")]).unwrap();
        let tree = build_tree(pairs).unwrap();
        assert_eq!(3, validate(tree));
    }
}
