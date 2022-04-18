use core::fmt;
use std::collections::{VecDeque, HashSet, HashMap};


struct Node {
    pub val: String,
    pub children: Vec<Node>
}

pub trait Root<T> {
    fn leaves(&self) -> &Vec<T>;
}

impl Root<Node> for Node {
    fn leaves(&self) -> &Vec<Node> {
        return &self.children;
    }
}

impl <T>fmt::Debug for dyn Root<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Root Node: Generic")
         .field("Root item",&"")
         .finish()
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Node")
         .field("Item",&self.val)
         .finish()
    }
}

struct VisitRecord<'a,T> {
    pub level: u32,
    pub id: u32,
    pub value: &'a T
}

#[derive(PartialEq)]
enum SearchMode {
    BreadthFirst,
    DepthFirst
}

fn read_tree<T>(root: &T, mode: SearchMode)
where
    T: Root<T>,
    T: fmt::Debug {

    let mut id = 0; // Give each visited node its own id
    let mut stack: VecDeque<VisitRecord<T>> = VecDeque::from([VisitRecord{
        level:1,
        id:0,
        value:root
    }]);

    // Last children of parent nodes, maps last child ID to parent ID
    let mut last_leaves: HashMap<u32,u32> = HashMap::new();
    while !stack.is_empty() {
        let current: Option<VisitRecord<T>> = stack.pop_front();
        if let Some(node) = current {
            println!("Level: {:?} - Id: {:?} - Value: {:?}", node.level, node.id, node.value);

            // All node's leaves visited at this point.
            if let Some(parent) = last_leaves.get(&node.id) {
                //println!("Parent and last leaf: {:?} - {:?}", parent, node.value);
            }
            let children: &Vec<T> = &node.value.leaves();
            last_leaves.insert(id + 1, node.id);
            let add_to_queue = |c| {
                id += 1;
                let leaf = VisitRecord{
                    level:node.level+1,
                    id,
                    value:c
                };
                if SearchMode::DepthFirst == mode {
                    stack.push_front(leaf);
                } else {
                    stack.push_back(leaf);
                }
            };

            if SearchMode::DepthFirst == mode {
                children.iter().rev().for_each(add_to_queue);
            } else {
                children.iter().for_each(add_to_queue);
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn t_tree() {
        let tree: Node = Node {
            val: String::from("a"),
            children: vec![
                Node { val: String::from("b"), children: vec![
                    Node { val: String::from("e"), children: vec![]},
                ] },
                Node { val: String::from("c"), children: vec![
                    Node { val: String::from("f"), children: vec![]},
                    Node { val: String::from("g"), children: vec![] }
                ] },
                Node { val: String::from("d"), children: vec![] }
            ]
        };
        read_tree(&tree, SearchMode::BreadthFirst);
        read_tree(&tree, SearchMode::DepthFirst);
    }

}
