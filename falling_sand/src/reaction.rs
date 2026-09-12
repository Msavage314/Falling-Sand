use crate::cell::Cell;
use crate::explosion::AcidExplosion;
use crate::explosion::Explosion;
use crate::explosion::FireExplosion;
use crate::explosion::RayTracedExplosion;
use crate::materials::Behavior;
use crate::materials::MATERIAL_COUNT;
use crate::materials::MaterialID;
use crate::rng;
use crate::stains::Stain;
use crate::stains::StainKind;
use std::sync::Arc;
use std::sync::LazyLock;
use strum::IntoEnumIterator;
/// A reactant can be either a material E.g. Water and Salt or a behavior e.g. Acid and anything with behavior Corrodible
#[derive(Debug, Clone, Copy)]
pub enum Reactant {
    Material(MaterialID),
    Behavior(Behavior),
    Stain(StainKind),
}
impl Reactant {
    pub fn matches(self, cell: Cell) -> bool {
        match self {
            Reactant::Material(id) => id == cell.material,
            Reactant::Behavior(flag) => {
                let mat_behavior = cell.material.properties().behavior;
                let stain_behavior = cell
                    .stain
                    .map(|s| s.kind.behavior())
                    .unwrap_or(Behavior::empty());
                (mat_behavior | stain_behavior).contains(flag)
            }
            Reactant::Stain(kind) => cell.stain.is_some_and(|s| s.kind == kind),
        }
    }
}

/// Represents the output of a chemical reaction.
#[derive(Debug, Clone)]
pub enum Product {
    Material(MaterialID),
    Stain(Stain),
    NoChange,
    Explosion {
        source: Arc<dyn Explosion>,
        x_offset: i32,
        y_offset: i32,
    },
}

/// Represents the outcome of a react
#[derive(Clone, Copy)]
pub struct ReactionOutcome {
    pub apply_fn: fn(a: Cell, b: Cell) -> Product,
}

#[derive(Clone, Copy)]
pub struct Reaction {
    pub a: Reactant,
    pub b: Reactant,

    pub output_a: Option<ReactionOutcome>,
    pub output_b: Option<ReactionOutcome>,

    pub chance: f32,
}

pub static REACTIONS: &[Reaction] = &[
    Reaction {
        a: Reactant::Material(MaterialID::Salt),
        b: Reactant::Material(MaterialID::Water),

        output_a: Some(ReactionOutcome {
            apply_fn: |_a, _b| Product::Material(MaterialID::SaltWater),
        }),
        output_b: Some(ReactionOutcome {
            apply_fn: |_a, _b| {
                if rng::chance(0.5) {
                    Product::Material(MaterialID::Water)
                } else {
                    Product::NoChange
                }
            },
        }),
        chance: 0.1,
    },
    Reaction {
        a: Reactant::Behavior(Behavior::HOT),
        b: Reactant::Material(MaterialID::Water),

        output_a: Some(ReactionOutcome {
            apply_fn: |a, _b| {
                if rng::chance(0.5) {
                    Product::Material(a.material.properties().cools_to)
                } else {
                    Product::NoChange
                }
            },
        }),
        output_b: Some(ReactionOutcome {
            apply_fn: |_a, _b| Product::Material(MaterialID::Steam),
        }),

        chance: 0.5,
    },
    Reaction {
        a: Reactant::Material(MaterialID::Steam),
        b: Reactant::Behavior(Behavior::STATIC),

        output_a: Some(ReactionOutcome {
            apply_fn: |_a, _b| {
                if rng::chance(0.5) {
                    Product::Material(MaterialID::Water)
                } else {
                    Product::NoChange
                }
            },
        }),
        output_b: Some(ReactionOutcome {
            apply_fn: |_a, _b| Product::NoChange,
        }),

        chance: 0.01,
    },
    Reaction {
        a: Reactant::Behavior(Behavior::MELTABLE),
        b: Reactant::Material(MaterialID::Lava),

        output_a: Some(ReactionOutcome {
            apply_fn: |a, _b| {
                if rng::chance(a.material.properties().lava_resistance) {
                    Product::Material(MaterialID::Empty)
                } else {
                    Product::NoChange
                }
            },
        }),
        output_b: Some(ReactionOutcome {
            apply_fn: |_a, _b| {
                if rng::chance(0.2) {
                    Product::Material(MaterialID::Empty)
                } else {
                    Product::NoChange
                }
            },
        }),
        chance: 0.05,
    },
    Reaction {
        a: Reactant::Behavior(Behavior::FLAMMABLE),
        b: Reactant::Material(MaterialID::Lava),

        output_a: Some(ReactionOutcome {
            apply_fn: |a, _b| {
                if rng::chance(a.material.properties().flammability) {
                    Product::Stain(Stain {
                        kind: StainKind::Burning,
                        intensity: a.material.properties().burn_intensity,
                        timer: a.material.properties().burn_time,
                    })
                } else {
                    Product::NoChange
                }
            },
        }),
        output_b: None,
        chance: 1.0,
    },
    Reaction {
        a: Reactant::Material(MaterialID::Fire),
        b: Reactant::Material(MaterialID::Empty),

        output_a: Some(ReactionOutcome {
            apply_fn: |_a, _b| Product::Material(MaterialID::Empty),
        }),
        output_b: Some(ReactionOutcome {
            apply_fn: |_a, _b| {
                if rng::chance(0.1) {
                    Product::Material(MaterialID::Smoke)
                } else {
                    Product::NoChange
                }
            },
        }),
        chance: 0.1,
    },
    Reaction {
        a: Reactant::Material(MaterialID::Fire),
        b: Reactant::Material(MaterialID::Fire),

        output_a: Some(ReactionOutcome {
            apply_fn: |_a, _b| Product::Material(MaterialID::Empty),
        }),
        output_b: Some(ReactionOutcome {
            apply_fn: |_a, _b| {
                if rng::chance(0.1) {
                    Product::Material(MaterialID::Smoke)
                } else {
                    Product::NoChange
                }
            },
        }),
        chance: 0.1,
    },
    Reaction {
        a: Reactant::Stain(StainKind::Burning),
        b: Reactant::Material(MaterialID::Empty),

        output_a: None,
        output_b: Some(ReactionOutcome {
            apply_fn: |_a, _b| {
                if rng::chance(0.5) {
                    Product::Material(MaterialID::Fire)
                } else {
                    Product::NoChange
                }
            },
        }),
        chance: 1.0,
    },
    Reaction {
        a: Reactant::Material(MaterialID::Smoke),
        b: Reactant::Behavior(Behavior::STATIC),

        output_a: Some(ReactionOutcome {
            apply_fn: |_a, _b| Product::Material(MaterialID::Empty),
        }),
        output_b: None,
        chance: 0.1,
    },
    Reaction {
        a: Reactant::Material(MaterialID::FlammableGas),
        b: Reactant::Behavior(Behavior::STATIC),

        output_a: Some(ReactionOutcome {
            apply_fn: |_a, _b| Product::Material(MaterialID::Empty),
        }),
        output_b: None,
        chance: 0.02,
    },
    Reaction {
        a: Reactant::Material(MaterialID::Fire),
        b: Reactant::Behavior(Behavior::FLAMMABLE),

        output_a: None,
        output_b: Some(ReactionOutcome {
            apply_fn: |_a, b| match b.stain {
                // Don't relight already burning cells. Causes small particles of wood to burn forever
                None => Product::Stain(Stain {
                    kind: StainKind::Burning,
                    intensity: b.material.properties().burn_intensity,
                    timer: b.material.properties().burn_time,
                }),
                _ => Product::NoChange,
            },
        }),
        chance: 0.01,
    },
    Reaction {
        a: Reactant::Behavior(Behavior::CORRODIBLE),
        b: Reactant::Material(MaterialID::Acid),

        output_a: Some(ReactionOutcome {
            apply_fn: |a, _b| {
                if rng::chance(a.material.properties().acid_resistance) {
                    Product::Material(MaterialID::FlammableGas)
                } else {
                    Product::NoChange
                }
            },
        }),
        output_b: Some(ReactionOutcome {
            apply_fn: |_a, _b| {
                if rng::chance(0.2) {
                    Product::Material(MaterialID::Empty)
                } else {
                    Product::NoChange
                }
            },
        }),
        chance: 0.3,
    },
    Reaction {
        a: Reactant::Material(MaterialID::Water),
        b: Reactant::Stain(StainKind::Burning),

        output_a: None,
        output_b: Some(ReactionOutcome {
            apply_fn: |_a, _b| {
                Product::Stain(Stain {
                    kind: StainKind::Wet,
                    intensity: 1.0,
                    timer: 2.0,
                })
            },
        }),

        chance: 0.5,
    },
    Reaction {
        a: Reactant::Material(MaterialID::Water),
        b: Reactant::Behavior(Behavior::PERMEABLE),

        output_a: Some(ReactionOutcome {
            apply_fn: |_a, _b| {
                if rng::chance(0.1) {
                    Product::Material(MaterialID::Empty)
                } else {
                    Product::NoChange
                }
            },
        }),
        output_b: Some(ReactionOutcome {
            apply_fn: |_a, b| {
                let existing = b.stain.map(|s| s.intensity).unwrap_or(0.0);

                Product::Stain(Stain {
                    kind: StainKind::Wet,
                    intensity: (existing + 0.1).min(1.0),
                    timer: 10.0,
                })
            },
        }),
        chance: 0.5,
    },
    Reaction {
        a: Reactant::Stain(StainKind::Burning),
        b: Reactant::Behavior(Behavior::FLAMMABLE),

        output_a: None,
        output_b: Some(ReactionOutcome {
            apply_fn: |_a, b| {
                if b.stain.is_some_and(|s| s.kind == StainKind::Burning) {
                    return Product::NoChange;
                }
                let wetness = b
                    .stain
                    .filter(|s| s.kind == StainKind::Wet)
                    .map(|s| s.intensity)
                    .unwrap_or(0.0);

                let flammability = b.material.properties().flammability * (1.0 - wetness);

                if flammability > 0.0 && rng::chance(flammability) {
                    Product::Stain(Stain {
                        kind: StainKind::Burning,
                        intensity: b.material.properties().burn_intensity,
                        timer: b.material.properties().burn_time,
                    })
                } else {
                    Product::NoChange
                }
            },
        }),
        chance: 1.0,
    },
    Reaction {
        a: Reactant::Stain(StainKind::Wet),
        b: Reactant::Stain(StainKind::Burning),

        output_a: None,
        output_b: Some(ReactionOutcome {
            apply_fn: |a, _b| {
                let intensity = (a.stain.map(|s| s.intensity).unwrap_or(1.0) - 0.1).clamp(0.0, 0.1);
                if intensity <= 0.0 {
                    return Product::NoChange;
                }
                Product::Stain(Stain {
                    kind: StainKind::Wet,
                    intensity,
                    timer: 2.0,
                })
            },
        }),
        chance: 0.5,
    },
    Reaction {
        a: Reactant::Material(MaterialID::Slime),
        b: Reactant::Behavior(Behavior::POWDER),

        output_a: None,
        output_b: Some(ReactionOutcome {
            apply_fn: |_a, b| {
                if b.stain.is_some_and(|s| s.kind == StainKind::Slimy) {
                    Product::NoChange
                } else {
                    Product::Stain(Stain {
                        kind: StainKind::Slimy,
                        intensity: 1.0,
                        timer: 1.0,
                    })
                }
            },
        }),
        chance: 0.05,
    },
    Reaction {
        a: Reactant::Material(MaterialID::ToxicSludge),
        b: Reactant::Behavior(Behavior::POWDER),

        output_a: None,
        output_b: Some(ReactionOutcome {
            apply_fn: |_a, b| {
                if b.stain.is_some_and(|s| s.kind == StainKind::Toxic) {
                    Product::NoChange
                } else {
                    Product::Stain(Stain {
                        kind: StainKind::Toxic,
                        intensity: 1.0,
                        timer: 100.0,
                    })
                }
            },
        }),
        chance: 0.05,
    },
    Reaction {
        a: Reactant::Material(MaterialID::ToxicSludge),
        b: Reactant::Behavior(Behavior::STATIC),

        output_a: None,
        output_b: Some(ReactionOutcome {
            apply_fn: |_a, b| {
                if b.stain.is_some_and(|s| s.kind == StainKind::Toxic) {
                    Product::NoChange
                } else {
                    Product::Stain(Stain {
                        kind: StainKind::Toxic,
                        intensity: 1.0,
                        timer: 100.0,
                    })
                }
            },
        }),
        chance: 0.05,
    },
    Reaction {
        a: Reactant::Stain(StainKind::Wet),
        b: Reactant::Behavior(Behavior::PERMEABLE),

        output_a: None,
        output_b: Some(ReactionOutcome {
            apply_fn: |a, b| {
                let source_intensity = a.stain.map(|s| s.intensity).unwrap_or(1.0);
                let new_intensity = source_intensity - 0.02;

                let (target_intensity, target_timer) = b
                    .stain
                    .filter(|s| s.kind == StainKind::Wet)
                    .map(|s| (s.intensity, s.timer))
                    .unwrap_or((0.0, 0.0));

                // Epsilon avoids perpetual re-triggering on near-equal intensities
                if new_intensity <= target_intensity + 0.01 {
                    return Product::NoChange;
                }

                Product::Stain(Stain {
                    kind: StainKind::Wet,
                    intensity: new_intensity,
                    // Don't reset to a flat max — just make sure the cell has at
                    // least enough time to keep existing; evaporation handles the rest
                    timer: target_timer.max(3.0),
                })
            },
        }),
        chance: 0.3,
    },
    Reaction {
        a: Reactant::Material(MaterialID::Snow),
        b: Reactant::Behavior(Behavior::HOT),

        output_a: Some(ReactionOutcome {
            apply_fn: |_a, _b| Product::Material(MaterialID::Steam),
        }),
        output_b: None,
        chance: 1.0,
    },
    Reaction {
        a: Reactant::Material(MaterialID::Water),
        b: Reactant::Material(MaterialID::ToxicSludge),

        output_a: None,
        output_b: Some(ReactionOutcome {
            apply_fn: |_a, _b| Product::Material(MaterialID::Water),
        }),
        chance: 0.1,
    },
    Reaction {
        a: Reactant::Behavior(Behavior::HOT),
        b: Reactant::Material(MaterialID::Gunpowder),

        output_a: None,
        output_b: Some(ReactionOutcome {
            apply_fn: |_a, _b| Product::Explosion {
                source: Arc::new(FireExplosion {
                    radius: 3,
                    particle_chance: 0.2,
                    velocity: 3.0,
                }),
                x_offset: 0,
                y_offset: 0,
            },
        }),
        chance: 1.0,
    },
    Reaction {
        a: Reactant::Behavior(Behavior::HOT),
        b: Reactant::Material(MaterialID::Nitro),

        output_a: None,
        output_b: Some(ReactionOutcome {
            apply_fn: |_a, _b| Product::Explosion {
                source: Arc::new(AcidExplosion { radius: 15 }),
                x_offset: 0,
                y_offset: 0,
            },
        }),
        chance: 1.0,
    },
    Reaction {
        a: Reactant::Behavior(Behavior::HOT),
        b: Reactant::Material(MaterialID::Dynamite),

        output_a: None,
        output_b: Some(ReactionOutcome {
            apply_fn: |_a, _b| Product::Explosion {
                source: Arc::new(RayTracedExplosion {
                    radius: 100,
                    power: 100.0,
                }),
                x_offset: 0,
                y_offset: 0,
            },
        }),
        chance: 1.0,
    },
    Reaction {
        a: Reactant::Behavior(Behavior::HOT),
        b: Reactant::Material(MaterialID::Methane),

        output_a: None,
        output_b: Some(ReactionOutcome {
            apply_fn: |_a, _b| Product::Explosion {
                source: Arc::new(FireExplosion {
                    radius: 6,
                    particle_chance: 0.01,
                    velocity: 0.5,
                }),
                x_offset: 0,
                y_offset: -1,
            },
        }),
        chance: 1.0,
    },
];
pub static REACTIONS_BY_MATERIAL: LazyLock<[Vec<&'static Reaction>; MATERIAL_COUNT]> =
    LazyLock::new(|| {
        std::array::from_fn(|i| {
            let mat = MaterialID::iter().nth(i).unwrap(); // or a From<usize> impl if you have one
            REACTIONS
                .iter()
                .filter(|r| match r.a {
                    Reactant::Material(id) => id == mat,
                    Reactant::Behavior(flag) => mat.properties().behavior.contains(flag),
                    Reactant::Stain(_) => true,
                })
                .collect()
        })
    });

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::Color;

    #[test]
    fn material_reactant_matches() {
        let r = Reactant::Material(MaterialID::Salt);
        assert!(r.matches(Cell {
            material: MaterialID::Salt,
            stain: None,
            color: Color::new(1.0, 1.0, 1.0, 1.0),
            awake: false
        }));
        assert!(!r.matches(Cell {
            material: MaterialID::Water,
            stain: None,
            color: Color::new(1.0, 1.0, 1.0, 1.0),
            awake: false
        }))
    }
    #[test]
    fn stain_reactant_matches() {
        let r = Reactant::Stain(StainKind::Burning);
        assert!(r.matches(Cell {
            material: MaterialID::Water,
            stain: Some(Stain {
                kind: StainKind::Burning,
                intensity: 1.0,
                timer: 1.0
            }),
            color: Color::new(1.0, 1.0, 1.0, 1.0),
            awake: false
        }));
        assert!(!r.matches(Cell {
            material: MaterialID::Water,
            stain: None,
            color: Color::new(1.0, 1.0, 1.0, 1.0),
            awake: false
        }))
    }
    #[test]
    fn behavior_reactant_matches() {
        let r = Reactant::Behavior(Behavior::HOT);
        assert!(r.matches(Cell {
            material: MaterialID::Fire,
            stain: None,
            color: Color::new(1.0, 1.0, 1.0, 1.0),
            awake: false
        }));

        assert!(r.matches(Cell {
            material: MaterialID::Wood,
            stain: Some(Stain {
                kind: StainKind::Burning,
                intensity: 1.0,
                timer: 1.0
            }),
            color: Color::new(1.0, 1.0, 1.0, 1.0),
            awake: false
        }));
    }
}
