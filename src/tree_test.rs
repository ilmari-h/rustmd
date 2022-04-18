use std::{collections::{VecDeque, HashMap}, slice::Iter, borrow::Borrow, fmt::Debug};

// trait Node<'a,T: 'a> {
//     type NodeIterator : Iterator<Item=&'a T>;
//     fn leaves(&'a self) -> Self::NodeIterator;
// }
//
// struct TestNode {
//     pub val: String,
//     pub children: Vec<TestNode>
// }
//
// impl<'a> Node<'a,TestNode> for TestNode {
//     type NodeIterator = Iter<'a,TestNode>;
//     fn leaves(&'a self) -> Self::NodeIterator {
//         return self.children.iter();
//     }
// }

struct TreeIter<'a,T> where T: Node<'a>{
    pub queue: VecDeque<&'a T>
}

impl <'a,T>Iterator for TreeIter<'a,T> where T: Node<'a> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        // TODO: mutate queue as in our previous algorithm
        self.queue.pop_front()
    }
}

impl <'a,T>TreeIter<'a,T> where T: Node<'a> {
    fn new( queue: VecDeque<&'a T> ) -> Self {
        return Self{queue}
    }
}

struct TestNode {
    pub val: String,
    pub children: Vec<TestNode>,
}

trait Tree<'a,T>
where T: Node<'a> {
    fn root(&self) -> &T;
    fn bfs_iter(&'a self) -> TreeIter<T> { // NOTE: here we can skip specifying the lifetime parameter, there's no reference to TreeIter's lifetime
        let mut queue: VecDeque<&T> = VecDeque::new();
        let leaves = self.root().leaves();
        for leaf in leaves {
            queue.push_back(leaf)
        }
        return TreeIter::new(queue)
    }
}

struct TestTree{
    pub root: TestNode
}

// NOTE: here get to use an anonymous lifetime because there's no reference to it in implementation?
impl Tree<'_,TestNode> for TestTree{
    fn root(&self) -> &TestNode {
        return &self.root;
    }
}

impl TestTree {
    fn new(root: TestNode) -> Self {
        Self{root}
    }
}

trait Node<'a>
where Self: Sized{
    fn leaves(&'a self) -> &'a [Self];
}

impl <'a>Node<'a> for TestNode {
    fn leaves(&'a self) -> &'a [TestNode] {
        return &self.children[..]
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

fn do_tree2(tree: &TestTree){
    let mut i = tree.bfs_iter();
    let x = i.next();
    let y = i.next();
}


fn test_tree() {
    let root: TestNode = TestNode {
        val: String::from("a"),
        children: vec![
            TestNode { val: String::from("b"), children: vec![
                TestNode { val: String::from("e"), children: vec![]},
            ] },
            TestNode { val: String::from("c"), children: vec![
                TestNode { val: String::from("f"), children: vec![]},
                TestNode { val: String::from("g"), children: vec![] }
            ] },
            TestNode { val: String::from("d"), children: vec![] }
        ]
    };
    let tree: TestTree = TestTree::new(root);
    do_tree2(&tree);

}

impl Debug for TestNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("content", &self.val)
            .finish()
    }
}

#[cfg(test)]
mod blog {

    use super::*;
    #[test]
    fn test_main() {

        let tree: TestNode = TestNode {
            val: String::from("a"),
            children: vec![
                TestNode { val: String::from("b"), children: vec![
                    TestNode { val: String::from("e"), children: vec![]},
                ] },
                TestNode { val: String::from("c"), children: vec![
                    TestNode { val: String::from("f"), children: vec![]},
                    TestNode { val: String::from("g"), children: vec![] }
                ] },
                TestNode { val: String::from("d"), children: vec![] }
            ]
        };
        //read_tree(&tree);
        test_tree();
    }
}
