use crate::{action::Action, game::{distance, DrawPack, Game, Shape}, impl_Drawable, impl_Movable, impl_Position, vector};
use crate::gametraits::*;

pub enum Effect {
    Chase {radius: f32, power: f32},
    Crumble,
    Lifetime(usize),
    Push {radius: f32, power: f32},
    Shoot {radius: f32, speed: f32, time_left: usize, cooldown: usize}
}

#[derive(Default)]
pub struct Enemy {
    pub velocity: (f32, f32),
    pub x: f32,
    pub y: f32,
    pub draw_packs: Vec<DrawPack>,
    pub radius: f32,
    pub effects: Vec<Effect>,
    pub just_collided: bool,
    pub view_radius: f32,
}

impl_Position!(Enemy);
impl_Movable!(Enemy);
impl_Drawable!(Enemy);

impl Enemy {
    pub fn new(x: f32, y: f32, velocity: (f32, f32), radius: f32, color: &str) -> Enemy {
        let mut p = Enemy {
            x,y,
            velocity,
            radius,
            view_radius: radius,
            draw_packs: vec![],
            ..Default::default()
        };
        p.draw_packs.push(DrawPack::new(color, Shape::Circle { radius: p.radius }, (0.0, 0.0)));

        p
    }
}

pub fn handle_effects(game: &mut Game) {
    let mut actions: Vec<(usize, Action)> = vec![];
    for (g, group) in game.enemies.iter().enumerate() {
        for (i, enemy) in group.1.iter().enumerate() {
            for effect in enemy.effects.iter() {
                match effect {
                    Effect::Chase { radius, power } => {
                        for player in game.players.iter() {
                            // if !player.alive {continue;}
                            let dist = distance(enemy, player);
                            if dist.2 <= *radius + player.radius {
                                let add = vector::normalize((dist.0, dist.1), *power);
                                actions.push((i, Action::UpdateEnemyVelocity(g, (enemy.velocity.0 + add.0, enemy.velocity.1 + add.1))));
                            }
                        }
                    }
                    Effect::Crumble => {
                        if enemy.just_collided {
                            actions.push((i, Action::SpawnCrumble(g)));
                        }
                    },
                    Effect::Lifetime(t) => {
                        actions.push((i, Action::ReduceLifetime(g)));
                    },
                    Effect::Push { radius, power } => {
                        for (p, player) in game.players.iter().enumerate() {
                            let dist = distance(enemy, player);
                            if dist.2 <= *radius + player.radius {
                                let add = vector::normalize((dist.0, dist.1), *power);
                                actions.push((p, Action::AddPlayerVelocity(add)));
                            }
                        }
                    },
                    Effect::Shoot { radius, speed, cooldown, time_left } => {
                        for player in game.players.iter() {
                            if !player.alive {continue;}
                            let dist = distance(enemy, player);
                            if dist.2 <= *radius + player.radius {
                                let v = vector::normalize((dist.0, dist.1), *speed);
                                if *time_left == 0 {
                                    actions.push((i, Action::SpawnProjectile { group: g, velocity: v, radius: 20.0, color: "black".to_owned()}));
                                }
                                actions.push((i, Action::ReduceCooldown(g)));
                            }
                        }
                    },
                }
            }
        }
    }
    for (i, action) in actions {
        action.execute(game, i);
    }
}
