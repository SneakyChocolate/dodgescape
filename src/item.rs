use crate::{action::Action, game::Game};


#[derive(Debug)]
pub struct Item {
    pub name: String,
    pub amount: usize,
    pub active: bool,
    pub effects: Vec<ItemEffect>,
}

#[derive(Debug)]
pub enum ItemEffect {
    Vision((f32,f32)),
}

pub fn handle_effects(game: &mut Game) {
    let mut actions: Vec<(usize, Action)> = vec![];
    for (p, player) in game.players.iter().enumerate() {
        for item in player.inventory.items.iter() {
            if !item.active {continue;}
            for effect in item.effects.iter() {
                match effect {
                    ItemEffect::Vision(zoom) => {
                        actions.push((p, Action::SetPlayerZoomlimit(*zoom)));
                    },
                }
            }
        }
        actions.push((p, Action::SetPlayerZoomlimit((1.0, 1.0))));
    }
    for (entity, action) in actions.iter().rev() {
        action.execute(game, *entity);
    }
}

impl Item {
    pub fn new(name: &str, amount: usize, effects: Vec<ItemEffect>) -> Self {
        Item {
            name: name.to_owned(),
            amount,
            active: false,
            effects,
        }
    }
}
