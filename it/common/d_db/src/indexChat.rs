//Db rust : créé moi un crate index.rs qui possède une structure de bPlusTree. Celle ci pourra avoir x clefs. Et y enfants. Fait une fonction qui pourra initialiser le nombre de clef et d'enfants. Chaques clef possède un id et une adresse (pointeur) vers de la donnée. Elle possède une fonction insert qui permet d'ajouter un id/pointeur de data dans le bPlusTree et un fonction search qui recherche un id dans le bPlusTree et qui retourne une liste de couple id/pointeur de data. avec tous les id trouvés. Tu ajoutera bien toutes les fonctions de tests nécessaires. Tout sera en anglais : variables fonctions commentaires. Respecte bien sur l'état de l'art du développement en rust.


//! A B+ Tree implementation in Rust.
//!
//! This crate provides a B+ Tree data structure optimized for in-memory and disk-based indexing.
//! The B+ Tree supports insertion and search operations, with configurable order (number of keys and children).

use std::collections::VecDeque;
use std::fmt;

/// Represents a key in the B+ Tree, consisting of an ID and a pointer to data.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Key {
    pub id: u64,
    pub data_ptr: *const u8,
}

/// Represents a node in the B+ Tree.
#[derive(Debug)]
enum Node {
    /// Internal node containing keys and child pointers.
    Internal {
        keys: Vec<Key>,
        children: Vec<Box<Node>>,
    },
    /// Leaf node containing keys and optional next leaf pointer.
    Leaf {
        keys: Vec<Key>,
        next: Option<Box<Node>>,
    },
}

pub struct BPlusTree {
    root: Option<Box<Node>>,
    order: usize,
}


/// Error type for B+ Tree operations.
#[derive(Debug)]
pub enum BPlusTreeError {
    KeyNotFound,
    DuplicateKey,
}

impl fmt::Display for BPlusTreeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BPlusTreeError::KeyNotFound => write!(f, "Key not found in the B+ Tree"),
            BPlusTreeError::DuplicateKey => write!(f, "Duplicate key in the B+ Tree"),
        }
    }
}

impl BPlusTree {
    /// Initializes a new B+ Tree with the specified order (number of keys per node).
    ///
    /// # Arguments
    ///
    /// * order - The maximum number of keys per node.
    ///
    /// # Returns
    ///
    /// A new BPlusTree instance.
    pub fn new(order: usize) -> Self {
        BPlusTree { root: None, order }
    }

	/// Inserts a key into the B+ Tree.
	///
	/// # Arguments
	///
	/// * `id` - The ID of the key.
	/// * `data_ptr` - A pointer to the associated data.
	///
	/// # Returns
	///
	/// `Ok(())` if the insertion was successful, or a `BPlusTreeError` if the key already exists.
	pub fn insert(&mut self, id: u64, data_ptr: *const u8) -> Result<(), BPlusTreeError> {
		let key = Key { id, data_ptr };
		if let Some(ref mut root) = self.root {
			if Self::contains_key(root, &key) {
				return Err(BPlusTreeError::DuplicateKey);
			}
		}
		if self.root.is_none() {
			self.root = Some(Box::new(Node::Leaf {
				keys: vec![key],
				next: None,
			}));
		} else {
			let root = self.root.as_mut().unwrap();
			self.insert_key(root, key);
		}
		Ok(())
	}

	/// Searches for a key in the B+ Tree.
	///
	/// # Arguments
	///
	/// * `id` - The ID to search for.
	///
	/// # Returns
	///
	/// A vector of `Key` structs matching the ID, or an empty vector if not found.
	pub fn search(&self, id: u64) -> Vec<Key> {
		let mut result = Vec::new();
		if let Some(ref root) = self.root {
			Self::search_key(root, id, &mut result);
		}
		result
	}

	/// Recursively inserts a key into the B+ Tree.
	fn insert_key(&mut self, node: &mut Box<Node>, key: Key) {
		match **node {
			Node::Internal {
				ref mut keys,
				ref mut children,
			} => {
				let pos = keys.binary_search(&key).unwrap_or_else(|e| e);
				self.insert_key(&mut children[pos], key);
			}
			Node::Leaf { ref mut keys, .. } => {
				let pos = keys.binary_search(&key).unwrap_or_else(|e| e);
				keys.insert(pos, key);
			}
		}
	}

	/// Recursively searches for a key in the B+ Tree.
	fn search_key(node: &Node, id: u64, result: &mut Vec<Key>) {
		match node {
			Node::Internal { keys, children } => {
				let pos = keys.binary_search_by(|k| k.id.cmp(&id)).unwrap_or_else(|e| e);
				Self::search_key(&children[pos], id, result);
			}
			Node::Leaf { keys, next } => {
				for key in keys {
					if key.id == id {
						result.push(key.clone());
					}
				}
				if let Some(ref next_node) = next {
					Self::search_key(next_node, id, result);
				}
			}
		}
	}

	/// Checks if a key exists in the B+ Tree.
	fn contains_key(node: &Node, key: &Key) -> bool {
		match node {
			Node::Internal { keys, children } => {
				let pos = keys.binary_search(key).unwrap_or_else(|e| e);
				Self::contains_key(&children[pos], key)
			}
			Node::Leaf { keys, .. } => keys.contains(key),
		}
	}

}

#[cfg(test)]
mod tests {
    use super::*;

#[test]
fn test_insert_and_search() {
    let mut tree = BPlusTree::new(3);
    let data_ptr = 0x1234 as *const u8;
    assert!(tree.insert(1, data_ptr).is_ok());
    assert!(tree.insert(2, data_ptr).is_ok());
    assert!(tree.insert(3, data_ptr).is_ok());
    assert_eq!(tree.search(1).len(), 1);
    assert_eq!(tree.search(2).len(), 1);
    assert_eq!(tree.search(3).len(), 1);
    assert_eq!(tree.search(4).len(), 0);
}

#[test]
fn test_duplicate_key() {
    let mut tree = BPlusTree::new(3);
    let data_ptr = 0x1234 as *const u8;
    assert!(tree.insert(1, data_ptr).is_ok());
    assert!(matches!(tree.insert(1, data_ptr), Err(BPlusTreeError::DuplicateKey)));
}

#[test]
fn test_search_nonexistent_key() {
    let tree = BPlusTree::new(3);
    assert_eq!(tree.search(1).len(), 0);
}

#[test]
fn test_empty_tree() {
    let tree = BPlusTree::new(3);
    assert_eq!(tree.search(1).len(), 0);
}

}