// Local
use super::{von_neumann_neighborhood, Cell};
use crate::universe::{grid2d::ILoc2D, Universe};

#[derive(Copy, Clone, Eq, PartialEq, std::hash::Hash, std::fmt::Debug)]
pub enum VonNeumann {
    Ground,
    Transition(Sensitised),
    Confluent(Excitation, Excitation),
    Transmission(TransmissionType, Direction, Excitation),
}

#[derive(Copy, Clone, Eq, PartialEq, std::hash::Hash, std::fmt::Debug)]
pub enum Direction {
    North,
    South,
    West,
    East,
}

#[derive(Copy, Clone, Eq, PartialEq, std::hash::Hash, std::fmt::Debug)]
pub enum TransmissionType {
    Ordinary,
    Special,
}

#[derive(Copy, Clone, Eq, PartialEq, std::hash::Hash, std::fmt::Debug)]
pub enum Excitation {
    Quiescent,
    Excited,
}

impl From<bool> for Excitation {
    fn from(value: bool) -> Self {
        if value {
            Self::Excited
        } else {
            Self::Quiescent
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, std::hash::Hash, std::fmt::Debug)]
pub enum Sensitised {
    S,
    S0,
    S00,
    S000,
    S01,
    S1,
    S10,
    S11,
}

impl VonNeumann {
    fn any_input(neighbors: &Vec<Self>) -> bool {
        if let VonNeumann::Transmission(_, Direction::South, Excitation::Excited) = neighbors[0] {
            return true;
        }
        if let VonNeumann::Transmission(_, Direction::West, Excitation::Excited) = neighbors[1] {
            return true;
        }
        if let VonNeumann::Transmission(_, Direction::North, Excitation::Excited) = neighbors[2] {
            return true;
        }
        if let VonNeumann::Transmission(_, Direction::East, Excitation::Excited) = neighbors[3] {
            return true;
        }
        false
    }

    fn transmission_update(neighbors: &Vec<Self>, ty: TransmissionType) -> Option<Excitation> {
        let mut excited = false;
        match neighbors[0] {
            VonNeumann::Transmission(nbor_ty, dir, ex) => {
                if dir == Direction::South {
                    if nbor_ty == ty {
                        return None;
                    }
                    excited |= ex == Excitation::Excited;
                }
            }
            _ => (),
        }
        match neighbors[1] {
            VonNeumann::Transmission(nbor_ty, dir, ex) => {
                if dir == Direction::West {
                    if nbor_ty == ty {
                        return None;
                    }
                    excited |= ex == Excitation::Excited;
                }
            }
            _ => (),
        }
        match neighbors[2] {
            VonNeumann::Transmission(nbor_ty, dir, ex) => {
                if dir == Direction::North {
                    if nbor_ty == ty {
                        return None;
                    }
                    excited |= ex == Excitation::Excited;
                }
            }
            _ => (),
        }
        match neighbors[3] {
            VonNeumann::Transmission(nbor_ty, dir, ex) => {
                if dir == Direction::East {
                    if nbor_ty == ty {
                        return None;
                    }
                    excited |= ex == Excitation::Excited;
                }
            }
            _ => (),
        }
        Some(Excitation::from(excited))
    }

    #[inline]
    fn transition(neighbors: &Vec<Self>, no_input: Sensitised, input: Sensitised) -> Self {
        if VonNeumann::any_input(neighbors) {
            Self::Transition(no_input)
        } else {
            Self::Transition(input)
        }
    }

    #[inline]
    fn transition_partial_end(
        neighbors: &Vec<Self>,
        no_input: Sensitised,
        input: (TransmissionType, Direction),
    ) -> Self {
        if VonNeumann::any_input(neighbors) {
            Self::Transition(no_input)
        } else {
            Self::Transmission(input.0, input.1, Excitation::Quiescent)
        }
    }

    #[inline]
    fn transition_end_confluent(neighbors: &Vec<Self>) -> Self {
        if VonNeumann::any_input(neighbors) {
            Self::Transmission(
                TransmissionType::Ordinary,
                Direction::South,
                Excitation::Quiescent,
            )
        } else {
            Self::Confluent(Excitation::Quiescent, Excitation::Quiescent)
        }
    }

    #[inline]
    fn transition_end(
        neighbors: &Vec<Self>,
        no_input: (TransmissionType, Direction),
        input: (TransmissionType, Direction),
    ) -> Self {
        if VonNeumann::any_input(neighbors) {
            Self::Transmission(no_input.0, no_input.1, Excitation::Quiescent)
        } else {
            Self::Transmission(input.0, input.1, Excitation::Quiescent)
        }
    }
}

impl Default for VonNeumann {
    fn default() -> Self {
        Self::Ground
    }
}

impl Cell for VonNeumann {
    type Location = ILoc2D;
    type Encoded = u32;

    fn encode(&self) -> Self::Encoded {
        return 0;
    }

    fn decode(encoded: &Self::Encoded) -> Self {
        return Self::Ground;
    }

    fn neighborhood(loc: Self::Location) -> Vec<Self::Location> {
        von_neumann_neighborhood(loc)
    }

    fn update<U: Universe<Cell = Self, Location = Self::Location>>(
        &self,
        universe: &U,
        loc: U::Location,
    ) -> Self {
        // TODO Use map or something functional to get the list of neighbors
        let mut neighbors = Vec::with_capacity(4);
        for nbor in Self::neighborhood(loc) {
            neighbors.push(universe.get(nbor))
        }

        match self {
            VonNeumann::Ground => {
                if VonNeumann::any_input(&neighbors) {
                    VonNeumann::Transition(Sensitised::S)
                } else {
                    VonNeumann::Ground
                }
            }

            VonNeumann::Transition(state) => match state {
                Sensitised::S => Self::transition(&neighbors, Sensitised::S0, Sensitised::S1),
                Sensitised::S0 => Self::transition(&neighbors, Sensitised::S00, Sensitised::S01),
                Sensitised::S00 => Self::transition_partial_end(
                    &neighbors,
                    Sensitised::S000,
                    (TransmissionType::Ordinary, Direction::West),
                ),
                Sensitised::S000 => Self::transition_end(
                    &neighbors,
                    (TransmissionType::Ordinary, Direction::East),
                    (TransmissionType::Ordinary, Direction::North),
                ),
                Sensitised::S01 => Self::transition_end(
                    &neighbors,
                    (TransmissionType::Ordinary, Direction::South),
                    (TransmissionType::Special, Direction::East),
                ),
                Sensitised::S1 => Self::transition(&neighbors, Sensitised::S10, Sensitised::S11),
                Sensitised::S10 => Self::transition_end(
                    &neighbors,
                    (TransmissionType::Special, Direction::North),
                    (TransmissionType::Special, Direction::West),
                ),
                Sensitised::S11 => Self::transition_end_confluent(&neighbors),
            },

            VonNeumann::Confluent(now, next) => {}

            VonNeumann::Transmission(ty, dir, excite) => {
                match Self::transmission_update(&neighbors, ty) {
                    
                }
            },
        }
    }
}
