use bitflags::bitflags;
use macroquad::color::Color;
use std::sync::LazyLock;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

bitflags! {
    #[derive(Debug,Clone,Copy,PartialEq )]
    pub struct Behavior: u16 {
        const FALLS = 1<<0; // Obeys gravity generally. Will fall down
        const FLOWS = 1 << 1; // flows sideways when blocked
        const STATIC = 1 << 2; // doesn't move
        const GRANULAR = 1 <<3; // Piles diagonally when blocked
        const RISES = 1<<4;

        const MELTABLE = 1<<5; // Destroyed by lava
        const FLAMMABLE = 1 <<6;

        const CORRODIBLE = 1<<7;

        const PERMEABLE = 1<<8; // Can be given the wet stain


        const SAND = Self::FALLS.bits() | Self::GRANULAR.bits();
        const LIQUID = Self::FALLS.bits() | Self::FLOWS.bits() | Self::GRANULAR.bits();
        const GAS = Self::RISES.bits() |Self::FLOWS.bits();
    }
}

/// Stores a material ID that represents a specific material
#[derive(Debug, Clone, Copy, PartialEq, EnumIter, Eq, Hash)]
pub enum MaterialID {
    Empty,
    Sand,
    Stone,
    Water,
    Slime,
    Salt,
    SaltWater,
    Lava,
    Steam,
    Dirt,
    Snow,
    Oil,
    Wood,
    Fire,
    Smoke,
    Acid,
    DenseRock,
    FlammableGas,
}
impl MaterialID {
    /// Returns a `MaterialProperties` struct of the properties associated with the materialID
    ///
    /// # Examples
    /// ```
    /// let x = MaterialID::Empty
    /// println!("{}",x.properties().desnsity)
    /// ```
    pub fn properties(self) -> &'static MaterialProperties {
        return &MATERIAL_TABLE[self as usize];
    }
}

pub static MATERIAL_TABLE: LazyLock<[MaterialProperties; 18]> = LazyLock::new(|| {
    [
        /* Empty */
        MaterialProperties {
            behavior: Behavior::empty(),
            density: 0.5,
            color: Color::new(0.05, 0.05, 0.05, 1.0),
            ..Default::default()
        },
        /* Sand  */
        MaterialProperties {
            behavior: Behavior::SAND | Behavior::MELTABLE | Behavior::CORRODIBLE,
            density: 1.5,
            color: Color::new(0.96, 0.82, 0.45, 1.0),
            lava_resistance: 0.8,
            flammability: 0.0,
            acid_resistance: 0.2,
            ..Default::default()
        },
        /* Stone */
        MaterialProperties {
            behavior: Behavior::STATIC | Behavior::MELTABLE | Behavior::CORRODIBLE,
            density: 3.0,
            color: Color::new(0.5, 0.5, 0.5, 1.0),
            lava_resistance: 0.05,
            acid_resistance: 0.7,
            ..Default::default()
        },
        /* Water */
        MaterialProperties {
            behavior: Behavior::LIQUID | Behavior::CORRODIBLE,
            density: 1.0,
            color: Color::new(0.1, 0.45, 0.82, 1.0),
            flow_distance: 5,
            acid_resistance: 0.5,
            ..Default::default()
        },
        /* Slime */
        MaterialProperties {
            behavior: Behavior::LIQUID | Behavior::MELTABLE | Behavior::CORRODIBLE,
            density: 1.3,
            color: Color::new(0.8, 0.3, 0.8, 1.0),
            flow_distance: 3,
            lava_resistance: 1.0,
            acid_resistance: 0.9,
            ..Default::default()
        },
        /* Salt */
        MaterialProperties {
            behavior: Behavior::SAND | Behavior::MELTABLE | Behavior::CORRODIBLE,
            density: 1.5,
            color: Color::new(0.9, 0.9, 1.0, 1.0),
            lava_resistance: 0.1,
            acid_resistance: 0.7,
            ..Default::default()
        },
        /* Salt Water */
        MaterialProperties {
            behavior: Behavior::LIQUID | Behavior::CORRODIBLE,
            density: 1.1,
            color: Color::new(0.43, 0.77, 0.8, 1.0),
            flow_distance: 3,
            acid_resistance: 0.5,
            ..Default::default()
        },
        /* Lava */
        MaterialProperties {
            behavior: Behavior::LIQUID | Behavior::CORRODIBLE,
            density: 1.1,
            color: Color::new(0.95, 0.7, 0.0, 1.0),
            flow_distance: 1,
            acid_resistance: 0.3,
            ..Default::default()
        },
        /* Steam */
        MaterialProperties {
            behavior: Behavior::GAS | Behavior::CORRODIBLE,
            density: 0.1,
            color: Color::new(0.9, 0.9, 0.9, 1.0),
            flow_distance: 6,
            acid_resistance: 0.1,
            ..Default::default()
        },
        /* Dirt */
        MaterialProperties {
            behavior: Behavior::SAND
                | Behavior::MELTABLE
                | Behavior::CORRODIBLE
                | Behavior::PERMEABLE,
            density: 1.7,
            color: Color::new(0.35, 0.23, 0.16, 1.0),
            lava_resistance: 0.4,
            acid_resistance: 0.5,
            ..Default::default()
        },
        /* Snow */
        MaterialProperties {
            behavior: Behavior::SAND | Behavior::MELTABLE | Behavior::CORRODIBLE,
            density: 1.7,
            color: Color::new(1.0, 1.0, 1.0, 1.0),
            lava_resistance: 1.0,
            acid_resistance: 0.0,
            ..Default::default()
        },
        /* Oil */
        MaterialProperties {
            behavior: Behavior::LIQUID | Behavior::FLAMMABLE | Behavior::CORRODIBLE,
            density: 0.9,
            color: Color::new(0.14, 0.1, 0.05, 1.0),
            flow_distance: 3,
            flammability: 0.1,
            burn_intensity: 2.0,
            burn_time: 3.0,
            acid_resistance: 0.1,
            ..Default::default()
        },
        /* Wood */
        MaterialProperties {
            behavior: Behavior::STATIC
                | Behavior::FLAMMABLE
                | Behavior::CORRODIBLE
                | Behavior::PERMEABLE,
            density: 1.7,
            color: Color::new(0.18, 0.12, 0.0, 1.0),
            flammability: 0.05,
            burn_intensity: 1.0,
            burn_time: 5.0,
            acid_resistance: 0.2,
            ..Default::default()
        },
        /* Fire */
        MaterialProperties {
            behavior: Behavior::GAS,
            density: 0.1,
            color: Color::new(1.0, 0.5, 0.0, 1.0),
            flow_distance: 1,
            ..Default::default()
        },
        /* Smoke */
        MaterialProperties {
            behavior: Behavior::GAS | Behavior::CORRODIBLE,
            density: 0.1,
            color: Color::new(0.4, 0.4, 0.4, 1.0),
            flow_distance: 2,
            acid_resistance: 0.0,
            ..Default::default()
        },
        /* Acid */
        MaterialProperties {
            behavior: Behavior::LIQUID,
            density: 0.9,
            color: Color::new(0.14, 0.74, 0.31, 1.0),
            flow_distance: 4,
            ..Default::default()
        },
        /* DenseRock */
        MaterialProperties {
            behavior: Behavior::STATIC,
            density: 0.9,
            color: Color::new(0.2, 0.2, 0.2, 1.0),
            ..Default::default()
        },
        /* FlammableGas */
        MaterialProperties {
            behavior: Behavior::GAS | Behavior::FLAMMABLE,
            density: 0.1,
            color: Color::new(0.2, 0.34, 0.11, 1.0),
            flow_distance: 7,
            flammability: 1.0,
            burn_intensity: 5.0,
            burn_time: 0.2,
            ..Default::default()
        },
    ]
});

#[derive(Debug)]
pub struct MaterialProperties {
    pub behavior: Behavior,
    pub density: f32,
    pub color: Color,

    pub flow_distance: u8,
    pub lava_resistance: f32,

    pub flammability: f32, // Chance of igniting when next to other materials burning
    pub burn_time: f32,    // How long this material burns once ignited
    pub burn_intensity: f32, // peak flame intensity

    pub acid_resistance: f32,
}

impl Default for MaterialProperties {
    fn default() -> Self {
        Self {
            behavior: Behavior::empty(),
            density: 0.0,
            color: Color::new(0.0, 0.0, 0.0, 1.0),
            flow_distance: 0,
            lava_resistance: 0.0,
            flammability: 0.0,
            burn_intensity: 0.0,
            burn_time: 0.0,
            acid_resistance: 0.0,
        }
    }
}
