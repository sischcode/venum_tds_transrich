use crate::errors::Result;

use super::shared::Divider;

pub trait DivideUsing<D: Divider> {
    type ITEM;

    fn divide_using(
        &self,
        divider: &D,
        dst_left: &mut Self::ITEM,
        dst_right: &mut Self::ITEM,
    ) -> Result<()>;
}
