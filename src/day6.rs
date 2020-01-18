use std::error::Error;
use std::path::PathBuf;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::collections::{HashMap, VecDeque};

pub fn run(filename: &PathBuf, part2: &bool) -> Result<(), Box<dyn Error>> {
    let vec = read(File::open(filename)?)?;
    let pairs = extract_pairs(vec)?;
    let tree = build_tree(pairs)?;

    if *part2 {
        if let Some(x) = count_transfers(tree) {
            println!("Orbital jumps: {}", x);
        }
    } else {
        println!("Orbits: {}", validate(tree));
    }
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

    fn find_node(&self, val: T) -> Option<usize> {
        for node in &self.arena {
            if node.val == val {
                return Some(node.idx.clone());
            }
        }

        None
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

fn count_transfers(tree: ArenaTree<String>) -> Option<i32> {
    let you_path = path_to_com(&tree, String::from("YOU"));
    let santa_path = path_to_com(&tree, String::from("SAN"));

    if let Some(closest_shared_parent) = find_mutual_key_with_smallest_value(&you_path, &santa_path) {
        Some(you_path.get(&closest_shared_parent).unwrap() + santa_path.get(&closest_shared_parent).unwrap())
    } else {
        None
    }
}

fn path_to_com(tree: &ArenaTree<String>, start: String) -> HashMap<String, i32> {
    let mut distance = 0;
    let mut path = HashMap::new();
    let mut node_idx_opt = (*tree).find_node(start);
    
    while let Some(node_idx) = node_idx_opt {
        let node = &(tree.arena[node_idx]);
        if let Some(parent_idx) = node.parent {
            let parent_node = &(tree.arena[parent_idx]);
            path.insert(parent_node.val.clone(), distance);
            distance = distance + 1;
            node_idx_opt = Some(parent_idx);
        } else {
            node_idx_opt = None
        }
    }

    path
}

fn find_mutual_key_with_smallest_value(map1: &HashMap<String, i32>, map2: &HashMap<String, i32>) -> Option<String> {
    let mut best_key = None;
    let mut best_value = i32::max_value();

    for (key, val) in (*map1).iter() {
        if (*map2).contains_key(key) && *val < best_value {
            best_key = Some(key.clone());
            best_value = *val;
        }
    }

    best_key
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

    #[test]
    fn test_path_simple() {
        let pairs = extract_pairs(vec![String::from("COM)YOU")]).unwrap();
        let tree = build_tree(pairs).unwrap();
        let path = path_to_com(&tree, String::from("YOU"));
        assert_eq!(0, *path.get(&String::from("COM")).unwrap());
    }

    #[test]
    fn test_path_two_steps() {
        let pairs = extract_pairs(vec![String::from("COM)INT"),
                                       String::from("INT)YOU")]).unwrap();
        let tree = build_tree(pairs).unwrap();
        let path = path_to_com(&tree, String::from("YOU"));
        assert_eq!(0, *path.get(&String::from("INT")).unwrap());
        assert_eq!(1, *path.get(&String::from("COM")).unwrap());
    }

    #[test]
    fn test_common_key_with_smallest_value() {
        let map1: HashMap<String, i32> = [
            (String::from("A"), 1),
            (String::from("B"), 2),
            (String::from("C"), 3)].iter().cloned().collect();
        let map2: HashMap<String, i32> = [
            (String::from("A"), 4),
            (String::from("C"), 5),
            (String::from("D"), 6)].iter().cloned().collect();
        assert_eq!(String::from("A"), find_mutual_key_with_smallest_value(&map1, &map2).unwrap());
    }

    #[test]
    fn test_count_transfers() {
        let input = vec![
            "COM)B",
            "B)C",
            "C)D",
            "D)E",
            "E)F",
            "B)G",
            "G)H",
            "D)I",
            "E)J",
            "J)K",
            "K)L",
            "K)YOU",
            "I)SAN"
        ].iter().map(|&x| String::from(x)).collect();
        let pairs = extract_pairs(input).unwrap();
        let tree = build_tree(pairs).unwrap();
        assert_eq!(4, count_transfers(tree).unwrap());
    }
}
