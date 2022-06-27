use crate::errors::Result;

pub trait Divider {
    type ITEM;
    fn divide(&self, src: &Option<Self::ITEM>) -> Result<(Option<Self::ITEM>, Option<Self::ITEM>)>;
}

pub trait Splitter {
    type ITEM;
    fn split(&self, src: &Option<Self::ITEM>) -> Result<Vec<Option<Self::ITEM>>>;
}
