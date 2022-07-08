use crate::errors::Result;

pub trait Divide {
    type ITEM;
    fn divide(&self, src: &Option<Self::ITEM>) -> Result<(Option<Self::ITEM>, Option<Self::ITEM>)>;
}

pub trait Split {
    type ITEM;
    fn split(&self, src: &Option<Self::ITEM>) -> Result<Vec<Option<Self::ITEM>>>;
}
