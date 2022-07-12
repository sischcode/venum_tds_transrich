use crate::errors::Result;

use super::value::Split;

pub trait SplitUsing<D: Split> {
    type ITEM;

    fn split_using(
        &self,
        split_impl: &D,
        dst_left: &mut Self::ITEM,
        dst_right: &mut Self::ITEM,
    ) -> Result<()>;
}

// pub trait SplitUsing2 {
//     type ITEM;
//     type SPLITIMPL: Split;

//     fn split_using(
//         &self,
//         split_impl: &Self::SPLITIMPL,
//         dst_left: &mut Self::ITEM,
//         dst_right: &mut Self::ITEM,
//     ) -> Result<()>;
// }
