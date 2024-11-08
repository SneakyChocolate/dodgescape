use rand::{thread_rng, Rng};

use crate::{action::Action, game::{DrawPack, Game, Shape}, impl_Drawable, impl_Entity, impl_Moveable, impl_Position, player::PlayerEffect, vector, Float};
use crate::gametraits::*;
use crate::{impl_RadiusTrait};

#[derive(Default)]
pub struct Enemy {
    pub id: usize,
    pub velocity: (Float, Float),
    pub speed_multiplier: Float,
    pub radius_multiplier: Float,
    pub x: Float,
    pub y: Float,
    pub draw_packs: Vec<DrawPack>,
    pub radius: Float,
    pub effects: Vec<EnemyEffect>,
    pub just_collided: bool,
    pub view_radius: Radius,
    pub harmless: bool,
    pub old_position: (Float, Float),
}

impl_Entity!(Enemy);

impl Enemy {
    pub fn new(x: Float, y: Float, velocity: (Float, Float), radius: Float, color: &str) -> Enemy {
        let mut p = Enemy {
            x,y,
            old_position: (x, y),
            velocity,
            radius,
            view_radius: Radius::Relative(1.0),
            draw_packs: vec![],
            speed_multiplier: 1.0,
            ..Default::default()
        };
        p.draw_packs.push(DrawPack::new(color, Shape::Circle { radius: Radius::Relative(1.0) }, (0.0, 0.0)));
        p.id = thread_rng().gen_range(0..10000);

        p
    }
}

#[derive(Clone)]
pub enum EnemyEffect {
    Chase {radius: Radius, power: Float},
    Crumble,
    Lifetime(usize),
    Push {radius: Radius, power: Float},
    Shoot {lifetime: usize, radius: Radius, projectile_radius: Float, speed: Float, time_left: usize, cooldown: usize, color: String, effects: Vec<EnemyEffect>, under_dps: Vec<DrawPack>},
    Explode {lifetime: usize, radius: (Float, Float), speed: Float, amount: usize, time_left: usize, cooldown: usize, color: String, effects: Vec<EnemyEffect>, under_dps: Vec<DrawPack>},
    SlowPlayers {radius: Radius, slow: Float, duration: usize},
    Grow {size: Float, maxsize: Float, defaultsize: Float},
    SpeedAlter {origin: usize, power: Float, ease: usize},
    Shrink {origin: usize, power: Float, ease: usize, start: usize},
    ShrinkPlayers {radius: Radius, shrink: Float, duration: usize},
}

pub fn handle_effects(game: &mut Game) {
    let mut actions: Vec<(usize, Action)> = vec![];
    let mut deletions: Vec<(usize, Action)> = vec![];
    for (g, group) in game.enemies.iter().enumerate() {
        for (i, enemy) in group.1.iter().enumerate() {
            for (e, effect) in enemy.effects.iter().enumerate() {
                match effect {
                    EnemyEffect::Chase { radius, power } => {
                        for player in game.players.iter() {
                            if !player.alive {continue;}
                            let dist = vector::distance((enemy.get_x(), enemy.get_y()), (player.get_x(), player.get_y()));
                            if dist.2 <= radius.translate(enemy.get_radius()) + player.get_radius() {
                                let add = vector::normalize((dist.0, dist.1), *power);
                                actions.push((i, Action::UpdateEnemyVelocity(g, (enemy.velocity.0 + add.0, enemy.velocity.1 + add.1))));
                            }
                        }
                    }
                    EnemyEffect::Crumble => {
                        if enemy.just_collided {
                            actions.push((i, Action::SpawnCrumble(g)));
                        }
                    },
                    EnemyEffect::Lifetime(t) => {
                        deletions.push((i, Action::ReduceLifetime { group: g, effect: e }));
                    },
                    EnemyEffect::Push { radius, power } => {
                        for (p, player) in game.players.iter().enumerate() {
                            if !player.alive {
                                continue;
                            }
                            let dist = vector::distance((enemy.get_x(), enemy.get_y()), (player.get_x(), player.get_y()));
                            if dist.2 <= radius.translate(enemy.get_radius()) + player.get_radius() {
                                let add = vector::normalize((dist.0, dist.1), *power);
                                actions.push((p, Action::AddPlayerPosition(add)));
                            }
                        }
                    },
                    EnemyEffect::Shoot { radius, speed, cooldown, time_left, lifetime, projectile_radius, color, effects, under_dps } => {
                        for player in game.players.iter() {
                            if !player.alive {continue;}
                            let dist = vector::distance((enemy.get_x(), enemy.get_y()), (player.get_x(), player.get_y()));
                            if dist.2 <= radius.translate(enemy.get_radius()) + player.get_radius() {
                                let v = vector::normalize((dist.0, dist.1), *speed);
                                if *time_left == 0 {
                                    actions.push((i, Action::SpawnProjectile { group: g, velocity: v, radius: *projectile_radius, color: color.clone(), lifetime: *lifetime, effects: effects.clone(), under_dps: under_dps.clone() }));
                                    actions.push((i, Action::ResetCooldown(g)));
                                }
                                break;
                            }
                        }
                        actions.push((i, Action::ReduceCooldown(g)));
                    },
                    EnemyEffect::Explode { lifetime, radius, speed, amount, time_left, cooldown, color, effects, under_dps } => {
                        if *time_left == 0 {
                            for _ in 0..*amount {
                                let v = (rand::thread_rng().gen_range(-*speed..=*speed), rand::thread_rng().gen_range(-*speed..=*speed));
                                let radius = rand::thread_rng().gen_range(radius.0..=radius.1);
                                actions.push((i, Action::SpawnProjectile { group: g, velocity: v, radius, color: color.clone(), lifetime: *lifetime, effects: effects.clone(), under_dps: under_dps.clone() }));
                            }
                            actions.push((i, Action::ResetCooldown(g)));
                        }
                        actions.push((i, Action::ReduceCooldown(g)));
                    },
                    EnemyEffect::SlowPlayers { radius, slow, duration } => {
                        for (p, player) in game.players.iter().enumerate() {
                            if !player.alive {continue;}
                            let dist = vector::distance((enemy.get_x(), enemy.get_y()), (player.get_x(), player.get_y()));
                            if dist.2 <= radius.translate(enemy.get_radius()) + player.get_radius() {
                                let id = enemy.id;
                                // check if effect of this item id is already applied
                                let position = player.effects.iter().position(|e| {
                                    match e {
                                        PlayerEffect::SpeedAlter { origin, slow, ease } => {
                                            *origin == id
                                        },
                                        _ => {
                                            false
                                        }
                                    }
                                });
                                match position {
                                    Some(e) => {
                                        let effect = player.effects.get(e).unwrap();
                                        match effect {
                                            PlayerEffect::SpeedAlter { origin, slow, ease } => {
                                                // ease = *duration;
                                                actions.push((p, Action::SetPlayerEase { effect: e, value: *duration }));
                                            },
                                            _ => { }
                                        };
                                    },
                                    None => {
                                        let effect = crate::player::PlayerEffect::SpeedAlter { slow: *slow, ease: *duration, origin: id };
                                        actions.push((p, Action::PushPlayerEffect(effect)));
                                    },
                                }
                            }
                        }
                    },
                    EnemyEffect::ShrinkPlayers { radius, shrink, duration } => {
                        for (p, player) in game.players.iter().enumerate() {
                            let dist = vector::distance((enemy.get_x(), enemy.get_y()), (player.get_x(), player.get_y()));
                            if dist.2 <= radius.translate(enemy.get_radius()) + player.get_radius() {
                                let id = enemy.id;
                                // check if effect of this item id is already applied
                                let position = player.effects.iter().position(|e| {
                                    match e {
                                        PlayerEffect::Shrink { origin, shrink, ease } => {
                                            *origin == id
                                        },
                                        _ => {
                                            false
                                        }
                                    }
                                });
                                match position {
                                    Some(e) => {
                                        let effect = player.effects.get(e).unwrap();
                                        match effect {
                                            PlayerEffect::Shrink { origin, shrink, ease } => {
                                                // ease = *duration;
                                                actions.push((p, Action::SetPlayerEase { effect: e, value: *duration }));
                                            },
                                            _ => { }
                                        };
                                    },
                                    None => {
                                        let effect = PlayerEffect::Shrink { origin: id, shrink: *shrink, ease: *duration };
                                        actions.push((p, Action::PushPlayerEffect(effect)));
                                    },
                                }
                            }
                        }
                    },
                    EnemyEffect::Grow { size, maxsize, defaultsize } => {
                        if enemy.just_collided {
                            actions.push((i, Action::SetEnemyRadius(g, *defaultsize)));
                        }
                        else if enemy.radius < *maxsize {
                            actions.push((i, Action::SetEnemyRadius(g, enemy.radius + *size)));
                        }
                    },
                    EnemyEffect::SpeedAlter { power, ease, origin } => {
                        if *ease == 0 {
                            // remove this effect
                            deletions.push((i, Action::RemoveEnemyEffect { group: g, effect: e }));
                        }
                        else {
                            deletions.push((i, Action::DecrementEnemyEase { group: g, effect: e }));
                            actions.push((i, Action::MulEnemySpeedMultiplier { group: g, f: *power }));
                        }
                    },
                    EnemyEffect::Shrink { power, ease, origin, start } => {
                        if *ease == 0 {
                            // remove this effect
                            deletions.push((i, Action::RemoveEnemyEffect { group: g, effect: e }));
                        }
                        else {
                            deletions.push((i, Action::DecrementEnemyEase { group: g, effect: e }));
                            let r = *power + *power * ((*start - *ease) as Float / *start as Float);
                            actions.push((i, Action::MulEnemyRadiusMultiplier { f: r, group: g }));
                        }
                    },
                }
            }
        }
    }
    // reset enemy speed multiplier to 1.0
    for group in game.enemies.iter_mut() {
        for enemy in group.1.iter_mut() {
            enemy.speed_multiplier = 1.0;
            enemy.radius_multiplier = 1.0;
        }
    }
    // reverse order due to deletions and index errors
    for (entity, action) in actions.iter().rev() {
        action.execute(game, *entity);
    }
    for (entity, action) in deletions.iter().rev() {
        action.execute(game, *entity);
    }
}
