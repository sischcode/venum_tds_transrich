use venum::venum::{Value, ValueName};
use venum_tds::traits::{VDataContainer, VDataContainerItem};

use crate::{
    container::{DeleteItemAtIdx, SplitItemAtIdx},
    container_transrich_pass::TransrichPass,
    errors::Result,
    errors::VenumTdsTransRichError,
    traits::{container::TransrichContainerInplace, item::SplitUsing, value::Split},
    value_splitting::ValueStringSeparatorCharSplit,
};

use super::jsonconf::{SplitterType, TransformEnrichPassConfig, TransformerConfig};

impl<C, CI> TryFrom<TransformerConfig> for Vec<Box<dyn TransrichContainerInplace<C>>>
where
    CI: VDataContainerItem + SplitUsing<ITEM = CI> + Default,
    C: VDataContainer<ITEM = CI>,
{
    type Error = VenumTdsTransRichError;

    fn try_from(tc: TransformerConfig) -> Result<Self> {
        match tc {
            TransformerConfig::DeleteItems { cfg } => {
                let v: Vec<Box<dyn TransrichContainerInplace<C>>> = Vec::with_capacity(cfg.len());
                for idx in cfg {
                    v.push(Box::new(DeleteItemAtIdx { 0: idx }));
                }
                Ok(v)
            }
            TransformerConfig::SplitItem { cfg } => {
                let v: Vec<Box<dyn TransrichContainerInplace<C>>> = Vec::with_capacity(1);
                match cfg.spec {
                    SplitterType::SeparatorChar { char } => {
                        let siai = SplitItemAtIdx {
                            delete_source_item: cfg.delete_after_split,
                            idx: cfg.idx,
                            splitter: ValueStringSeparatorCharSplit {
                                sep_char: char,
                                split_none: true, // TODO: config
                            },
                            target_left: (
                                Value::from(cfg.target_left.target_type),
                                cfg.target_left.idx,
                                String::from("TODO"), // TODO
                            ),
                            target_right: (
                                Value::from(cfg.target_right.target_type),
                                cfg.target_right.idx,
                                String::from("TODO"), // TODO
                            ),
                        };
                        v.push(Box::new(siai));
                    }
                    SplitterType::Pattern { pattern } => todo!(),
                };
                Ok(v)
            }
            TransformerConfig::AddItem { cfg } => todo!(),
        }
    }
}

// impl<C: VDataContainer> TryFrom<TransformEnrichPassConfig> for TransrichPass<C> {
//     type Error = VenumTdsTransRichError;

//     fn try_from(tepc: TransformEnrichPassConfig) -> Result<Self> {
//         tepc.transformers.iter_mut().map(|tec| {

//         })

//         todo!()
//     }
// }
