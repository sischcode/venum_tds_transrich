use venum::venum::Value;

use crate::errors::Result;

pub trait Split {
    fn split(&self, src: &Option<Value>) -> Result<(Option<Value>, Option<Value>)>;
}

pub trait SplitN {
    fn split_n(&self, src: &Option<Value>) -> Result<Vec<Option<Value>>>;
}

// concat | join | template
pub enum MergeType {
    Concat,
    Join(&'static str),
    Template(String),
}
pub trait Merge {
    fn merge(&self, src_a: &Option<Value>, src_b: &Option<Value>) -> Result<Option<Value>>;
}

pub trait MergeN {
    fn merge(&self, src: Vec<&Option<Value>>) -> Result<Option<Value>>;
}
