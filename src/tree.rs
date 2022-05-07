use std::{usize, collections::VecDeque, ops::Index, fmt::Display, fmt::Debug};

#[derive(Debug)]
pub struct Node<T> {
    pub val: T,
    pub children: Vec<usize>,
    pub level: usize,
    idx: usize
}

#[derive(Debug)]
pub struct NodePosition {
    pub depth: usize,
    pub siblings: usize,
    pub raw_idx: usize,
    pub parent_raw_idx: usize
}

pub struct Tree<T>
where T: PartialEq {
    nodes: Vec<Node<T>>
}

pub struct TreeIter<'a,T> {
    dfs: bool,
    idx_queue: VecDeque<usize>,
    arena: Vec<&'a Node<T>>
}

impl<T> Display for Tree<T> where T: PartialEq + Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result: String = self
            .nodes_dfs()
            .fold("".to_string(),
                |mut acc: String,node|
                {
                    acc =
                        acc +
                        &format!("{}{}{}",
                            " ".repeat(node.level),
                            if node.level > 0 {"└"} else {"─"},
                            if node.children.len() > 0 {"┬"} else {"─"}
                        ).to_string() +
                        &format!("─ {:?}\n",node.val).to_string();
                    acc
                }
            );
        write!(f,"{}",result)
    }
}

impl<'a,T> Iterator for TreeIter<'a,T> {
    type Item = &'a T;
    fn next(& mut self) -> Option<Self::Item> {
        if let Some(n_i) = self.idx_queue.pop_front() {
            if n_i > self.arena.len() - 1 { return None }
            let found = &self.arena[n_i];
            if self.dfs {
                for leaf_idx in found.children.iter().rev() {
                    self.idx_queue.push_front(*leaf_idx)
                }
            } else {
                self.idx_queue.extend(&found.children);
            }
            return Some(&self.arena[n_i].val);
        }
        return None;
    }
}

pub struct TreeNodeIter<'a,T> {
    dfs: bool,
    idx_queue: VecDeque<usize>,
    arena: Vec<&'a Node<T>>
}

impl<'a,T> Iterator for TreeNodeIter<'a,T> {
    type Item = &'a Node<T>;
    fn next(& mut self) -> Option<Self::Item> {
        if let Some(n_i) = self.idx_queue.pop_front() {
            if n_i > self.arena.len() - 1 { return None }
            let found = &self.arena[n_i];
            if self.dfs {
                for leaf_idx in found.children.iter().rev() {
                    self.idx_queue.push_front(*leaf_idx)
                }
            } else {
                self.idx_queue.extend(&found.children);
            }
            return Some(&self.arena[n_i]);
        }
        return None;
    }
}

#[derive(Clone)]
pub enum TreeIndex {
    Arena(usize),
    Dfs(usize),
    Bfs(usize)
}

impl<T> Index<TreeIndex> for Tree<T>
where T: PartialEq {
    type Output = T;
    fn index(&self, index: TreeIndex) -> &Self::Output {
        return &self.get(index).unwrap();
    }
}

impl<T> Tree<T>
where T: PartialEq {
    pub fn new(root: T) -> Self {
        return Self{nodes: vec![Node{val:root, children: Vec::new(), level: 0,idx: 0}]};
    }

    pub fn iter_bfs(&self) -> TreeIter<'_,T> {
        let references: Vec<&Node<T>> = self.nodes.iter().map(|owned| owned).collect();
        return  TreeIter{dfs: false, idx_queue : VecDeque::from([0]), arena: references };
    }

    pub fn iter_dfs(&self) -> TreeIter<'_,T> {
        let references: Vec<&Node<T>> = self.nodes.iter().map(|owned| owned).collect();
        return  TreeIter{dfs: true, idx_queue : VecDeque::from([0]), arena: references };
    }

    pub fn nodes_bfs(&self) -> TreeNodeIter<'_,T> {
        let references: Vec<&Node<T>> = self.nodes.iter().map(|owned| owned).collect();
        return  TreeNodeIter{dfs: false, idx_queue : VecDeque::from([0]), arena: references };
    }

    pub fn nodes_dfs(&self) -> TreeNodeIter<'_,T> {
        let references: Vec<&Node<T>> = self.nodes.iter().map(|owned| owned).collect();
        return  TreeNodeIter{dfs: true, idx_queue : VecDeque::from([0]), arena: references };
    }


    pub fn get_node(&self, index: TreeIndex) -> Option<&Node<T>> {
        match index {
            TreeIndex::Bfs(bfs_i) => {
                let mut queue: VecDeque<usize> = VecDeque::from([0]);
                let mut i_count = 0;
                while let Some(n_i) = queue.pop_front() {
                    if i_count == bfs_i { return self.nodes.get(n_i) };
                    let found = &self.nodes[n_i].children;
                    queue.extend(found);
                    i_count = i_count+1;
                }
                return None;
            }
            TreeIndex::Arena(raw_i) => self.nodes.get(raw_i),
            _ => None
        }
    }

    pub fn get(&self, index: TreeIndex) -> Option<&T> {
        self.get_node(index).map(|v| &v.val)
    }

    pub fn get_mut_node(&mut self, index: TreeIndex) -> Option<&mut Node<T>> {
        match index {
            TreeIndex::Bfs(bfs_i) => {
                let mut queue: VecDeque<usize> = VecDeque::from([0]);
                let mut i_count = 0;
                while let Some(n_i) = queue.pop_front() {
                    if i_count == bfs_i { return self.nodes.get_mut(n_i)}
                    let found = &self.nodes[n_i].children;
                    queue.extend(found);
                    i_count = i_count+1;
                }
                return None;
            }
            TreeIndex::Arena(raw_i) => self.nodes.get_mut(raw_i),
            _ => None
        }
    }


    pub fn get_mut(&mut self, index: TreeIndex) -> Option<&mut T> {
        return self.get_mut_node(index).map(|x| &mut x.val);
    }

    fn find_mut_node(&mut self, comp: &T) -> Option<&mut Node<T>> {
        let mut queue: VecDeque<usize> = VecDeque::from([0]);
        let mut i_count = 0;
        while let Some(n_i) = queue.pop_front() {
            if &self.nodes[n_i].val == comp { return self.nodes.get_mut(n_i)}
            let found = &self.nodes[n_i].children;
            queue.extend(found);
            i_count = i_count+1;
        }
        return None;
    }

    pub fn add_node(&mut self, parent: &T, val: T) -> Option<NodePosition> {
        let n_i = self.nodes.len();
        if let Some(parent) = self.find_mut_node(parent) {
            let p_level = parent.level;
            let p_children = parent.children.len();
            let p_idx = parent.idx;
            parent.children.push(n_i);
            self.nodes.push(Node{val, children: Vec::new(), level: p_level+1, idx: n_i});
            return Some(
                NodePosition{
                    depth: p_level + 1,
                    siblings: p_children + 1,
                    raw_idx: n_i,
                    parent_raw_idx: p_idx
                }
            );
        }
        return None;
    }

    pub fn add_node_by_index(&mut self, parent_index: TreeIndex, val: T) -> Option<NodePosition> {
        let n_i = self.nodes.len();
        if let Some(parent) = self.get_mut_node(parent_index) {
            let p_level = parent.level;
            let p_children = parent.children.len();
            let p_idx = parent.idx;
            parent.children.push(n_i);
            self.nodes.push(Node{val, children: Vec::new(), level: p_level+1, idx: n_i});
            return Some(
                NodePosition{
                    depth: p_level + 1,
                    siblings: p_children + 1,
                    raw_idx: n_i,
                    parent_raw_idx: p_idx
                }
            );
        }
        return None;
    }
}

#[cfg(test)]
mod tree_t {
    use super::*;

    fn make_tree() -> Tree<&'static str> {
        let mut tree: Tree<&str> = Tree::new("a");
        tree.add_node(&"a", "b");
        tree.add_node(&"a", "c");
        tree.add_node(&"a", "d");
        tree.add_node(&"b", "e");
        tree.add_node(&"c", "f");
        tree.add_node(&"c", "g");
        return tree;
    }

    #[test]
    fn initialization() {
        let tree: Tree<&str> = make_tree();
        let vals = ["a","b","c","d","e","f","g"];

        assert_eq!(tree.nodes.len(), 7);

        let tree_values_in_order: Vec<&str> = (0..7)
            .map(|index| *tree.get(TreeIndex::Arena(index)).unwrap()).collect();
        assert_eq!(tree_values_in_order, vals);
    }

    #[test]
    fn bfs_iteration() {
        let tree: Tree<&str> = make_tree();
        let vals = ["a","b","c","d","e","f","g"];
        let collected: Vec<&str> = tree.iter_bfs().map(|r| *r).collect();
        assert_eq!(collected, vals);
    }

    #[test]
    fn dfs_iteration() {
        let tree: Tree<&str> = make_tree();
        let vals = ["a","b","e","c","f","g","d"];
        let collected: Vec<&str> = tree.iter_dfs().map(|r| *r).collect();
        assert_eq!(collected, vals);
    }

    #[test]
    fn mutation() {
        let mut tree: Tree<&str> = make_tree();
        let vals_bfs = ["a","b","c","d","e","f","g","h"];
        let vals_dfs = ["a","b","e","c","f","g","h","d"];
        assert_eq!(tree.add_node(&"g", "h").unwrap().raw_idx, 7);
        let collected_bfs: Vec<&str> = tree.iter_bfs().map(|r| *r).collect();
        assert_eq!(collected_bfs, vals_bfs);

        let collected_dfs: Vec<&str> = tree.iter_dfs().map(|r| *r).collect();
        assert_eq!(collected_dfs, vals_dfs);
    }
}
