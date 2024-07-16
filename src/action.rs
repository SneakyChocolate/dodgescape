use crate::{enemy::{Effect, Enemy}, game::{Game, Moveable}, vector};

pub enum Action {
    UpdateVelocity((f32,f32)),
    SpawnCrumble,
    ReduceLifetime,
    Despawn,
}

impl Action {
    pub fn execute(&self, game: &mut Game, entity: usize) {
        match self {
            Action::UpdateVelocity(v) => {
                let enemy = game.enemies.get_mut(entity).unwrap();
                enemy.velocity = *v;
            },
            Action::SpawnCrumble => {
                let enemy = game.enemies.get_mut(entity).unwrap();
                let (x, y, v, r) = (enemy.x, enemy.y, enemy.velocity.clone(), enemy.radius / 2.0);
                // cumble
                let mut crumble =  Enemy::new(x, y, vector::normalize(v, 0.5), r, "rgb(0,0,0)");
                crumble.effects.push(Effect::Lifetime(2000));
                game.enemies.push(crumble);
            },
            Action::Despawn => {
                game.enemies.remove(entity);
            },
            Action::ReduceLifetime => {
                // TODO why does unwrap throw an error?
                let enemy = match game.enemies.get_mut(entity) {
                    Some(e) => e,
                    None => {return;},
                };
                let mut despawn: Vec<usize> = vec![];
                for effect in enemy.effects.iter_mut() {
                    match effect {
                        crate::enemy::Effect::Lifetime(t) => {
                            if *t == 0 {
                                despawn.push(entity);
                            }
                            else {
                                *effect = crate::enemy::Effect::Lifetime(*t - 1);
                            }
                        },
                        _ => {},
                    }
                }
                for i in despawn {
                    game.enemies.remove(i);
                }
            },
        }
    }
}

