use serde::Deserialize;
use venum::venum::ValueName;

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "name", rename_all = "camelCase")]
pub enum SplitterType {
    SeparatorChar { char: char },
    Pattern { pattern: String },
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ItemTargetConfig {
    pub idx: usize,
    pub header: Option<String>,
    pub target_type: ValueName,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SplitItemConfig {
    pub idx: usize,
    pub spec: SplitterType,
    pub delete_after_split: bool,
    pub target_left: ItemTargetConfig,
    pub target_right: ItemTargetConfig,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum RuntimeValue {
    CurrentDateTimeUTC,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "name", rename_all = "camelCase")]
pub enum AddItemType {
    Meta { key: String },
    Static { value: String },
    Runtime { rt_value: RuntimeValue },
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AddItemConfig {
    pub spec: AddItemType,
    pub target: ItemTargetConfig,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TransformerConfig {
    DeleteItems { cfg: Vec<usize> },
    SplitItem { cfg: SplitItemConfig },
    AddItem { cfg: AddItemConfig },
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct OrderItemsEntry {
    pub from: usize,
    pub to: usize,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TransformEnrichPassConfig {
    pub comment: Option<String>,
    pub transformers: Vec<TransformerConfig>,
    pub order_items: Option<Vec<OrderItemsEntry>>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConfigRoot {
    pub passes: Vec<TransformEnrichPassConfig>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transformer_config_delete_items() {
        let data = r#"
        {
            "type": "deleteItems",
            "cfg": [0, 1]
        }
        "#;
        assert_eq!(
            TransformerConfig::DeleteItems {
                cfg: vec![0_usize, 1_usize]
            },
            serde_json::from_str(data).expect("could not deserialize ")
        )
    }

    #[test]
    fn test_transformer_config_split_item_sep_char() {
        let data = r#"
        {
            "type": "splitItem",
            "cfg": {
                "idx": 2,
                "spec": {
                    "name": "separatorChar",
                    "char": ";"
                },
                "deleteAfterSplit": true,
                "targetLeft": {
                    "idx": 10,
                    "header": "some_float32_left",
                    "targetType": "Float32"
                },
                "targetRight": {
                    "idx": 11,
                    "header": "some_string_right",
                    "targetType": "String"
                }
            }
        }
        "#;
        assert_eq!(
            TransformerConfig::SplitItem {
                cfg: SplitItemConfig {
                    idx: 2,
                    spec: SplitterType::SeparatorChar { char: ';' },
                    delete_after_split: true,
                    target_left: ItemTargetConfig {
                        header: Some(String::from("some_float32_left")),
                        idx: 10_usize,
                        target_type: ValueName::Float32,
                    },
                    target_right: ItemTargetConfig {
                        header: Some(String::from("some_string_right")),
                        idx: 11_usize,
                        target_type: ValueName::String,
                    },
                }
            },
            serde_json::from_str(data).expect("could not deserialize ")
        )
    }

    #[test]
    fn test_transformer_config_split_item_regex_pattern() {
        let data = r#"
        {
            "type": "splitItem",
            "cfg": {
                "idx": 2,
                "spec": {
                    "name": "pattern",
                    "pattern": "(\\d+\\.\\d+) \\(([[:alpha:]].+)\\)"
                },
                "deleteAfterSplit": true,
                "targetLeft": {
                    "idx": 10,
                    "header": "some_float32_left",
                    "targetType": "Float32"
                },
                "targetRight": {
                    "idx": 11,
                    "header": "some_string_right",
                    "targetType": "String"
                }
            }
        }
        "#;
        assert_eq!(
            TransformerConfig::SplitItem {
                cfg: SplitItemConfig {
                    idx: 2,
                    spec: SplitterType::Pattern {
                        pattern: String::from("(\\d+\\.\\d+) \\(([[:alpha:]].+)\\)"),
                    },
                    delete_after_split: true,
                    target_left: ItemTargetConfig {
                        header: Some(String::from("some_float32_left")),
                        idx: 10_usize,
                        target_type: ValueName::Float32,
                    },
                    target_right: ItemTargetConfig {
                        header: Some(String::from("some_string_right")),
                        idx: 11_usize,
                        target_type: ValueName::String,
                    },
                }
            },
            serde_json::from_str(data).expect("could not deserialize ")
        )
    }

    #[test]
    fn test_transformer_config_add_item_static() {
        let data = r#"
        {
            "type": "addItem",
            "cfg": {
                "spec": {
                    "name": "static",
                    "value": "Europe"
                },
                "target": {
                    "idx": 12,
                    "header": "Region",
                    "targetType": "String"
                }
            }
        }
        "#;
        assert_eq!(
            TransformerConfig::AddItem {
                cfg: AddItemConfig {
                    spec: AddItemType::Static {
                        value: String::from("Europe")
                    },
                    target: ItemTargetConfig {
                        header: Some(String::from("Region")),
                        idx: 12_usize,
                        target_type: ValueName::String
                    }
                }
            },
            serde_json::from_str(data).expect("could not deserialize ")
        )
    }

    #[test]
    pub fn test_order_items() {
        let data = r#"
        [
            { "from": 3, "to": 0 },
            { "from": 4, "to": 1 },
            { "from": 10, "to": 2 }
        ]
        "#;
        assert_eq!(
            vec![
                OrderItemsEntry {
                    from: 3_usize,
                    to: 0_usize
                },
                OrderItemsEntry {
                    from: 4_usize,
                    to: 1_usize
                },
                OrderItemsEntry {
                    from: 10_usize,
                    to: 2_usize
                }
            ],
            serde_json::from_str::<Vec<OrderItemsEntry>>(data).expect("could not deserialize ")
        )
    }

    #[test]
    fn test_transform_enrich_pass_config() {
        let data = r#"
        {
            "comment": "pass1",
            "transformers": [{
                "type": "deleteItems",
                "cfg": [0, 1]
            }, {
                "type": "splitItem",
                "cfg": {
                    "idx": 2,
                    "spec": {
                        "name": "separatorChar",
                        "char": ";"
                    },
                    "deleteAfterSplit": true,
                    "targetLeft": {
                        "idx": 10,
                        "header": "some_float32_left",
                        "targetType": "Float32"
                    },
                    "targetRight": {
                        "idx": 11,
                        "header": "some_string_right",
                        "targetType": "String"
                    }
                }
            }],
            "orderItems": [
                { "from": 3, "to": 0 },
                { "from": 4, "to": 1 },
                { "from": 10, "to": 2 }
            ]
        }
        "#;

        assert_eq!(
            TransformEnrichPassConfig {
                comment: Some(String::from("pass1")),
                transformers: vec![
                    TransformerConfig::DeleteItems {
                        cfg: vec![0_usize, 1_usize]
                    },
                    TransformerConfig::SplitItem {
                        cfg: SplitItemConfig {
                            idx: 2,
                            spec: SplitterType::SeparatorChar { char: ';' },
                            delete_after_split: true,
                            target_left: ItemTargetConfig {
                                header: Some(String::from("some_float32_left")),
                                idx: 10_usize,
                                target_type: ValueName::Float32,
                            },
                            target_right: ItemTargetConfig {
                                header: Some(String::from("some_string_right")),
                                idx: 11_usize,
                                target_type: ValueName::String,
                            },
                        }
                    }
                ],
                order_items: Some(vec![
                    OrderItemsEntry {
                        from: 3_usize,
                        to: 0_usize
                    },
                    OrderItemsEntry {
                        from: 4_usize,
                        to: 1_usize
                    },
                    OrderItemsEntry {
                        from: 10_usize,
                        to: 2_usize
                    }
                ]),
            },
            serde_json::from_str(data).expect("could not deserialize ")
        )
    }

    #[test]
    fn test_config_root_config() {
        let data = r#"
        { 
            "passes": [{
                "comment": "pass1",
                "transformers": [{
                    "type": "deleteItems",
                    "cfg": [0, 1]
                }, {
                    "type": "splitItem",
                    "cfg": {
                        "idx": 2,
                        "spec": {
                            "name": "separatorChar",
                            "char": ";"
                        },
                        "deleteAfterSplit": true,
                        "targetLeft": {
                            "idx": 10,
                            "header": "some_float32_left",
                            "targetType": "Float32"
                        },
                        "targetRight": {
                            "idx": 11,
                            "header": "some_string_right",
                            "targetType": "String"
                        }
                    }
                }],
                "orderItems": [
                    { "from": 3, "to": 0 },
                    { "from": 4, "to": 1 },
                    { "from": 10, "to": 2 }
                ]
            }, {
                "comment": "pass2",
                "transformers": [{
                    "type": "addItem",
                    "cfg": {
                        "spec": {
                            "name": "static",
                            "value": "Europe"
                        },
                        "target": {
                            "idx": 12,
                            "header": "Region",
                            "targetType": "String"
                        }
                    }
                }],
                "orderItems": [
                    { "from": 12, "to": 3 }
                ]
            }]
        }
        "#;

        assert_eq!(
            ConfigRoot {
                passes: vec![
                    TransformEnrichPassConfig {
                        comment: Some(String::from("pass1")),
                        transformers: vec![
                            TransformerConfig::DeleteItems {
                                cfg: vec![0_usize, 1_usize]
                            },
                            TransformerConfig::SplitItem {
                                cfg: SplitItemConfig {
                                    idx: 2,
                                    spec: SplitterType::SeparatorChar { char: ';' },
                                    delete_after_split: true,
                                    target_left: ItemTargetConfig {
                                        header: Some(String::from("some_float32_left")),
                                        idx: 10_usize,
                                        target_type: ValueName::Float32,
                                    },
                                    target_right: ItemTargetConfig {
                                        header: Some(String::from("some_string_right")),
                                        idx: 11_usize,
                                        target_type: ValueName::String,
                                    },
                                }
                            }
                        ],
                        order_items: Some(vec![
                            OrderItemsEntry {
                                from: 3_usize,
                                to: 0_usize
                            },
                            OrderItemsEntry {
                                from: 4_usize,
                                to: 1_usize
                            },
                            OrderItemsEntry {
                                from: 10_usize,
                                to: 2_usize
                            }
                        ]),
                    },
                    TransformEnrichPassConfig {
                        comment: Some(String::from("pass2")),
                        transformers: vec![TransformerConfig::AddItem {
                            cfg: AddItemConfig {
                                spec: AddItemType::Static {
                                    value: String::from("Europe")
                                },
                                target: ItemTargetConfig {
                                    header: Some(String::from("Region")),
                                    idx: 12_usize,
                                    target_type: ValueName::String
                                }
                            }
                        }],
                        order_items: Some(vec![OrderItemsEntry {
                            from: 12_usize,
                            to: 3_usize
                        }]),
                    },
                ]
            },
            serde_json::from_str(data).expect("could not deserialize ")
        )
    }
}
