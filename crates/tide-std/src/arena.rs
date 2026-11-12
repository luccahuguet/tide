//! Generational handles for graphs and cycles (reference §3.6; 02 §10.9).
//!
//! Handles belong to the arena that created them. Arenas do not detect handles from another
//! arena of the same element type: mixing them is a logic error and may return another element,
//! return `None`, or panic, but cannot cause undefined behavior. No global identity is used.

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

/// An owned collection with stable generational handles. It is `Pure` exactly when `T` is.
///
/// Removed slots are reused last-freed-first. A slot that reaches generation `u32::MAX` is
/// retired, so an old handle never becomes valid again in its originating arena.
pub struct Arena<T> {
    slots: Vec<Slot<T>>,
    free: Option<u32>,
    len: usize,
}

struct Slot<T> {
    generation: u32,
    entry: Entry<T>,
}

enum Entry<T> {
    Occupied(T),
    Vacant(Option<u32>),
}

impl<T> Arena<T> {
    /// Creates an empty arena, without requiring `T: Default`.
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            free: None,
            len: 0,
        }
    }

    /// Inserts a value and returns its handle.
    ///
    /// Panics if a new slot would exceed `u32::MAX` slots (including retired slots).
    pub fn insert(&mut self, value: T) -> Id<T> {
        let index = if let Some(index) = self.free {
            let slot = &mut self.slots[index as usize];
            let Entry::Vacant(next) = slot.entry else {
                unreachable!()
            };
            self.free = next;
            slot.entry = Entry::Occupied(value);
            index
        } else {
            assert!(
                self.slots.len() < u32::MAX as usize,
                "arena slot limit exceeded"
            );
            let index = self.slots.len() as u32;
            self.slots.push(Slot {
                generation: 0,
                entry: Entry::Occupied(value),
            });
            index
        };
        self.len += 1;
        Id {
            index,
            generation: self.slots[index as usize].generation,
            _p: PhantomData,
        }
    }

    /// Returns the value, or `None` for a stale handle.
    pub fn get(&self, id: Id<T>) -> Option<&T> {
        let slot = self.slots.get(id.index as usize)?;
        if slot.generation != id.generation {
            return None;
        }
        match &slot.entry {
            Entry::Occupied(value) => Some(value),
            Entry::Vacant(_) => None,
        }
    }

    /// Returns the mutable value, or `None` for a stale handle.
    pub fn get_mut(&mut self, id: Id<T>) -> Option<&mut T> {
        let slot = self.slots.get_mut(id.index as usize)?;
        if slot.generation != id.generation {
            return None;
        }
        match &mut slot.entry {
            Entry::Occupied(value) => Some(value),
            Entry::Vacant(_) => None,
        }
    }

    /// Whether the handle names a live element.
    pub fn contains(&self, id: Id<T>) -> bool {
        self.get(id).is_some()
    }

    /// Removes and returns the value, or `None` for a stale handle.
    pub fn remove(&mut self, id: Id<T>) -> Option<T> {
        if !self.contains(id) {
            return None;
        }
        let slot = &mut self.slots[id.index as usize];
        slot.generation += 1;
        let next = if slot.generation == u32::MAX {
            None
        } else {
            self.free
        };
        let Entry::Occupied(value) = std::mem::replace(&mut slot.entry, Entry::Vacant(next)) else {
            unreachable!()
        };
        if slot.generation != u32::MAX {
            self.free = Some(id.index);
        }
        self.len -= 1;
        Some(value)
    }

    /// Borrows distinct live elements in the order requested.
    /// Returns `None` if any handle is stale or if handles repeat; an empty array succeeds.
    pub fn get_disjoint_mut<const N: usize>(&mut self, ids: [Id<T>; N]) -> Option<[&mut T; N]> {
        if !ids.iter().all(|&id| self.contains(id)) {
            return None;
        }
        let slots = self
            .slots
            .get_disjoint_mut(ids.map(|id| id.index as usize))
            .ok()?;
        Some(slots.map(|slot| match &mut slot.entry {
            Entry::Occupied(value) => value,
            Entry::Vacant(_) => unreachable!(),
        }))
    }

    /// The number of live elements, excluding removed and retired slots.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether there are no live elements.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Iterates over live handles and values in slot-index order.
    /// This provisional API does not promise insertion order after slot reuse.
    pub fn iter(&self) -> impl Iterator<Item = (Id<T>, &T)> {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| match &slot.entry {
                Entry::Occupied(value) => Some((
                    Id {
                        index: index as u32,
                        generation: slot.generation,
                        _p: PhantomData,
                    },
                    value,
                )),
                Entry::Vacant(_) => None,
            })
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Index<Id<T>> for Arena<T> {
    type Output = T;
    #[track_caller]
    fn index(&self, id: Id<T>) -> &T {
        self.get(id).expect("stale arena id")
    }
}

impl<T> IndexMut<Id<T>> for Arena<T> {
    #[track_caller]
    fn index_mut(&mut self, id: Id<T>) -> &mut T {
        self.get_mut(id).expect("stale arena id")
    }
}

/// A `u32` slot index and generation, with no ownership of `T`.
///
/// `Copy`, `Eq`, `Hash`, `Ord`, `Debug`, `Pure`, `Send` and `Sync` hold for every `T`;
/// `'static` holds when `T` is. Ordering compares index, then generation. `Ord` and `Debug`
/// remain provisional. Handles must be used only with their originating arena.
pub struct Id<T> {
    index: u32,
    generation: u32,
    _p: PhantomData<fn() -> T>,
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for Id<T> {}
impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.generation == other.generation
    }
}
impl<T> Eq for Id<T> {}
impl<T> Hash for Id<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
        self.generation.hash(state);
    }
}
impl<T> PartialOrd for Id<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<T> Ord for Id<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.index, self.generation).cmp(&(other.index, other.generation))
    }
}
impl<T> fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Id")
            .field("index", &self.index)
            .field("generation", &self.generation)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exhausted_generation_retires_without_disturbing_the_free_list() {
        let mut arena = Arena::new();
        let first = arena.insert(1);
        let last = arena.insert(2);
        arena.remove(first);
        arena.slots[last.index as usize].generation = u32::MAX - 1;
        let last = Id {
            generation: u32::MAX - 1,
            ..last
        };
        assert_eq!(arena.remove(last), Some(2));
        assert_eq!(arena.remove(last), None);
        assert!(arena.is_empty());
        let reused = arena.insert(3);
        assert_eq!(reused.index, first.index);
        let appended = arena.insert(4);
        assert_eq!(appended.index, 2);
        assert_eq!(arena.get(first), None);
        assert_eq!(arena.get(last), None);
        assert_eq!(arena.len(), 2);
        let outside = Id {
            index: u32::MAX,
            ..last
        };
        assert_eq!(arena.get(outside), None);
        assert_eq!(arena.get_mut(outside), None);
        assert_eq!(arena.remove(outside), None);
        assert!(arena.get_disjoint_mut([reused, outside]).is_none());
        assert_eq!(
            arena.iter().map(|(_, value)| *value).collect::<Vec<_>>(),
            vec![3, 4]
        );
    }

    #[test]
    fn handles_do_not_own_or_require_traits_of_their_payload() {
        fn send_sync<T: Send + Sync>() {}
        send_sync::<Id<std::rc::Rc<std::cell::Cell<u8>>>>();
        struct NoTraits;
        let mut arena: Arena<NoTraits> = Arena::default();
        let id = arena.insert(NoTraits);
        let mut set = std::collections::HashSet::new();
        set.insert(id);
        assert!(!set.insert(id.clone()));
        assert_eq!(id.partial_cmp(&id), Some(Ordering::Equal));
        assert_eq!(format!("{id:?}"), "Id { index: 0, generation: 0 }");
    }
}
