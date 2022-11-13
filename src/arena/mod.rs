use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

pub mod prelude {
    use std::sync::{Arc, RwLock, Weak};

    pub trait HasId: Sync + Send  {
        type Id;
        fn get_id(&self) -> Self::Id;
    }

    impl HasId for usize {
        type Id = usize;
        fn get_id(&self) -> usize { *self }
    }

    pub type SharedRef<T> = Arc<RwLock<T>>;
    pub type WeakRef<T> = Weak<RwLock<T>>;

    pub trait IsMemoryArena {
        type Id;
        type Node;

        fn get_node(&self, id: &Self::Id) -> Option<SharedRef<Self::Node>>;
        fn get_node_weak(&self, id: &Self::Id) -> Option<WeakRef<Self::Node>>;

        /// Adds a node to the tree.
        fn add_node(&mut self, node: Self::Node) -> Result<(), String>;

        /// Removes the node from the tree.
        fn delete_node(&mut self, id: &Self::Id) -> Result<(), String>;

        /// Returns a new unique Id.
        fn get_new_id(&mut self) -> Self::Id;
    }
}

use prelude::*;

pub struct Arena<T> {
    storage: Arc<RwLock<HashMap<usize, SharedRef<T>>>>,
    id_counter: AtomicUsize
}

impl<T: HasId + Debug + Clone + Send + Sync> Arena<T> {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::<usize, SharedRef<T>>::new())),
            id_counter: AtomicUsize::default()
        }
    }
}

impl<T: HasId + Debug + Clone + Send + Sync> IsMemoryArena for Arena<T>
    where usize: From<T::Id>
{
    type Id = usize;
    type Node = T;

    fn get_node(&self, id: &Self::Id) -> Option<SharedRef<Self::Node>> {
        self.storage.read().unwrap().get(id).map(Arc::clone)
    }

    fn get_node_weak(&self, id: &Self::Id) -> Option<WeakRef<Self::Node>> {
        self.storage.read().unwrap().get(id).map(Arc::downgrade)
    }

    fn add_node(&mut self, node: Self::Node) -> Result<(), String> {
        if self.storage.read().unwrap().contains_key(&node.get_id().into()) {
            return Err(String::from("node already exists!"));
        }

        self.storage.write().unwrap().insert(node.get_id().into(), SharedRef::new(RwLock::new(node.clone())));

        Ok(())
    }

    fn delete_node(&mut self, id: &Self::Id) -> Result<(), String> {
        if !self.storage.read().unwrap().contains_key(id) {
            return Err(String::from("node doesn't exist!"));
        }

        self.storage.write().unwrap().remove( id);

        Ok(())
    }

    fn get_new_id(&mut self) -> Self::Id {
        self.id_counter.fetch_add(1, Ordering::SeqCst)
    }
}
