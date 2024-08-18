use crate::item::Item;


#[derive(Debug, Default)]
pub struct Inventory {
    pub open: bool,
    pub items: Vec<Item>,
}
