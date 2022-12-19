use std::fmt::Debug;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::arena::{Arena, Id};
use crate::arena::prelude::{HasId, IsMemoryArena};
use crate::spatial::quadtree::prelude::*;

/// This is the trait bound for the payload associated with a Point in the tree.
pub trait IsPayload: Clone + Debug + Send + Sync {}

impl<T: Clone + Debug + Send + Sync> IsPayload for T {}

/// This represents the type of payload that is stored in each Quad of the tree.
pub type Node<T> = (Vec2, T);

/// A quad represents a quadrant in 3D space, it contains a single point and optionally 4 other
/// quads which subdivide the space further.
#[derive(Clone, Debug)]
struct Quad<P: IsPayload> {
    pub id: Id,

    pub bbox: BBox2D,

    pub point: Option<Node<P>>,

    // The ordering goes SW, SE, NE, NW
    pub children: Option<[Id; 4]>
}

/// A Point Quadtree is a data structure used to perform efficient queries of points / regions in
/// 2D space. The tree works by recursively subdividing (partitioning) 3D space into buckets.
pub struct PointQuadtree<P: IsPayload> {
    arena: Arena<Quad<P>>,
    root_id: Id,
    size: AtomicUsize
}

impl<P: IsPayload> PointQuadtree<P> {

    /// Returns the number of points contained in this tree.
    pub fn len(&self) -> usize {
        self.size.load(Ordering::SeqCst)
    }

    /// Returns a new Quadtree bounded by the given BBox.
    pub fn new(bbox: &BBox2D) -> Self {
        let mut arena = Arena::new();

        let root_id = arena.get_new_id();
        let root = Quad::<P> {
            id: root_id,
            bbox: *bbox,
            point: None,
            children: None
        };

        arena.add_node(root).expect("could not add root node!");

        Self {
            arena,
            root_id,
            size: Default::default()
        }
    }

    /// Attempts to insert 'elem' into the tree, returning false if the point already exists.
    pub fn insert(&mut self, point: &Vec2, payload: P) -> bool {
        let root = self.root_id;
        if self._insert(&(*point, payload), &root) {
            self.size.fetch_add(1, Ordering::SeqCst);
            return true;
        }

        false
    }

    /// Returns all points in the tree within the given BBox.
    pub fn find_within(&self, bbox: &BBox2D) -> Vec<Node<P>> {
        self._find_within(bbox, &self.root_id)
    }

    /// Searches the tree for the given point.
    pub fn find(&self, p: &Vec2) -> Option<Node<P>> {
        self._find(p, &self.root_id)
    }

    fn _find_within(&self, bbox: &BBox2D, quad_id: &Id) -> Vec<Node<P>> {
        let quad_ref = self.arena.get_node(quad_id).expect("could not find node");
        let quad = quad_ref.read().unwrap();

        if !quad.bbox.intersects(bbox) {
            return vec![];
        }

        let mut result = vec![];

        match &quad.point {
            None => {},
            Some(node) => {
                if bbox.contains(&node.0) {
                    result.push(node.clone())
                }
            }
        }

        match &quad.children {
            None => {}
            Some(children) => {
                for id in children {
                    result.append(&mut self._find_within(bbox, id))
                }
            }
        }

        result
    }

    fn _find(&self, p: &Vec2, quad_id: &Id) -> Option<Node<P>> {
        let quad_ref = self.arena.get_node(quad_id).expect("could not find node");
        let quad = quad_ref.read().unwrap();

        // If the bbox itself doesn't contain the point, then the point could not possibly be
        // contained in this node or any subtrees of this node.
        if !quad.bbox.contains(p) {
            return None;
        }

        // Otherwise, we'll need to look at the point contained in this node or the points contained
        // in the subtrees of this node.
        match &quad.point {
            // If we don't have a point, the point can't be contained.
            None => None,

            Some(point) => {
                // Let's see if the point stored at this node matches.
                if point.0 == *p {
                    Some(point.clone())
                } else {
                    // Otherwise, we'll need to look in all of this node's subtrees.
                    match &quad.children {
                        // If we don't have any subtrees, the point can't be contained.
                        None => None,

                        Some(children) => {
                            for child in children {
                                if let Some(result) = self._find(p, child) {
                                    return Some(result);
                                }
                            }
                            None
                        }
                    }
                }
            }
        }
    }

    pub fn _insert(&mut self, elem: &Node<P>, quad_id: &Id) -> bool {
        let quad_ref = self.arena.get_node(quad_id).expect("could not find node");
        let mut quad = quad_ref.write().unwrap();

        if !quad.bbox.contains(&elem.0) {
            return false;
        }

        if quad.point.is_none() {
            quad.point = Some(elem.clone());
            return true;
        } else {
            if quad.point.as_ref().unwrap().0 == elem.0 {
                return false;
            }

            // --
            // Subdivide we need to.
            if quad.children.is_none() {
                let mut add_one = |bbox| {
                    let new_id : Id = self.arena.get_new_id();

                    let new_node = Quad::<P>::new(new_id.clone(), bbox);
                    self.arena.add_node(new_node).expect("could not add node!");
                    new_id
                };

                let boxes = quad.bbox.subdivide(&quad.point.as_ref().unwrap().0);

                quad.children = Some(
                    [
                        add_one(boxes[0]),
                        add_one(boxes[1]),
                        add_one(boxes[2]),
                        add_one(boxes[3]),
                    ]
                );
            }

            // --
            // Then try to insert the point into any of our children.
            quad.children.as_ref().unwrap().iter().any(|i| {
                self._insert(&elem, i)
            })
        }
    }
}

impl<P: IsPayload> Quad<P> {
    pub fn new(id: Id, bbox: BBox2D) -> Self {
        Self {
            id,
            bbox,
            point: None,
            children: None,
        }
    }
}

impl<P: IsPayload> HasId for Quad<P> {
    type Id = Id;
    fn get_id(&self) -> Self::Id {
        self.id
    }
}
