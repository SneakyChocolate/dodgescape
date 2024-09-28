
use crate::{enemy::{Enemy, EnemyEffect}, game::{DrawPack, Game}, gametraits::Radius, player::{Player, PlayerEffect}, vector};

pub enum Action {
    AddPlayerPosition((f32,f32)),
    AddEnemyPosition{group: usize, x: f32, y: f32},
    AddPlayerVelocity((f32,f32)),
    DecreaseItemEffect {item: usize, effect: usize},
    DecrementEnemyEase{group: usize, effect: usize},
    Despawn(usize),
    MulEnemyRadiusMultiplier {f: f32, group: usize},
    MulEnemySpeedMultiplier {f: f32, group: usize},
    MulPlayerRadiusMultiplier {f: f32},
    MulPlayerSpeed(f32),
    MulPlayerSpeedMultiplier {f: f32},
    MulPlayerVelocity(f32),
    PushPlayerEffect(PlayerEffect),
    ReduceCooldown(usize),
    ReduceLifetime{group: usize, effect: usize},
    RemoveEnemyEffect {group: usize, effect: usize},
    RemovePlayerEffect {effect: usize},
    RemovePlayerItem {item: usize},
    ResetCooldown(usize),
    SetEnemyRadius(usize, f32),
    SetEnemySpeedAlterEase{group: usize, effect: usize, value: usize},
    SetPlayerEase{effect: usize, value: usize},
    SetPlayerSpeed(f32),
    SetPlayerVelocity((f32,f32)),
    SetPlayerZoomlimit((f32,f32)),
    SpawnCrumble(usize),
    SpawnEnemy { color: String, effects: Vec<EnemyEffect>, group: usize, radius: f32, velocity: (f32, f32) },
    SpawnProjectile { group: usize, velocity: (f32, f32), radius: f32, color: String, lifetime: usize, effects: Vec<EnemyEffect>, under_dps: Vec<DrawPack> },
    UpdateEnemyVelocity(usize, (f32,f32)),
    RevivePlayers {radius: Radius},
}

fn get_player<'a>(game: &'a mut Game, player: usize) -> &'a mut Player {
    game.players.get_mut(player).unwrap()
}
fn get_enemy<'a>(game: &'a mut Game, group: usize, enemy: usize) -> &'a mut Enemy {
    game.enemies.get_mut(group).unwrap().1.get_mut(enemy).unwrap()
}
impl Action {
    pub fn execute(&self, game: &mut Game, entity: usize) {
        match self {
            Action::UpdateEnemyVelocity(g, v) => {
                let enemy = get_enemy(game, *g, entity);
                enemy.velocity = *v;
            },
            Action::SetPlayerVelocity(v) => {
                let player = get_player(game, entity);
                player.velocity = *v;
            },
            Action::SetPlayerZoomlimit(v) => {
                let player = get_player(game, entity);
                player.zoomlimit = *v;
            },
            Action::AddPlayerVelocity(v) => {
                let player = get_player(game, entity);
                player.velocity = (player.velocity.0 + v.0, player.velocity.1 + v.1);
            },
            Action::AddPlayerPosition(v) => {
                let player = get_player(game, entity);
                player.x += v.0;
                player.y += v.1;
            },
            Action::MulPlayerVelocity(factor) => {
                let player = get_player(game, entity);
                player.velocity = (player.velocity.0 * factor, player.velocity.1 * factor);
            },
            Action::SpawnCrumble(g) => {
                let enemy = get_enemy(game, *g, entity);
                let (x, y, v, r) = (enemy.x, enemy.y, enemy.velocity.clone(), enemy.radius / 2.0);
                // cumble
                let mut crumble = Enemy::new(x, y, vector::normalize(v, 0.5), r, "rgb(0,0,0)");
                crumble.effects.push(EnemyEffect::Lifetime(2000));
                game.enemies.get_mut(*g).unwrap().1.push(crumble);
            },
            Action::Despawn(g) => {
                game.enemies.get_mut(*g).unwrap().1.remove(entity);
            },
            Action::ReduceLifetime { group, effect } => {
                let enemy = get_enemy(game, *group, entity);
                let effect = enemy.effects.get_mut(*effect).unwrap();
                match effect {
                    EnemyEffect::Lifetime(t) => {
                        if *t == 0 {
                            game.enemies.get_mut(*group).unwrap().1.remove(entity);
                        }
                        else {
                            *t -= 1;
                        }
                    },
                    _ => {},
                }
            },
            Action::ReduceCooldown(group) => {
                let enemy = get_enemy(game, *group, entity);
                for effect in enemy.effects.iter_mut() {
                    match effect {
                        crate::enemy::EnemyEffect::Shoot { radius, speed, time_left, cooldown, lifetime, projectile_radius, color, effects, under_dps } => {
                            if *time_left > 0 {
                                *time_left -= 1;
                            }
                        },
                        crate::enemy::EnemyEffect::Explode { lifetime, radius, speed, amount, time_left, cooldown, color, effects, under_dps: underDPs } => {
                            if *time_left > 0 {
                                *time_left -= 1;
                            }
                        },
                        _ => {},
                    }
                }
            },
            Action::ResetCooldown(group) => {
                let enemy = get_enemy(game, *group, entity);
                for effect in enemy.effects.iter_mut() {
                    match effect {
                        crate::enemy::EnemyEffect::Shoot { radius, speed, time_left, cooldown, lifetime, projectile_radius, color, effects, under_dps } => {
                            *time_left = *cooldown;
                        },
                        crate::enemy::EnemyEffect::Explode { lifetime, radius, speed, amount, time_left, cooldown, color, effects, under_dps: underDPs } => {
                            *time_left = *cooldown;
                        },
                        _ => {},
                    }
                }
            },
            Action::SpawnProjectile { group, velocity, color, radius, lifetime, effects, under_dps: underDPs } => {
                let enemy = get_enemy(game, *group, entity);
                // projectile
                let mut projectile = Enemy::new(enemy.x, enemy.y, *velocity, *radius, color.as_str());
                let udps = underDPs.clone();
                for udp in udps {
                    projectile.draw_packs.insert(0, udp);
                }
                projectile.effects = effects.clone();
                projectile.effects.push(EnemyEffect::Lifetime(*lifetime));
                game.enemies.get_mut(*group).unwrap().1.push(projectile);
            },
            Action::SpawnEnemy { group, velocity, color, radius, effects } => {
                let enemy = get_enemy(game, *group, entity);
                // projectile
                let mut projectile = Enemy::new(enemy.x, enemy.y, *velocity, *radius, color.as_str());
                projectile.effects = effects.clone();
                game.enemies.get_mut(*group).unwrap().1.push(projectile);
            },
            Action::SetEnemyRadius(group, radius) => {
                let enemy = get_enemy(game, *group, entity);
                enemy.radius = *radius;
            },
            Action::SetPlayerSpeed(s) => {
                let player = game.players.get_mut(entity).unwrap();
                player.speed = *s;
            },
            Action::MulPlayerSpeed(s) => {
                let player = game.players.get_mut(entity).unwrap();
                player.speed *= *s;
            },
            Action::RemoveEnemyEffect { group, effect } => {
                let enemy = game.enemies.get_mut(*group).unwrap().1.get_mut(entity).unwrap();
                enemy.effects.remove(*effect);
            },
            Action::MulEnemySpeedMultiplier { group, f } => {
                let enemy = game.enemies.get_mut(*group).unwrap().1.get_mut(entity).unwrap();
                enemy.speed_multiplier *= *f;
            },
            Action::MulEnemyRadiusMultiplier { group, f } => {
                let enemy = game.enemies.get_mut(*group).unwrap().1.get_mut(entity).unwrap();
                enemy.radius_multiplier *= *f;
            },
            Action::MulPlayerSpeedMultiplier { f } => {
                let player = game.players.get_mut(entity).unwrap();
                player.speed_multiplier *= *f;
            },
            Action::MulPlayerRadiusMultiplier { f } => {
                let player = game.players.get_mut(entity).unwrap();
                player.radius_multiplier *= *f;
            },
            Action::RemovePlayerEffect { effect } => {
                let player = game.players.get_mut(entity).unwrap();
                player.effects.remove(*effect);
            },
            Action::PushPlayerEffect(effect) => {
                let player = game.players.get_mut(entity).unwrap();
                player.effects.push(*effect);
            },
            Action::DecrementEnemyEase { group, effect } => {
                let enemy = game.enemies.get_mut(*group).unwrap().1.get_mut(entity).unwrap();
                let effect = enemy.effects.get_mut(*effect).unwrap();
                match effect {
                    EnemyEffect::SpeedAlter { origin, power: slow, ease } => {
                        *ease -= 1;
                    },
                    EnemyEffect::Shrink { origin, power: slow, ease } => {
                        *ease -= 1;
                    },
                    _ => { }
                }
            },
            Action::SetEnemySpeedAlterEase { group, effect, value } => {
                let enemy = game.enemies.get_mut(*group).unwrap().1.get_mut(entity).unwrap();
                let effect = enemy.effects.get_mut(*effect).unwrap();
                match effect {
                    EnemyEffect::SpeedAlter { origin, power: slow, ease } => {
                        *ease = *value;
                    },
                    _ => { }
                }
            },
            Action::SetPlayerEase {effect, value } => {
                let player = game.players.get_mut(entity).unwrap();
                let effect = player.effects.get_mut(*effect).unwrap();
                match effect {
                    PlayerEffect::SpeedAlter { origin, slow, ease } => {
                        *ease = *value;
                    },
                    PlayerEffect::Shrink { origin, shrink, ease } => {
                        *ease = *value;
                    },
                    _ => { }
                }
            },
            Action::RemovePlayerItem { item } => {
                let player = game.players.get_mut(entity).unwrap();
                player.inventory.items.remove(*item);
            },
            Action::DecreaseItemEffect { item: i, effect: e } => {
                let player = game.players.get_mut(entity).unwrap();
                let item = player.inventory.items.get_mut(*i).unwrap();
                let effect = item.effects.get_mut(*e).unwrap();
                match effect {
                    crate::item::ItemEffect::Consumable { uses } => {
                        *uses -= 1;
                        item.active = false;
                        if *uses == 0 {
                            player.inventory.items.remove(*i);
                        }
                    },
                    _ => {}
                };
            },
            Action::RevivePlayers { radius } => {
                let center = game.players.get_mut(entity).unwrap();
                let r = center.radius;
                let center = (center.x, center.y);
                for player in game.players.iter_mut() {
                    let dist = vector::distance((player.x, player.y), center);
                    if dist.2 <= radius.translate(radius.translate(r)) {
                        player.alive = true;
                    }
                }
            },
            Action::AddEnemyPosition { group, x, y } => {
                let enemy = get_enemy(game, *group, entity);
                enemy.x += *x;
                enemy.y += *y;
            },
        }
    }
}

