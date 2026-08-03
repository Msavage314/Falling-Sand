use crate::cell::Cell;
use crate::materials::Behavior;
use crate::materials::MaterialID;
use crate::rng;
use crate::stains::Stain;
use crate::stains::StainKind;

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
            Reactant::Behavior(flag) => cell.material.properties().behavior.contains(flag),
            Reactant::Stain(kind) => cell.stain.is_some_and(|s| s.kind == kind),
        }
    }
}

/// Represents the output of a chemical reaction.
#[derive(Debug, Clone, Copy)]
pub enum Product {
    Material(MaterialID),
    Stain(Stain),
    NoChange,
}

#[derive(Clone, Copy)]
pub struct ReactionOutcome {
    pub apply_fn: fn(a: Cell, b: Cell) -> Product,
}

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
        a: Reactant::Material(MaterialID::Lava),
        b: Reactant::Material(MaterialID::Water),

        output_a: Some(ReactionOutcome {
            apply_fn: |_a, _b| {
                if rng::chance(0.5) {
                    Product::Material(MaterialID::Stone)
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
        chance: 0.1,
    },
    Reaction {
        a: Reactant::Material(MaterialID::Fire),
        b: Reactant::Behavior(Behavior::FLAMMABLE),

        output_a: None,
        output_b: Some(ReactionOutcome {
            apply_fn: |_a, b| {
                Product::Stain(Stain {
                    kind: StainKind::Burning,
                    intensity: b.material.properties().burn_intensity,
                    timer: b.material.properties().burn_time,
                })
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

        output_a: None,
        output_b: Some(ReactionOutcome {
            apply_fn: |_a, b| {
                let existing = b.stain.map(|s| s.intensity).unwrap_or(0.0);
                Product::Stain(Stain {
                    kind: StainKind::Wet,
                    intensity: (existing + 5.0).min(1.0),
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
        a: Reactant::Stain(StainKind::Wet),
        b: Reactant::Behavior(Behavior::PERMEABLE),

        output_a: None,
        output_b: Some(ReactionOutcome {
            apply_fn: |a, b| {
                let intensity = a.stain.map(|s| s.intensity).unwrap_or(1.0) - 0.05;
                if intensity <= 0.0 {
                    return Product::NoChange;
                }
                Product::Stain(Stain {
                    kind: StainKind::Wet,
                    intensity,
                    timer: 5.0,
                })
            },
        }),
        chance: 0.3,
    },
];
