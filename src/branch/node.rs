use std::mem;

use crate::Vector2;
use crate::branch::line::Line;

pub enum Node<It> {
    Leaf(It),
    Children(Vec<Node<It>>),
}

pub struct NodeIter<'a, It> {
    children: &'a [Node<It>],
    parent: Option<Box<NodeIter<'a, It>>>,
}

impl<It> Default for NodeIter<'_, It> {
    fn default() -> Self {
        NodeIter { children: &[], parent: None }
    }
}

impl<It> Node<It> {
    pub fn iter(&self) -> NodeIter<'_, It> {
        NodeIter {
            children: std::slice::from_ref(self),
            parent: None,
        }
    }

    pub fn iter_mut(&mut self) -> NodeIterMut<'_, It> {
        NodeIterMut {
            children: std::slice::from_mut(self),
            parent: None,
        }
    }
}

impl<'a, It> Iterator for NodeIter<'a, It> {
    type Item = &'a It;

    fn next(&mut self) -> Option<Self::Item> {
        match self.children.get(0) {
            None => match self.parent.take() {
                Some(parent) => {
                    // continue with the parent node
                    *self = *parent;
                    self.next()
                }
                None => None,
            },
            Some(Node::Leaf(item)) => {
                self.children = &self.children[1..];
                Some(item)
            }
            Some(Node::Children(children)) => {
                self.children = &self.children[1..];

                // start iterating the child trees
                *self = NodeIter {
                    children: children.as_slice(),
                    parent: Some(Box::new(mem::take(self))),
                };
                self.next()
            }
        }
    }
}

pub struct NodeIterMut<'a, It> {
    children: &'a mut [Node<It>],
    parent: Option<Box<NodeIterMut<'a, It>>>,
}

impl<It> Default for NodeIterMut<'_, It> {
    fn default() -> Self {
        NodeIterMut {
            children: &mut [],
            parent: None,
        }
    }
}

impl<'a, It> Iterator for NodeIterMut<'a, It> {
    type Item = &'a mut It;

    fn next(&mut self) -> Option<Self::Item> {
        let children = mem::take(&mut self.children);
        match children.split_first_mut() {
            None => match self.parent.take() {
                Some(parent) => {
                    // continue with the parent node
                    *self = *parent;
                    self.next()
                }
                None => None,
            },
            Some((first, rest)) => {
                self.children = rest;
                match first {
                    Node::Leaf(item) => Some(item),
                    Node::Children(children) => {
                        *self = NodeIterMut {
                            children: children.as_mut_slice(),
                            parent: Some(Box::new(mem::take(self))),
                        };
                        self.next()
                    }
                }
            }
        }
    }
}

#[test]
fn test_node_iterator() {
    let tree = Node::Children(vec![
        Node::Leaf(1),
        Node::Leaf(2),
        Node::Children(vec![
            Node::Leaf(4),
            Node::Leaf(3),
            Node::Leaf(5)
        ])
    ]);

    assert_eq!(tree.iter().copied().collect::<Vec<i32>>(), vec![1, 2, 4, 3, 5]);
}

#[test]
fn test_line_tree() {
    let tree = Node::Children(vec![
        Node::Leaf(Line::new(Vector2 { x: 1.0, y: 2.0 }, Vector2 { x: 0.0, y: 0.0 }, None)),
        Node::Children(vec![
            Node::Leaf(Line::new(Vector2 { x: -1.0, y: -1.0 }, Vector2 { x: -2.0, y: -2.0 }, None))
        ])
    ]);

    let lines = tree.iter().collect::<Vec<&Line>>();

    println!("{:?}", lines);
}