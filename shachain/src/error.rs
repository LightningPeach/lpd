use std::{
    error::Error,
    fmt,
};

use store_tree::Leaf;
use util::LeafIndex;

#[derive(Debug)]
pub struct CanNotDeriveTreeElement {
    from_index: LeafIndex,
    to_index: LeafIndex,
}

impl fmt::Display for CanNotDeriveTreeElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "can't derive tree's element, from_index: {:?}, to_index: {:?}\n",
               self.from_index, self.to_index,
        )
    }
}

impl Error for CanNotDeriveTreeElement {}

impl CanNotDeriveTreeElement {
    pub fn new(from_index: LeafIndex, to_index: LeafIndex) -> Self {
        Self {
            from_index,
            to_index,
        }
    }
}

#[derive(Debug)]
pub struct InvalidLeaf(Leaf);

impl fmt::Display for InvalidLeaf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid leaf, index: {:?}, value: {:?}\n",
               self.0.get_index(), self.0.get_value(),
        )
    }
}

impl Error for InvalidLeaf {}

impl InvalidLeaf {
    pub fn new(leaf: Leaf) -> Self {
        InvalidLeaf(leaf)
    }
}

#[derive(Debug)]
pub struct CanNotFindElementByIndex(LeafIndex);

impl fmt::Display for CanNotFindElementByIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "can't find element by index, index: {:?}\n", self.0)
    }
}

impl Error for CanNotFindElementByIndex {}

impl CanNotFindElementByIndex {
    pub fn new(index: LeafIndex) -> Self {
        CanNotFindElementByIndex(index)
    }
}

#[derive(Debug)]
pub enum AddLeafError {
    Derive(CanNotDeriveTreeElement),
    InvalidLeaf(InvalidLeaf),
}

impl fmt::Display for AddLeafError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AddLeafError::Derive(e) => write!(f, "{}", e),
            AddLeafError::InvalidLeaf(e) => write!(f, "{}", e),
        }
    }
}

impl Error for AddLeafError {
    fn cause(&self) -> Option<&Error> {
        match *self {
            AddLeafError::Derive(ref err) => Some(err),
            AddLeafError::InvalidLeaf(ref err) => Some(err),
        }
    }
}

impl From<CanNotDeriveTreeElement> for AddLeafError {
    fn from(e: CanNotDeriveTreeElement) -> Self {
        AddLeafError::Derive(e)
    }
}

impl From<InvalidLeaf> for AddLeafError {
    fn from(e: InvalidLeaf) -> Self {
        AddLeafError::InvalidLeaf(e)
    }
}

#[derive(Debug)]
pub enum LookupError {
    CanNotDeriveTreeElement(CanNotDeriveTreeElement),
    CanNotFindElementByIndex(CanNotFindElementByIndex),
}

impl fmt::Display for LookupError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LookupError::CanNotDeriveTreeElement(e) => write!(f, "{}", e),
            LookupError::CanNotFindElementByIndex(e) => write!(f, "{}", e),
        }
    }
}

impl From<CanNotDeriveTreeElement> for LookupError {
    fn from(e: CanNotDeriveTreeElement) -> Self {
        LookupError::CanNotDeriveTreeElement(e)
    }
}

impl From<CanNotFindElementByIndex> for LookupError {
    fn from(e: CanNotFindElementByIndex) -> Self {
        LookupError::CanNotFindElementByIndex(e)
    }
}