use venum_tds::traits::VDataContainer;

use crate::errors::Result;

pub trait TransrichContainerInplace<C: VDataContainer> {
    fn apply(&self, container: &mut C) -> Result<()>;
}
