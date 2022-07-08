use crate::errors::Result;

use super::shared::Divide;

pub trait DivideUsing<D: Divide> {
    type ITEM;

    fn divide_using(
        &self,
        divider: &D,
        dst_left: &mut Self::ITEM,
        dst_right: &mut Self::ITEM,
    ) -> Result<()>;
}
