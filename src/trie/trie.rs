use std::fmt::Debug;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::arena::*;
use crate::arena::prelude::*;
use crate::trie::grammar::*;

type Id = usize;

#[derive(Debug, Clone)]
struct TrieNode<T: Debug + Clone + Send + Sync> {
    pub id: Id,

    pub payload: Option<T>,

    /// These 2 are dependent on the Grammar of the Trie
    pub arity: usize,
    pub children: Vec<Option<Id>>,
}

impl<T: Debug + Clone + Send + Sync> HasId for TrieNode<T> {
    type Id = Id;

    fn get_id(&self) -> Self::Id {
        self.id
    }
}

/// This class represents a thread-safe Trie (prefix tree) data structure.
pub struct Trie<T: Debug + Clone + Send + Sync> {
    arena: Arena<TrieNode<T>>,
    grammar: Grammar,
    root: Id,
    size: AtomicUsize
}

impl<T: Debug + Clone + Send + Sync> TrieNode<T> {
    /// Constructs a new TriNode from the given arguments
    pub fn new(id: Id, payload: Option<T>, arity: usize) -> Self {
        Self {
            id,
            payload,
            arity,
            children: vec![None; arity]
        }
    }

    /// Returns true if a payload is stored at this node.
    pub fn is_terminal(&self) -> bool {
        self.payload.is_some()
    }

    /// Returns true if all children are None.
    pub fn is_leaf(&self) -> bool {
        self.children.iter().all(|x| x.is_none())
    }

    /// Returns true if the node has no children and it is not terminal.
    fn can_delete(&self) -> bool {
        !self.is_terminal() && self.is_leaf()
    }
}

enum OnCollision {
    ReturnError,
    ApplyFn,
}

impl<T: Default + Debug + Clone + Send + Sync> Trie<T> {

    /// Constructs a new Trie with the given Grammar
    pub fn new(grammar: Grammar) -> Self {
        let mut arena = Arena::<TrieNode<T>>::new();

        let root: Id = arena.get_new_id();

        let root_node = TrieNode::<T>::new(
            root,
            None,
            grammar.seq().len()
        );

        arena.add_node(root_node).expect("failed to add root to tree!");

        Self {
            arena,
            grammar,
            root,
            size: AtomicUsize::new(0)
        }
    }

    /// Attempts to insert 'seq', returning an error if it already exists.
    pub fn insert(&mut self, seq: &str, t: T) -> Result<(), String> {
        let seq = self.preprocess_seq(seq);
        let root = self.root;
        self._insert_apply(&seq[..], &root, t, |_| T::default(), OnCollision::ReturnError)
            .and_then(|_| Ok(()))
    }

    /// Inserts 'seq', returning the previous value if it already exists.
    pub fn insert_or_update(&mut self, seq: &str, t: T) -> Result<Option<T>, String> {
        self.insert_or_apply(seq, t.clone(), |_| t.clone())
    }

    /// Inserts 'seq', returning the previous value if it already exists.
    pub fn insert_or_apply<F>(
        &mut self,
        seq: &str,
        t: T,
        f: F
    ) -> Result<Option<T>, String>
        where F: Fn(&T) -> T
    {
        let seq = self.preprocess_seq(seq);
        let root = self.root;
        self._insert_apply(&seq[..], &root, t, f, OnCollision::ApplyFn)
    }

    fn _insert_apply<F>(
        &mut self,
        seq: &[usize],
        node_id: &Id,
        t: T,
        f: F,
        on_collision: OnCollision,
    ) -> Result<Option<T>, String>
        where F: Fn(&T) -> T
    {
        if seq.len() == 0 {
            let node_ref = self.arena.get_node(node_id).expect("node doesnt exist!");
            let mut node = node_ref.write().unwrap();

            return if node.payload.is_some() {
                match on_collision {
                    OnCollision::ReturnError => {
                        Err(String::from("key already exists"))
                    }
                    OnCollision::ApplyFn => {
                        let prev = node.payload.take().unwrap();
                        node.payload = Some(f(&prev));
                        Ok(Some(prev))
                    }
                }
            } else {
                self.size.fetch_add(1, Ordering::SeqCst);
                node.payload = Some(t);
                Ok(None)
            }
        }

        let (idx, remaining) = seq.split_first().unwrap();

        let next_id: Id = {
            let node_ref = self.arena.get_node(node_id).expect("node doesnt exist!");

            let child_id = node_ref.read().unwrap().children[*idx];

            match child_id {
                None => {
                    let next_id = self.arena.get_new_id();

                    let child = TrieNode::<T>::new(
                        next_id.clone(),
                        None,
                        node_ref.read().unwrap().arity
                    );

                    self.arena.add_node(child).expect("could not add node!");

                    node_ref.write().unwrap().children[*idx] = Some(next_id);

                    next_id
                }

                Some(next) => { next }
            }
        };

        self._insert_apply(&remaining[..], &next_id, t, f, on_collision)
    }

    pub fn find(&self, seq: &str) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            self._find(&self.preprocess_seq(seq)[..], &self.root)
        }
    }

    pub fn contains(&self, seq: &str) -> bool {
        self.find(seq).is_some()
    }

    pub fn len(&self) -> usize {
        self.size.load(Ordering::SeqCst)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn delete(&mut self, seq: &str) -> Result<Option<T>, String> {
        if self.is_empty() {
            Err(String::from("sequence not found because container is empty!"))
        } else {
            let seq = self.preprocess_seq(seq);
            let root = self.root;
            self._delete(&seq[..], &root).and_then(|(_, x)| Ok(x))
        }
    }

    fn _delete(&mut self, seq: &[usize], node_id: &Id) -> Result<(bool, Option<T>), String> {
        let node_ref = self.arena.get_node(node_id).unwrap();

        match seq.split_first() {
            None => {
                let mut node = node_ref.write().unwrap();

                if !node.is_terminal() {
                    Err(String::from("sequence not found!"))
                } else {
                    let prev_result = node.payload.take();

                    self.size.fetch_sub(1, Ordering::SeqCst);
                    if node.id != self.root && node.can_delete() {
                        self.arena.delete_node(&node.id).expect("could not delete node");
                        Ok((true, prev_result))
                    } else {
                        Ok((false, prev_result))
                    }
                }
            }

            // --
            // Otherwise, we'll need to traverse deeper in the tree by recursively calling
            // _find(...) on the correct child.
            Some((next_idx, remainder)) => {
                let child_id = node_ref.read().unwrap().children[*next_idx];

                match child_id {
                    None => {
                        Err(String::from("sequence not found!"))
                    },

                    Some(id) => {
                        match self._delete(remainder, &id) {
                            Err(e) => Err(e),

                            Ok((child_deleted, payload)) => {
                                let mut node = node_ref.write().unwrap();
                                if child_deleted {
                                    node.children[id] = None;
                                }

                                if node.can_delete() {
                                    self.arena.delete_node(node_id).expect("could not delete node");
                                    Ok((true, payload))
                                } else {
                                    Ok((false, payload))
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn preprocess_seq(&self, seq: &str) -> Vec<usize> {
        match self.grammar.to_indices(seq) {
            Ok(indices) => indices,
            Err(msg) => panic!("{}", msg)
        }
    }

    fn _find(&self, seq: &[usize], node_id: &Id) -> Option<T> {
        match self.arena.get_node(node_id) {
            // If the node doesn't exist, the string is definitely not in the tree.
            None => {
                None
            }

            // If the node exists, we need to search deeper for the string.
            Some(node_ref) => {
                match seq.split_first() {
                    // --
                    // If seq is empty, then the string is found IFF 'node.payload' is Some
                    None => {
                        node_ref.read().unwrap().payload.clone()
                    }

                    // --
                    // Otherwise, we'll need to traverse deeper in the tree by recursively calling
                    // _find(...) on the correct child.
                    Some((next_idx, remainder)) => {
                        match node_ref.read().unwrap().children[*next_idx] {
                            None => { None }
                            Some(id) => {
                                self._find(&remainder[..], &id)
                            }
                        }
                    }
                }
            }
        }
    }
}
