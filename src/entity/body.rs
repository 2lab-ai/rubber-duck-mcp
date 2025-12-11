use crate::world::Position;
use rand::Rng;
use serde::{Deserialize, Serialize};

use super::wildlife::Species;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BodyPartKind {
    Head,
    Torso,
    ArmLeft,
    ArmRight,
    LegLeft,
    LegRight,
    FrontLeftLeg,
    FrontRightLeg,
    BackLeftLeg,
    BackRightLeg,
    Tail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyPart {
    pub kind: BodyPartKind,
    pub max_hp: f32,
    pub hp: f32,
    pub vital: bool,
    pub movement: bool,
    pub manipulation: bool,
    pub cognition: bool,
}

impl BodyPart {
    pub fn new(
        kind: BodyPartKind,
        max_hp: f32,
        vital: bool,
        movement: bool,
        manipulation: bool,
        cognition: bool,
    ) -> Self {
        Self {
            kind,
            max_hp,
            hp: max_hp,
            vital,
            movement,
            manipulation,
            cognition,
        }
    }

    pub fn ratio(&self) -> f32 {
        if self.max_hp <= 0.0 {
            1.0
        } else {
            (self.hp / self.max_hp).clamp(0.0, 1.0)
        }
    }

    pub fn is_broken(&self) -> bool {
        self.hp <= 0.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Body {
    pub parts: Vec<BodyPart>,
}

impl Body {
    pub fn human_default() -> Self {
        Self {
            parts: vec![
                BodyPart::new(BodyPartKind::Head, 40.0, true, false, false, true),
                BodyPart::new(BodyPartKind::Torso, 80.0, true, false, false, false),
                BodyPart::new(BodyPartKind::ArmLeft, 50.0, false, false, true, false),
                BodyPart::new(BodyPartKind::ArmRight, 50.0, false, false, true, false),
                BodyPart::new(BodyPartKind::LegLeft, 60.0, false, true, false, false),
                BodyPart::new(BodyPartKind::LegRight, 60.0, false, true, false, false),
            ],
        }
    }

    pub fn for_species(species: Species) -> Self {
        match species {
            Species::Deer | Species::Caribou | Species::SnowHare | Species::Rabbit => Self {
                parts: vec![
                    BodyPart::new(BodyPartKind::Head, 30.0, true, false, false, false),
                    BodyPart::new(BodyPartKind::Torso, 70.0, true, false, false, false),
                    BodyPart::new(BodyPartKind::FrontLeftLeg, 40.0, false, true, false, false),
                    BodyPart::new(BodyPartKind::FrontRightLeg, 40.0, false, true, false, false),
                    BodyPart::new(BodyPartKind::BackLeftLeg, 40.0, false, true, false, false),
                    BodyPart::new(BodyPartKind::BackRightLeg, 40.0, false, true, false, false),
                ],
            },
            Species::Wolf | Species::Fox | Species::DesertFox | Species::SnowFox => Self {
                parts: vec![
                    BodyPart::new(BodyPartKind::Head, 28.0, true, false, false, false),
                    BodyPart::new(BodyPartKind::Torso, 55.0, true, false, false, false),
                    BodyPart::new(BodyPartKind::FrontLeftLeg, 35.0, false, true, false, false),
                    BodyPart::new(BodyPartKind::FrontRightLeg, 35.0, false, true, false, false),
                    BodyPart::new(BodyPartKind::BackLeftLeg, 35.0, false, true, false, false),
                    BodyPart::new(BodyPartKind::BackRightLeg, 35.0, false, true, false, false),
                    BodyPart::new(BodyPartKind::Tail, 10.0, false, false, false, false),
                ],
            },
            _ => Self {
                parts: vec![BodyPart::new(BodyPartKind::Torso, 20.0, true, true, false, false)],
            },
        }
    }

    pub fn head_health_ratio(&self) -> f32 {
        let heads: Vec<_> = self.parts.iter().filter(|p| p.cognition || matches!(p.kind, BodyPartKind::Head)).collect();
        if heads.is_empty() {
            1.0
        } else {
            let sum: f32 = heads.iter().map(|p| p.ratio()).sum();
            sum / heads.len() as f32
        }
    }

    pub fn overall_health_ratio(&self) -> f32 {
        if self.parts.is_empty() {
            1.0
        } else {
            let sum: f32 = self.parts.iter().map(|p| p.ratio()).sum();
            sum / self.parts.len() as f32
        }
    }

    pub fn movement_factor(&self) -> f32 {
        let movers: Vec<_> = self.parts.iter().filter(|p| p.movement).collect();
        if movers.is_empty() {
            1.0
        } else {
            let sum: f32 = movers.iter().map(|p| p.ratio()).sum();
            (sum / movers.len() as f32).clamp(0.0, 1.0)
        }
    }

    pub fn is_vital_broken(&self) -> bool {
        self.parts.iter().any(|p| p.vital && p.is_broken())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BodyHitEvent {
    pub part: BodyPartKind,
    pub damage: f32,
    pub was_vital: bool,
    pub part_broken: bool,
    pub killed: bool,
}

impl Body {
    pub fn apply_random_damage(
        &mut self,
        rng: &mut impl Rng,
        damage: f32,
    ) -> Option<BodyHitEvent> {
        if self.parts.is_empty() || damage <= 0.0 {
            return None;
        }

        let idx = rng.gen_range(0..self.parts.len());
        let part = &mut self.parts[idx];
        part.hp = (part.hp - damage).max(0.0);
        let part_broken = part.is_broken();
        let was_vital = part.vital;
        let killed = was_vital && part_broken;

        Some(BodyHitEvent {
            part: part.kind,
            damage,
            was_vital,
            part_broken,
            killed,
        })
    }

    pub fn describe_hit(&self, hit: &BodyHitEvent, name: &str) -> String {
        let part_name = match hit.part {
            BodyPartKind::Head => "head",
            BodyPartKind::Torso => "torso",
            BodyPartKind::ArmLeft => "left arm",
            BodyPartKind::ArmRight => "right arm",
            BodyPartKind::LegLeft => "left leg",
            BodyPartKind::LegRight => "right leg",
            BodyPartKind::FrontLeftLeg => "front left leg",
            BodyPartKind::FrontRightLeg => "front right leg",
            BodyPartKind::BackLeftLeg => "back left leg",
            BodyPartKind::BackRightLeg => "back right leg",
            BodyPartKind::Tail => "tail",
        };

        if hit.killed {
            format!(
                "You strike the {}'s {} hard. It collapses and lies still.",
                name, part_name
            )
        } else if hit.part_broken {
            format!(
                "Your blow lands on the {}'s {}. The limb fails, leaving it badly crippled.",
                name, part_name
            )
        } else {
            format!(
                "You hit the {}'s {}, leaving it bruised and staggering.",
                name, part_name
            )
        }
    }
}

/// Convenience: distance helper for attacks that may want range checks later.
pub fn is_adjacent(a: &Position, b: &Position) -> bool {
    a.distance_to(b) <= 1.5
}

