use crate::errors::Result;
use crate::traits::value::*;

pub trait DivideBy<S, T>
where
    S: DivideValue,
{
    fn divide_by(&self, splitter_impl: &S, dst_left: &mut T, dst_right: &mut T) -> Result<()>;
}
