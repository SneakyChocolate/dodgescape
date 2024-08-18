
#[derive(Debug)]
pub struct Item {
    pub name: String,
    pub amount: usize,
    pub active: bool,
}

impl Item {
    pub fn new(name: &str, amount: usize) -> Self {
        Item {
            name: name.to_owned(),
            amount,
            active: false,
        }
    }
}
