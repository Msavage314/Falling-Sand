use crate::materials::Behavior;
use crate::materials::MaterialID;
use crate::stains::Stain;
use crate::stains::StainKind;

/// A reactant can be either a material E.g. Water and Salt or a behavior e.g. Acid and anything with behavior Corrodable
#[derive(Debug, Clone, Copy)]
pub enum Reactant {
    Material(MaterialID),
    Behavior(Behavior),
    Stain(StainKind),
}
impl Reactant {
    pub fn matches(self, mat: MaterialID, stain: Option<Stain>) -> bool {
        match self {
            Reactant::Material(id) => id == mat,
            Reactant::Behavior(flag) => mat.properties().behavior.contains(flag),
            Reactant::Stain(kind) => stain.is_some_and(|s| s.kind == kind),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Product {
    Material(MaterialID),
    Stain(StainKind),
}

#[derive(Clone, Copy)]
pub struct ReactionOutcome {
    pub chance_fn: fn(self_mat: MaterialID, other_mat: MaterialID) -> f32,
    pub result: Product,
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
            chance_fn: |a, b| 1.0,
            result: Product::Material(MaterialID::SaltWater),
        }),
        output_b: Some(ReactionOutcome {
            chance_fn: |a, b| 0.5,
            result: Product::Material(MaterialID::Water),
        }),
        chance: 0.1,
    },
    Reaction {
        a: Reactant::Material(MaterialID::Lava),
        b: Reactant::Material(MaterialID::Water),

        output_a: Some(ReactionOutcome {
            chance_fn: |a, b| 0.5,
            result: Product::Material(MaterialID::Stone),
        }),
        output_b: Some(ReactionOutcome {
            chance_fn: |a, b| 1.0,
            result: Product::Material(MaterialID::Steam),
        }),

        chance: 0.5,
    },
    Reaction {
        a: Reactant::Material(MaterialID::Steam),
        b: Reactant::Material(MaterialID::Steam),

        output_a: Some(ReactionOutcome {
            chance_fn: |a, b| 0.1,
            result: Product::Material(MaterialID::Water),
        }),
        output_b: Some(ReactionOutcome {
            chance_fn: |a, b| 1.0,
            result: Product::Material(MaterialID::Empty),
        }),

        chance: 0.01,
    },
    Reaction {
        a: Reactant::Behavior(Behavior::MELTABLE),
        b: Reactant::Material(MaterialID::Lava),

        output_a: Some(ReactionOutcome {
            chance_fn: |a, b| a.properties().lava_resistance,
            result: Product::Material(MaterialID::Lava),
        }),
        output_b: Some(ReactionOutcome {
            chance_fn: |a, b| 0.2,
            result: Product::Material(MaterialID::Empty),
        }),
        chance: 0.05,
    },
    Reaction {
        a: Reactant::Behavior(Behavior::FLAMMABLE),
        b: Reactant::Material(MaterialID::Lava),

        output_a: Some(ReactionOutcome {
            chance_fn: |a, b| a.properties().flammability,
            result: Product::Stain(StainKind::Burning),
        }),
        output_b: None,
        chance: 0.05,
    },
    Reaction {
        a: Reactant::Material(MaterialID::Fire),
        b: Reactant::Material(MaterialID::Empty),

        output_a: Some(ReactionOutcome {
            chance_fn: |a, b| 1.0,
            result: Product::Material(MaterialID::Empty),
        }),
        output_b: Some(ReactionOutcome {
            chance_fn: |a, b| 0.1,
            result: Product::Material(MaterialID::Smoke),
        }),
        chance: 0.1,
    },
    Reaction {
        a: Reactant::Stain(StainKind::Burning),
        b: Reactant::Material(MaterialID::Empty),

        output_a: None,
        output_b: Some(ReactionOutcome {
            chance_fn: |a, b| 0.1,
            result: Product::Material(MaterialID::Fire),
        }),
        chance: 1.0,
    },
    Reaction {
        a: Reactant::Material(MaterialID::Empty),
        b: Reactant::Material(MaterialID::Smoke),

        output_a: None,
        output_b: Some(ReactionOutcome {
            chance_fn: |a, b| 1.0,
            result: Product::Material(MaterialID::Empty),
        }),
        chance: 0.01,
    },
    Reaction {
        a: Reactant::Material(MaterialID::Fire),
        b: Reactant::Behavior(Behavior::FLAMMABLE),

        output_a: None,
        output_b: Some(ReactionOutcome {
            chance_fn: |a, b| 1.0,
            result: Product::Stain(StainKind::Burning),
        }),
        chance: 0.01,
    },
    Reaction {
        a: Reactant::Material(MaterialID::Acid),
        b: Reactant::Behavior(Behavior::CORRODABLE),

        output_a: Some(ReactionOutcome {
            chance_fn: |a, b| 0.1,
            result: Product::Material(MaterialID::Empty),
        }),
        output_b: Some(ReactionOutcome {
            chance_fn: |a, b| 1.0,
            result: Product::Material(MaterialID::Empty),
        }),
        chance: 0.1,
    },
    Reaction {
        a: Reactant::Material(MaterialID::Water),
        b: Reactant::Stain(StainKind::Burning),

        output_a: None,
        output_b: Some(ReactionOutcome {
            chance_fn: |_a, _b| 1.0,
            result: Product::Stain(StainKind::Wet),
        }),
        chance: 0.5,
    },
    // Reaction {
    //     a: Reactant::Material(MaterialID::Water),
    //     b: Reactant::Material(MaterialID::Wood),

    //     output_a: None,
    //     output_b: Some(ReactionOutcome {
    //         chance_fn: |_a, _b| 1.0,
    //         result: Product::Stain(StainKind::Wet),
    //     }),
    //     chance: 0.05,
    // },
];
