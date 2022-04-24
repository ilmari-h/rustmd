use std::{collections::{VecDeque, HashMap}, slice::Iter, borrow::{Borrow, BorrowMut}, fmt::Debug, mem::replace};

struct TreeIter<'a,T> where T: Node<'a> {
    pub dfs: bool,
    pub queue: VecDeque<&'a T>
}

struct TreeIterMut<'a,T> where T: Node<'a> {
    pub dfs: bool,
    pub queue: VecDeque<&'a mut T>,
    pub remaining_queue: VecDeque<&'a mut T>
}

impl <'a,T>TreeIter<'a,T> where T: Node<'a> {
    fn new( queue: VecDeque<&'a T>, dfs: bool ) -> Self {
        return Self{queue, dfs}
    }
}

impl <'a,T>Iterator for TreeIter<'a,T> where T: Node<'a> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        match self.queue.pop_front() {
            Some(node) => {
                if self.dfs {
                    for leaf_node in node.leaves().iter().rev() {
                        self.queue.push_front(leaf_node)
                    }
                } else {
                    for leaf_node in node.leaves() {
                        self.queue.push_back(leaf_node)
                    }
                }
                return Some(node);
            }
            None => return None
        }
    }
}

impl <'a,T>TreeIterMut<'a,T> where T: Node<'a> {
    fn new( queue: VecDeque<&'a mut T>, dfs: bool ) -> Self {
        return Self{queue, dfs, remaining_queue: VecDeque::new()}
    }
}

impl <'a,T>Iterator for TreeIterMut<'a,T> where T: Node<'a> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_queue.len() > 0 {
            return self.remaining_queue.pop_front()
        }
        // Using mem::replace for atomically removing and adding nodes to queue
        if self.queue.len() > 0  {
            println!("ITEMS:");
            let d: &'a mut T = self.queue.pop_front().unwrap();
            let nodes = VecDeque::from_iter(d.leaves_mut());
            self.remaining_queue = replace(&mut self.queue, nodes);
            println!("QUEUE {:?}",self.queue.len());
            return self.queue.pop_front()
        }
        print!("NONE");
        return None
    }
}

#[derive(Debug)]
struct TestNode {
    pub val: &'static str,
    pub children: Vec<TestNode>,
}

trait Tree<'a,T>
where T: Node<'a> {
    fn root(&self) -> &T;
    fn root_mut(&mut self) -> &mut T;
    fn bfs_iter_mut(&'a mut self) -> TreeIterMut<T> {
        return TreeIterMut::new( VecDeque::from([self.root_mut()]), false );
    }
    fn bfs_iter(&'a self) -> TreeIter<T> {
        return TreeIter::new( VecDeque::from([self.root()]), false );
    }
    fn dfs_iter(&'a self) -> TreeIter<T> {
        return TreeIter::new( VecDeque::from([self.root()]), true );
    }
}

struct TestTree{
    pub root: TestNode
}

impl Tree<'_,TestNode> for TestTree{
    fn root(&self) -> &TestNode {
        return &self.root;
    }
    fn root_mut(&mut self) -> &mut TestNode {
        return &mut self.root;
    }
}

impl TestTree {
    fn new(root: TestNode) -> Self {
        Self{root}
    }
    fn get_mut_node(&mut self, value: &str) -> Option< &mut TestNode> {
        let c = self.root.leaves_mut().iter_mut().find(|c| c.val == value);
        return c;
    }
}

trait Node<'a>
where Self: Sized{
    fn leaves(&'a self) -> &'a [Self];
    fn leaves_mut(&'a mut self) -> &'a mut [Self];
}

impl <'a>Node<'a> for TestNode {
    fn leaves(&'a self) -> &'a [TestNode] {
        return &self.children[..]
    }
    fn leaves_mut(&'a mut self) -> &'a mut [TestNode] {
        return &mut self.children[..]
    }
}

fn read_tree<'a,T>(tree:&'a T)
where T: Node<'a>, T: Debug{
    let mut queue: VecDeque<&T> = VecDeque::from([tree]);
    while let Some(node) = queue.pop_front() {
        println!("{:?}", node);
        node.leaves().iter().for_each(|child_node| {
            queue.push_back(child_node);
        })
    }
    println!("");
}

fn do_tree<'a,T,N>(tree: &'a T)
where
    T: Tree<'a,N>,
    N: 'a,
    N: Node<'a>,
    N: Debug {
    tree.bfs_iter().for_each(|x| println!("{:?}", x))
}

#[cfg(test)]
mod blog {
    use super::*;
    #[test]
    fn test_tree() {
        let root: TestNode = TestNode {
            val: "a",
            children: vec![
                TestNode { val: "b", children: vec![
                    TestNode { val: "e", children: vec![]},
                ] },
                TestNode { val: "c", children: vec![
                    TestNode { val: "f", children: vec![]},
                    TestNode { val: "g", children: vec![] }
                ] },
                TestNode { val: "d", children: vec![] }
            ]
        };
        let tree: TestTree = TestTree::new(root);
        assert!(["a","b","c","d","e","f","g"].iter().eq(tree.bfs_iter().map(|n| &n.val)));
        assert!(["a","b","e","c","f","g","d"].iter().eq(tree.dfs_iter().map(|n| &n.val)));
    }

    #[test]
    fn test_tree_mut() {
        let root: TestNode = TestNode {
            val: "a",
            children: vec![
                TestNode { val: "b", children: vec![
                    TestNode { val: "e", children: vec![]},
                ] },
                TestNode { val: "c", children: vec![
                    TestNode { val: "f", children: vec![]},
                    TestNode { val: "g", children: vec![] }
                ] },
                TestNode { val: "d", children: vec![] }
            ]
        };
        let mut tree: TestTree = TestTree::new(root);
        let found = tree.bfs_iter_mut().find(|n| n.val == "b");
        if let Some(node) = found {
            node.children.pop();
        }
        read_tree(tree.root());
        tree.bfs_iter_mut().for_each(|n| { n.children.pop(); });
    }
}
