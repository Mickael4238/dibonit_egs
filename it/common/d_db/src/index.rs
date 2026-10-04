//Db rust : créé moi un crate index.rs qui possède une structure de bPlusTree. Celle ci pourra avoir x clefs. Et y enfants. Fait une fonction qui pourra initialiser le nombre de clef et d'enfants. Chaques clef possède un id et une adresse (pointeur) vers de la donnée. Elle possède une fonction insert qui permet d'ajouter un id/pointeur de data dans le bPlusTree et un fonction search qui recherche un id dans le bPlusTree et qui retourne une liste de couple id/pointeur de data. avec tous les id trouvés. Tu ajoutera bien toutes les fonctions de tests nécessaires. Tout sera en anglais : variables fonctions commentaires. Respecte bien sur l'état de l'art du développement en rust.


//! A B+ Tree implementation in Rust.
//!
//! This crate provides a B+ Tree data structure optimized for in-memory and disk-based indexing.
//! The B+ Tree supports insertion and search operations, with configurable order (number of keys and children).


/*
<leaf adr = "0">
	<pref_leaf adr = ""/>
	<med_key = 0/>
	<children>
		<Internal>
			<min_key = 0/>
				<keys>
					<key col1= "TOTO" col2='21' ptrData = 'adrxxx'/>
					<key col1= "TOTO" col2='22' ptrData = 'adrxxx'/>
					<key col1= "TOTO" col2='23' ptrData = 'adrxxx'/>
					[200]
				</keys>
			<med_key = 0/>
			<children>
				<Internal>
					<min_key = 0/>
						<keys>
							<key col1= "TOTO" col2='21' ptrData = 'adrxxx'/>
							<key col1= "TOTO" col2='22' ptrData = 'adrxxx'/>
							<key col1= "TOTO" col2='23' ptrData = 'adrxxx'/>
							[200]
						</keys>
					<med_key = 0/>
					<children>

					</children>
					<max_key = 0/>
				</Internal>
			</children>
			<max_key = 0/>
		</Internal>
		<Internal>
			<min_key = 0/>
				<keys>
					<key col1= "TOTO" col2='21' ptrData = 'adrxxx'/>
					<key col1= "TOTO" col2='22' ptrData = 'adrxxx'/>
					<key col1= "TOTO" col2='23' ptrData = 'adrxxx'/>
					[...200]
				</keys>
			<med_key = 0/>
			<children>
					[...100]				
			</children>
			<max_key = 0/>
		</Internal>
		[
	</children>
	<max_key = 0/>
	<next_leaf adr = ""/>
</leaf>
*/

use std::fmt;
use d_err::{DErr, DErrMap};

/// A specialized `Result` type for operations in this module.
///
/// # Type Parameters
/// - `T`: The success type, wrapped in `Ok`.
/// - `DErr`: The full function path of an error and its root cause
type Re<T> = Result<T, DErr<Fe, Rc>>;
/*
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Key {
    pub col2: i32,
}

#[derive(Debug)]
pub struct KeyCondition{
	pub condition_key : Key,
	pub condition_operator: SearchOperator,
}*/

/// bPlusTree key associated with its data_ptr
///    pub key : Key,
//     pub data_ptr: *const u8,
#[derive(Debug, Clone)]
struct KeyPtr {
    key : Key,
    data_ptr: *const u8,
}
/// key : Option<Key>   => the index key
/// data_ptr: *const u8 => the data pointer
pub struct InsertedKeyPtr {
    pub key : Option<Key>,
    pub data_ptr: *const u8,
}

/// search_key : Key    		  => the searched key
/// search_pattern: SearchPattern => the searched pattern
#[derive(Debug, Clone,)]
pub struct KeySearch {
    pub search_key : Key,
    pub search_operator: SearchOperator,
	pub search_direction: SearchDirection,
}
#[derive(Debug)]
pub struct KeySearch2{
	pub condition_node : ConditionNode,
	pub search_operator: SearchOperator,
	pub search_direction: SearchDirection,
}

/*
impl KeySearch { //TODO TO REMOVE?
	pub fn match_key(&self, key : &Key) -> Re<bool> {
		Ok(match self.search_operator {
			SearchOperator::Equal				=> *key == self.search_key,
			SearchOperator::GreaterThan			=> *key >  self.search_key,
			SearchOperator::GreaterThanOrEquals	=> *key >= self.search_key,
			SearchOperator::LowerThan			=> *key <  self.search_key,
			SearchOperator::LowerThanOrEquals	=> *key <= self.search_key,
			SearchOperator::StartWith			=> *key == self.search_key, // TODO IMPLEMENT
			SearchOperator::Like				=> *key == self.search_key, // TODO IMPLEMENT
			SearchOperator::Regex				=> *key == self.search_key, // TODO IMPLEMENT
		} )
	}
}
/// ```rust
/// Enum representing a condition node used for constructing search queries.
///
/// This enum defines the structure of a condition node, which can represent
/// a basic key search, or a combination of key searches with logical operators.
/// It provides a flexible way to model search conditions in a hierarchical manner.
///
/// # Variants
///
/// * `Single(KeySearch)`
///     - Represents a single key search condition. The `KeySearch` type
///       encapsulates the specific search query details.
///
/// * `And(Box<ConditionNode>, Box<ConditionNode>)`
///     - Represents a logical AND operation between two condition nodes.
///       Both child nodes must evaluate to `true` for the entire condition to be met.
///       Stored as boxed variants to allow recursive definitions with heap allocation.
///
/// * `Or(Box<ConditionNode>, Box<ConditionNode>)`
///     - Represents a logical OR operation between two condition nodes.
///       At least one of the child nodes must evaluate to `true` for the condition to pass.
///       Stored as boxed variants to handle recursive definitions.
///
/// # Example
///
/// ```rust
/// use crate::ConditionNode;
/// use crate::KeySearch;
///
/// let single_condition = ConditionNode::Single(KeySearch::new("example_key"));
/// let and_condition = ConditionNode::And(
///     Box::new(single_condition),
///     Box::new(ConditionNode::Single(KeySearch::new("another_key"))),
/// );
/// let or_condition = ConditionNode::Or(
///     Box::new(and_condition),
///     Box::new(ConditionNode::Single(KeySearch::new("third_key"))),
/// );
///
/// println!("{:?}", or_condition);
/// ```
///
/// This would represent a condition where:
/// - The query matches if "example_key" AND "another_key" are true,
///   OR "third_key" is true.
///
/// # Notes
///
/// The usage of `Box` for nested `ConditionNode` types allows for recursive
/// data structures while avoiding stack overflow caused by excessive nesting.
/// ```
#[derive(Debug)]
pub enum ConditionNode {
	Single(KeyCondition),
	And(Box<ConditionNode>, Box<ConditionNode>),
	Or(Box<ConditionNode>, Box<ConditionNode>),
}
impl ConditionNode {
	/// ```rust
	/// Matches a provided key against specified conditions encapsulated in the `ConditionNode`.
	///
	/// # Parameters
	/// - `key`: A reference to the `Key` object to be matched against the condition.
	///
	/// # Returns
	/// - `Re<bool>`: A result containing `Ok(true)` or `Ok(false)` based on the condition evaluation,
	///    or an error if an issue occurs during the matching process.
	///
	/// # Matching Logic
	/// The method evaluates the condition based on the type of `ConditionNode`:
	///
	/// 1. **`ConditionNode::Single(key_search)`**:
	///    - Compares the provided key against the single search condition defined by the `key_search` object.
	///    - The specific operation for evaluation is determined by the `SearchOperator`:
	///      - `SearchOperator::Equal`: Checks for equality (`*key == key_search.search_key`).
	///      - `SearchOperator::GreaterThan`: Checks if the `key` is greater than `key_search.search_key`.
	///      - `SearchOperator::GreaterThanOrEquals`: Checks if the `key` is greater than or equal to `key_search.search_key`.
	///      - `SearchOperator::LowerThan`: Checks if the `key` is less than `key_search.search_key`.
	///      - `SearchOperator::LowerThanOrEquals`: Checks if the `key` is less than or equal to `key_search.search_key`.
	///      - `SearchOperator::StartWith`, `SearchOperator::Like`, `SearchOperator::Regex`: These are currently placeholders
	///         with equality checks and need proper implementations (`TODO` marked).
	///
	/// 2. **`ConditionNode::And(left, right)`**:
	///    - Evaluates the `key` with both the `left` and `right` conditions using a logical "AND".
	///    - If both conditions evaluate to `true`, the result is `true`. Otherwise, the result is `false`.
	///    - Errors during evaluation are mapped using `d_err_map` with specific error codes (`Fe::KeyMatch`).
	///
	/// 3. **`ConditionNode::Or(left, right)`**:
	///    - Evaluates the `key` with both the `left` and `right` conditions using a logical "OR".
	///    - If either condition evaluates to `true`, the result is `true`. If both are `false`, the result is `false`.
	///    - Errors during evaluation are mapped using `d_err_map` with specific error codes (`Fe::KeyMatch`).
	///
	/// # Errors
	/// - The function may return an error if an issue occurs while evaluating logical conditions within `And` or `Or` nodes.
	/// - Errors are mapped using `d_err_map` to provide additional error context.
	///
	/// # TODO
	/// - Implement logic for `SearchOperator::StartWith`, `SearchOperator::Like`, and `SearchOperator::Regex` cases
	///   as they currently default to basic equality checking.
	///
	/// # Examples
	/// ```rust
	/// let key = Key::new("sample_key");
	/// let condition = ConditionNode::Single(key_search_with_equal_operator);
	/// let result = condition.key_match(&key)?;
	/// assert_eq!(result, true);
	/// ```
	/// ```
	fn match_key(&self,key : &Key) ->  Re<bool> {
		match self {
			ConditionNode::Single(key_condition) => {
				Ok(match key_condition.condition_operator {
					SearchOperator::Equal				=> *key == key_condition.condition_key,
					SearchOperator::GreaterThan			=> *key >  key_condition.condition_key,
					SearchOperator::GreaterThanOrEquals	=> *key >= key_condition.condition_key,
					SearchOperator::LowerThan			=> *key <  key_condition.condition_key,
					SearchOperator::LowerThanOrEquals	=> *key <= key_condition.condition_key,
					SearchOperator::StartWith			=> *key == key_condition.condition_key, // TODO IMPLEMENT
					SearchOperator::Like				=> *key == key_condition.condition_key, // TODO IMPLEMENT
					SearchOperator::Regex				=> *key == key_condition.condition_key, // TODO IMPLEMENT
				})
			}
			ConditionNode::And(left, right) => {
				Ok(left.match_key(&key).d_err_map(Fe::MatchKey, 1)? && right.match_key(&key).d_err_map(Fe::MatchKey, 2)?)
			}
			ConditionNode::Or(left, right) => {
				Ok(left.match_key(&key).d_err_map(Fe::MatchKey, 3)? || right.match_key(&key).d_err_map(Fe::MatchKey, 4)?)
			}
		}
	}
	fn can_be_lower_or_equal_than_key(&self,key : &Key) ->  Re<bool> {
		match self {
			ConditionNode::Single(key_condition) => {
				Ok(match key_condition.condition_operator {
					SearchOperator::Equal				=> *key >= key_condition.condition_key,
					SearchOperator::GreaterThan			=> *key >= key_condition.condition_key,
					SearchOperator::GreaterThanOrEquals	=> *key >= key_condition.condition_key,
					SearchOperator::LowerThan			=> true,
					SearchOperator::LowerThanOrEquals	=> true,
					SearchOperator::StartWith			=> {
						// TODO IMPLEMENT
						*key == key_condition.condition_key
					},
					SearchOperator::Like				=> true,
					SearchOperator::Regex				=> true,
				})
			}
			ConditionNode::And(left, right) => {
				Ok(     left.can_be_lower_or_equal_than_key(&key).d_err_map(Fe::CanBeLowerOrEqualThanKey, 1)?
					&& right.can_be_lower_or_equal_than_key(&key).d_err_map(Fe::CanBeLowerOrEqualThanKey, 2)?)
			}
			ConditionNode::Or(left, right) => {
				Ok(     left.can_be_lower_or_equal_than_key(&key).d_err_map(Fe::CanBeLowerOrEqualThanKey, 3)?
					|| right.can_be_lower_or_equal_than_key(&key).d_err_map(Fe::CanBeLowerOrEqualThanKey, 4)?)
			}
		}
	}
	fn can_be_greater_or_equal_than_key(&self,key : &Key) ->  Re<bool> {
			match self {
				ConditionNode::Single(key_condition) => {
					Ok(match key_condition.condition_operator {
						SearchOperator::Equal				=> *key <=  key_condition.condition_key,
						SearchOperator::GreaterThan			=> true,
						SearchOperator::GreaterThanOrEquals	=> true,
						SearchOperator::LowerThan			=> *key <=  key_condition.condition_key,
						SearchOperator::LowerThanOrEquals	=> *key <= key_condition.condition_key,
						SearchOperator::StartWith			=> {
							// TODO IMPLEMENT
							*key == key_condition.condition_key
						},
						SearchOperator::Like				=> true,
						SearchOperator::Regex				=> true,
					})
				}
				ConditionNode::And(left, right) => {
					Ok(     left.can_be_greater_or_equal_than_key(&key).d_err_map(Fe::CanBeGreaterOrEqualThanKey, 1)?
						&& right.can_be_greater_or_equal_than_key(&key).d_err_map(Fe::CanBeGreaterOrEqualThanKey, 2)?)
				}
				ConditionNode::Or(left, right) => {
					Ok(     left.can_be_greater_or_equal_than_key(&key).d_err_map(Fe::CanBeGreaterOrEqualThanKey, 3)?
						|| right.can_be_greater_or_equal_than_key(&key).d_err_map(Fe::CanBeGreaterOrEqualThanKey, 4)?)
				}
			}
		}
	fn can_be_lower_than_key(&self,key : &Key) ->  Re<bool> {
			match self {
				ConditionNode::Single(key_condition) => {
					Ok(match key_condition.condition_operator {
						SearchOperator::Equal				=> *key > key_condition.condition_key,
						SearchOperator::GreaterThan			=> *key >  key_condition.condition_key,
						SearchOperator::GreaterThanOrEquals	=> *key > key_condition.condition_key,
						SearchOperator::LowerThan			=> true,
						SearchOperator::LowerThanOrEquals	=> true,
						SearchOperator::StartWith			=> {
							// TODO IMPLEMENT
							*key == key_condition.condition_key
						},
						SearchOperator::Like				=> true,
						SearchOperator::Regex				=> true,
					})
				}
				ConditionNode::And(left, right) => {
					Ok(     left.can_be_lower_than_key(&key).d_err_map(Fe::CanBeLowerThanKey, 1)?
						&& right.can_be_lower_than_key(&key).d_err_map(Fe::CanBeLowerThanKey, 2)?)
				}
				ConditionNode::Or(left, right) => {
					Ok(     left.can_be_lower_than_key(&key).d_err_map(Fe::CanBeLowerThanKey, 3)?
						|| right.can_be_lower_than_key(&key).d_err_map(Fe::CanBeLowerThanKey, 4)?)
				}
			}
		}
	fn can_be_greater_than_key(&self,key : &Key) ->  Re<bool> {
		// est ce qyu'il est possible de trouver des ID plus grand que key qui pourront remplire la condition?
		match self {
			ConditionNode::Single(key_condition) => {
				Ok(match key_condition.condition_operator {
					SearchOperator::Equal				=> *key <  key_condition.condition_key,
					SearchOperator::GreaterThan			=> true,
					SearchOperator::GreaterThanOrEquals	=> true,
					SearchOperator::LowerThan			=> *key <  key_condition.condition_key,
					SearchOperator::LowerThanOrEquals	=> *key < key_condition.condition_key,
					SearchOperator::StartWith			=> {
						// TODO IMPLEMENT
						*key == key_condition.condition_key
					},
					SearchOperator::Like				=> true,
					SearchOperator::Regex				=> true,
				})
			}
			ConditionNode::And(left, right) => {
				Ok(     left.can_be_greater_than_key(&key).d_err_map(Fe::CanBeGreaterThanKey, 1)?
					&& right.can_be_greater_than_key(&key).d_err_map(Fe::CanBeGreaterThanKey, 2)?)
			}
			ConditionNode::Or(left, right) => {
				Ok(     left.can_be_greater_than_key(&key).d_err_map(Fe::CanBeGreaterThanKey, 3)?
					|| right.can_be_greater_than_key(&key).d_err_map(Fe::CanBeGreaterThanKey, 4)?)
			}
		}
	}
}*/
/// Represents a node in the B+ Tree.
#[derive(Debug)]
enum Node {
    /// Internal node containing keys and child nodes.
    Internal {
		max_key 	: Key,
        keys		: Vec<KeyPtr>, 			    // contrains keys where  med_key <=  keys  <= max_key
		med_key 	: Key,
        children	: Vec<Box<Node>>,	    // contrains keys where  min_key <= keys  < med_key
		min_key 	: Key,
    },
    /// Leaf node containing child nodes and optional previous/next leaf pointer.
    Leaf {
		prev_leaf	: Option<Box<Node>>, // serai plutot une edresse dans le fichier
		max_key 	: Key,
        children	: Vec<Box<Node>>,	    // contrains  min_key <= keys  <= max_key
		min_key		: Key,
		next_leaf	: Option<Box<Node>>  // serai plutot une edresse dans le fichier
    },
}

impl Node {
	fn recursive_search(&self,key_search : &mut KeySearch,  result : &mut Vec<KeyPtr>) -> Re<bool> {
		match self {
			Node::Internal{max_key, keys,med_key,children, min_key, ..} => {
				match key_search.search_direction {
					SearchDirection::Descending =>{
						if !self.dsc_search_own(max_key,keys, med_key, key_search,result).d_err_map(Fe::RecursiveSearch, 1)?{
							return Ok(false);
						}
						self.dsc_search_child(med_key,children, min_key, key_search,result).d_err_map(Fe::RecursiveSearch, 2)
					},
					SearchDirection::Ascending => {
						if !self.asc_search_child(med_key,children, min_key, key_search,result).d_err_map(Fe::RecursiveSearch, 3)?{
							return Ok(false);
						}
						self.asc_search_own(max_key,keys, med_key, key_search,result).d_err_map(Fe::RecursiveSearch, 4)
					},
				}
			},
			Node::Leaf{prev_leaf, max_key,children,min_key,next_leaf, ..} => {
				match key_search.search_direction {
					SearchDirection::Descending => {
						if self.dsc_search_child(max_key, children,min_key, key_search,result).d_err_map(Fe::RecursiveSearch, 5)?{
							if prev_leaf.is_none() {return Ok(false)}
							prev_leaf.as_ref().unwrap().recursive_search(key_search,result).d_err_map(Fe::RecursiveSearch, 6)
						}else{
							Ok(false)
						}
					},
					SearchDirection::Ascending  => {
						if self.asc_search_child(max_key, children,min_key, key_search,result).d_err_map(Fe::RecursiveSearch, 7)?{
							if next_leaf.is_none() {return Ok(false)}
							next_leaf.as_ref().unwrap().recursive_search(key_search,result).d_err_map(Fe::RecursiveSearch, 8)
						}else{
							Ok(false)
						}
					},
				}
			}
		}
	}
	fn dsc_search_own (& self, max_key : &Key, keys : &Vec<KeyPtr>, med_key : &Key, key_search: &mut KeySearch, result : &mut Vec<KeyPtr>) -> Re<bool> {
		if !self.can_be_lte_max(max_key, key_search).d_err_map(Fe::DscSearch, 1)? {
			return Ok(false);
		}
		if self.can_be_gte_med(med_key, key_search).d_err_map(Fe::DscSearch, 1)?{
			for key in keys.iter().rev(){
				if key_search.match_key(&key.key).d_err_map(Fe::DscSearch, 1)? {
					result.push(key.clone());
				}
				if !self.can_be_lt_max(&key.key, key_search).d_err_map(Fe::DscSearch, 1)? {
					return Ok(false);
				}
			}
		}
		return Ok(true);
	}
	// TODO check CLONES can be passed as ref.
	fn dsc_search_child (& self,med_key : &Key,children	: &Vec<Box<Node>>,min_key : &Key,key_search : &mut KeySearch,  result : &mut Vec<KeyPtr>) -> Re<bool> {
		if !self.can_be_lt_med (med_key, &mut key_search.clone()).d_err_map(Fe::DscSearch, 1)?{
			return Ok(false);
		}
		if self.can_be_gte_min(& min_key.clone(),&mut key_search.clone()).d_err_map(Fe::DscSearch, 1)?{
			//TODO search in children
		}
		self.can_be_lt_min (min_key, key_search).d_err_map(Fe::DscSearch, 1)
	}
	fn asc_search_child (& self,med_key : &Key,children	: &Vec<Box<Node>>,min_key : &Key,key_search : &mut KeySearch,  result : &mut Vec<KeyPtr>) -> Re<bool> {
		if !self.can_be_gte_min(min_key,&mut key_search.clone()).d_err_map(Fe::AscSearch, 1)? {
			return Ok(false);
		}
		if self.can_be_lte_med(med_key, key_search).d_err_map(Fe::AscSearch, 1)?{
			// TODO search in child
		}
		return Ok(true);
	}
	fn asc_search_own (& self,max_key : &Key,keys : &Vec<KeyPtr>,med_key : &Key,key_search : &mut KeySearch,  result : &mut Vec<KeyPtr>) -> Re<bool> {
		if !self.can_be_gte_med (med_key,&mut key_search.clone()).d_err_map(Fe::AscSearch, 1)?{
			return Ok(false);
		}
		if self.can_be_lte_max(&max_key.clone(),&mut key_search.clone()).d_err_map(Fe::AscSearch, 1)?{
			for key in keys.iter(){
				if key_search.match_key(&key.key).d_err_map(Fe::DscSearch, 1)? {
					result.push(key.clone());
				}
				if !self.can_be_gt_med(&key.key, key_search).d_err_map(Fe::DscSearch, 1)? {
					return Ok(false);
				}
			}
		}
		self.can_be_gt_max (max_key, key_search).d_err_map(Fe::AscSearch, 1)
	}
	// TODO MOVE in keySearch? and remove max min med...
	fn can_be_lte_max(& self,max_key : &Key,key_search : &mut KeySearch) -> Re<bool> {
		Ok(true) //TODO TO IMPLEMENT
	}
	fn can_be_gte_med(& self,med_key : &Key,key_search : &mut KeySearch) -> Re<bool> {
		Ok(true) //TODO TO IMPLEMENT
	}
	fn can_be_lte_med(& self,med_key : &Key,key_search : &mut KeySearch) -> Re<bool> {
		Ok(true) //TODO TO IMPLEMENT
	}
	fn can_be_gte_min(& self,min_key : &Key,key_search : &mut KeySearch) -> Re<bool> {
		Ok(true) //TODO TO IMPLEMENT
	}
	fn can_be_lt_min(& self,min_key : &Key,key_search : &mut KeySearch) -> Re<bool> {
		Ok(true) //TODO TO IMPLEMENT
	}
	fn can_be_lt_med(& self,med_key : &Key,key_search : &mut KeySearch) -> Re<bool> {
		Ok(true) //TODO TO IMPLEMENT
	}
	fn can_be_gt_med(& self,med_key : &Key,key_search : &mut KeySearch) -> Re<bool> {
		Ok(true) //TODO TO IMPLEMENT
	}
	fn can_be_gt_max(& self,max_key : &Key,key_search : &mut KeySearch) -> Re<bool> {
		Ok(true) //TODO TO IMPLEMENT
	}
	///
	fn match_max_key(& self,max_key : Key,search_key : KeySearch) -> Re<SearchResult> {
		if  search_key.match_key(max_key).d_err_map(Fe::MatchMaxKey, 1)? {
			return Ok(SearchResult::Ok);
		}
		match search_key.search_direction {
			SearchDirection::Ascending  => {Ok(SearchResult::KoAndQuit)}    ,
			SearchDirection::Descending => {Ok(SearchResult::KoAndContinue)},
		}
	}
	fn match_min_key(& self,min_key : Key,search_key : KeySearch) -> Re<SearchResult> {
		if  search_key.match_key(min_key).d_err_map(Fe::MatchMinKey, 1)? {
			return Ok(SearchResult::Ok);
		}
		match search_key.search_direction {
			SearchDirection::Ascending  => {Ok(SearchResult::KoAndContinue)},
			SearchDirection::Descending => {Ok(SearchResult::KoAndQuit)}    ,
		}
	}
}

enum SearchResult{
	Ok,
	KoAndContinue,
	KoAndQuit,
}

#[derive(Debug,Clone)]
enum SearchDirection{
	Ascending,
	Descending,
}

pub struct BPlusTree {
    root: Option<Box<Node>>,
    max_keys_per_node: u8,
    max_child_per_node: u8,
	starting_address : u32
}


#[derive(Debug)]
/// RootCause of the error
pub enum Rc {
	KeyNotFound,
	DuplicateKey,
	IncorrectMemoryAddress,
	MaxNumberOfKeyInvalid,
	MaxNumberOfChildInInvalid,
	SearchPatternInvalid,
	NoInsertedKeyPtrProvided,
	NoDataPtr,
}

impl fmt::Display for Rc {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Rc::KeyNotFound 				=> write!(f, "Key not found in the B+ Tree"),
			Rc::DuplicateKey 				=> write!(f, "Duplicate key in the B+ Tree"),
			Rc::IncorrectMemoryAddress  	=> write!(f, "Physical memory address incorrect"),
			Rc::MaxNumberOfKeyInvalid 		=> write!(f, "Maximum number of keys is invalid"),
			Rc::MaxNumberOfChildInInvalid 	=> write!(f, "Maximum number of child is invalid"),
			Rc::SearchPatternInvalid 		=> write!(f, "Incorrect search pattern provided"),
			Rc::NoInsertedKeyPtrProvided 	=> write!(f, "No inserted Key provided"),
			Rc::NoDataPtr 					=> write!(f, "No DataPtr provided"),
		}
	}
}

#[derive(Debug)]
/// Function where the Error transit
enum Fe {
	New,
	Insert,
	MatchMaxKey,
	MatchMinKey,
	RecursiveSearch,
	AscSearch,
	DscSearch,
	MatchKey,
	CanBeGreaterThanKey,
	CanBeLowerThanKey,
	CanBeGreaterOrEqualThanKey,
	CanBeLowerOrEqualThanKey,
}

impl BPlusTree {
    /// Creates a new B+ Tree.
    ///
    /// # Arguments
    ///
    /// * `max_n_keys_per_node` - Maximum number of keys per node (must be > 0).
    /// * `max_child_per_node` - Maximum number of children per node (must be > 0).
    /// * `starting_address` - Starting memory address.
    ///
    /// # Returns
    ///
    /// `Result<Self, DErr>` - A new tree or an DError if arguments are invalid.
    pub fn new(
		max_keys_per_node: u8,
        max_child_per_node: u8,
        starting_address: u32
	) -> Re<Self> {
        if max_keys_per_node == 0 {
			return Err(DErr::RootError(Fe::New, 1, Rc::MaxNumberOfKeyInvalid));
		}		
		else if max_child_per_node == 0 {
			return Err(DErr::RootError(Fe::New, 1, Rc::MaxNumberOfChildInInvalid));
        }
        Ok(BPlusTree {
            root: None,
			max_keys_per_node,
            max_child_per_node,
            starting_address,
        })
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
	/// `Ok((key))` if the insertion was successful, or a `BPlusTreeError` if the key already exists.
	pub fn insert(&mut self, inserted_key_ptr: InsertedKeyPtr)-> Re<Key>  {
		/*if inserted_key_ptr.is_none() { 		// hey we are missing something here
			return Err(DErr::RootError(Fe::Insert, 1, Rc::NoInsertedKeyPtrProvided));
		}
		if inserted_key_ptr.data_ptr.is_none() {	// hey we are missing something here
			return Err(DErr::RootError(Fe::Insert, 1, Rc::NoDataPtr));
		}*/
		if self.root.is_none() { 				// we will create it
			let key_ptr : KeyPtr = {
				if inserted_key_ptr.key.is_none(){
					KeyPtr{
						key:self.generate_first_key().d_err_map(Fe::Insert, 3)?,
						data_ptr: inserted_key_ptr.data_ptr,
					}
				}else{
					KeyPtr{
						key:inserted_key_ptr.key.unwrap(),
						data_ptr: inserted_key_ptr.data_ptr,
					}
				}
			};

			// creating the root
			// creating its 1st child
			let mut children =Vec::new();
			children.push(Box::new(Node::Internal {
				max_key 	: key_ptr.key.clone(),
				keys		: vec![key_ptr], 			    // contrains keys where  med_key <=  keys  <= max_key
				med_key 	: key_ptr.key.clone(),
				children	:  Vec::new(),	       	        // contrains keys where  min_key <= keys  < med_key
				min_key 	: key_ptr.key.clone()
			}));
			self.root = Some(Box::new(Node::Leaf {
				prev_leaf	: None,
				max_key 	: key_ptr.key.clone(),
				children,
				min_key		: key_ptr.key.clone(),
				next_leaf	: None
			}));
		}else { 		//root already exist
			let key_ptr : KeyPtr = {
				if inserted_key_ptr.key.is_none(){
					KeyPtr{
						key:self.generate_next_key().d_err_map(Fe::Insert, 4)?,
						data_ptr: inserted_key_ptr.data_ptr,
					}
				}else{
					self.search(KeySearch{
						// TODO could be replaced by search and insert, to avoid double parsing
						search_key: inserted_key_ptr.key.clone().unwrap(),
						search_pattern: SearchPattern::Equal,
						search_direction: SearchDirection::Descending,
					})?;
					// todo MAP result OK with data to Error key already exit
					KeyPtr{
						key:inserted_key_ptr.key.unwrap(),
						data_ptr: inserted_key_ptr.data_ptr,
					}
				}
			};
		}
		//TODO maybe move the if self.root.is_none() {  inside the building of keyPtr??
		// inserting the key in the nodes
		Ok(( key))
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
	pub fn search(&self, key_search: KeySearch) -> Re<Vec<KeyPtr>> {
		let mut result = Vec::new();
		if self.root.is_none() {
			return Ok(result);
		}
		self.root.unwrap().recursive_search(key_search,&mut result);

		return Ok(result);
	}
	fn generate_first_key(&mut self) -> Re<Key> {
		Ok(Key {
			col2: 0,
		})
	}
	fn generate_next_key(&mut self) -> Re<Key> {
		Ok(Key {
			col2: 0,
		})
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