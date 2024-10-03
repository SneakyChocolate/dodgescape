use std::collections::HashMap;

use crate::item::Item;


#[derive(Debug, Default)]
pub struct Inventory {
    pub open: bool,
    pub items: Vec<Item>,
    pub selected_item: Option<usize>,
    pub bind_mode: bool,
    pub bindings: HashMap<String, usize>,
}
