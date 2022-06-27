use crate::errors::Result;

use super::shared::Divider;

pub trait DivideUsing<S: Divider> {
    type ITEM;

    fn divide_using(
        &self,
        divider: &S,
        dst_left: &mut Self::ITEM,
        dst_right: &mut Self::ITEM,
    ) -> Result<()>;
}
