use std::fmt::Display;

use crate::types::*;
use serde::{Deserialize, Serialize};

use self::marker::OrderMarker;

pub mod marker;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PendingOrder {
    MoveTo(SquadUuid, Option<OrderMarkerIndex>, Vec<WorldPoint>),
    MoveFastTo(SquadUuid, Option<OrderMarkerIndex>, Vec<WorldPoint>),
    SneakTo(SquadUuid, Option<OrderMarkerIndex>, Vec<WorldPoint>),
    Defend(SquadUuid),
    Hide(SquadUuid),
    EngageOrFire(SquadUuid),
}

impl PendingOrder {
    pub fn expect_path_finding(&self) -> bool {
        matches!(
            self,
            PendingOrder::MoveTo(_, _, _)
                | PendingOrder::MoveFastTo(_, _, _)
                | PendingOrder::SneakTo(_, _, _)
        )
    }

    pub fn squad_index(&self) -> &SquadUuid {
        match self {
            PendingOrder::MoveTo(squad_index, _, _) => squad_index,
            PendingOrder::MoveFastTo(squad_index, _, _) => squad_index,
            PendingOrder::SneakTo(squad_index, _, _) => squad_index,
            PendingOrder::Defend(squad_index) => squad_index,
            PendingOrder::Hide(squad_index) => squad_index,
            PendingOrder::EngageOrFire(squad_index) => squad_index,
        }
    }

    pub fn cached_points(&self) -> Vec<WorldPoint> {
        match self {
            PendingOrder::MoveTo(_, _, cached_points) => cached_points.clone(),
            PendingOrder::MoveFastTo(_, _, cached_points) => cached_points.clone(),
            PendingOrder::SneakTo(_, _, cached_points) => cached_points.clone(),
            PendingOrder::Defend(_) => vec![],
            PendingOrder::Hide(_) => vec![],
            PendingOrder::EngageOrFire(_) => vec![],
        }
    }

    pub fn order_marker_index(&self) -> &Option<OrderMarkerIndex> {
        match self {
            PendingOrder::MoveTo(_, order_marker_index, _) => order_marker_index,
            PendingOrder::MoveFastTo(_, order_marker_index, _) => order_marker_index,
            PendingOrder::SneakTo(_, order_marker_index, _) => order_marker_index,
            PendingOrder::Defend(_) => &None,
            PendingOrder::Hide(_) => &None,
            PendingOrder::EngageOrFire(_) => &None,
        }
    }

    pub fn push_cache_point(&mut self, new_point: WorldPoint) {
        match self {
            PendingOrder::MoveTo(_, _, points)
            | PendingOrder::MoveFastTo(_, _, points)
            | PendingOrder::SneakTo(_, _, points) => points.push(new_point),
            _ => {}
        }
    }

    pub fn is_hide(&self) -> bool {
        matches!(self, Self::Hide(_))
    }
}

impl Display for PendingOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PendingOrder::MoveTo(_, _, _) => f.write_str("MoveTo"),
            PendingOrder::MoveFastTo(_, _, _) => f.write_str("MoveFastTo"),
            PendingOrder::SneakTo(_, _, _) => f.write_str("SneakTo"),
            PendingOrder::Defend(_) => f.write_str("Defend"),
            PendingOrder::Hide(_) => f.write_str("Hide"),
            PendingOrder::EngageOrFire(_) => f.write_str("EngageOrFire"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Order {
    Idle,
    MoveTo(WorldPaths, Option<Box<Order>>),
    MoveFastTo(WorldPaths, Option<Box<Order>>),
    SneakTo(WorldPaths, Option<Box<Order>>),
    Defend(Angle),
    Hide(Angle),
    EngageSquad(SquadUuid),
    SuppressFire(WorldPoint),
}

impl Order {
    pub fn marker(&self) -> Option<OrderMarker> {
        match self {
            Order::MoveTo(_, _) => Some(OrderMarker::MoveTo),
            Order::MoveFastTo(_, _) => Some(OrderMarker::MoveFastTo),
            Order::SneakTo(_, _) => Some(OrderMarker::SneakTo),
            Order::Defend(_) => Some(OrderMarker::Defend),
            Order::Hide(_) => Some(OrderMarker::Hide),
            Order::EngageSquad(_) => Some(OrderMarker::EngageSquad),
            Order::SuppressFire(_) => Some(OrderMarker::SuppressFire),
            Order::Idle => None,
        }
    }

    pub fn angle(&self) -> Option<Angle> {
        match self {
            Order::MoveTo(_, _) | Order::MoveFastTo(_, _) | Order::SneakTo(_, _) => None,
            Order::Defend(angle) => Some(*angle),
            Order::Hide(angle) => Some(*angle),
            Order::SuppressFire(_) => None,
            Order::EngageSquad(_) => None,
            Order::Idle => None,
        }
    }

    pub fn reach_step(&mut self) -> bool {
        match self {
            Order::MoveTo(paths, _) | Order::MoveFastTo(paths, _) | Order::SneakTo(paths, _) => {
                paths
                    .remove_next_point()
                    .expect("Reach a move behavior implies containing point");

                if paths.next_point().is_none() {
                    return true;
                }
            }
            Order::Defend(_) => {}
            Order::Hide(_) => {}
            Order::Idle => {}
            Order::EngageSquad(_) => {}
            Order::SuppressFire(_) => {}
        }

        false
    }

    pub fn then(&self) -> Option<Order> {
        match self {
            Self::MoveTo(_, then) => then,
            Self::MoveFastTo(_, then) => then,
            Self::SneakTo(_, then) => then,
            _ => &None,
        }
        .clone()
        .map(|order| *order)
    }
}

impl Display for Order {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Order::MoveTo(_, _) => f.write_str("MoveTo"),
            Order::MoveFastTo(_, _) => f.write_str("MoveFastTo"),
            Order::SneakTo(_, _) => f.write_str("SneakTo"),
            Order::Defend(_) => f.write_str("Defend"),
            Order::Hide(_) => f.write_str("Hide"),
            Order::Idle => f.write_str("Idle"),
            Order::EngageSquad(_) => f.write_str("Engage"),
            Order::SuppressFire(_) => f.write_str("SuppressFire"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{WorldPath, WorldPaths};

    fn move_order(points: Vec<WorldPoint>) -> Order {
        Order::MoveTo(WorldPaths::new(vec![WorldPath::new(points)]), None)
    }

    #[test]
    fn marker_maps_every_order_variant() {
        let paths = WorldPaths::new(vec![WorldPath::new(vec![])]);

        assert_eq!(Order::Idle.marker(), None);
        assert_eq!(move_order(vec![]).marker(), Some(OrderMarker::MoveTo));
        assert_eq!(
            Order::MoveFastTo(paths.clone(), None).marker(),
            Some(OrderMarker::MoveFastTo)
        );
        assert_eq!(
            Order::SneakTo(paths, None).marker(),
            Some(OrderMarker::SneakTo)
        );
        assert_eq!(
            Order::Defend(Angle::zero()).marker(),
            Some(OrderMarker::Defend)
        );
        assert_eq!(Order::Hide(Angle::zero()).marker(), Some(OrderMarker::Hide));
        assert_eq!(
            Order::EngageSquad(SquadUuid(0)).marker(),
            Some(OrderMarker::EngageSquad)
        );
        assert_eq!(
            Order::SuppressFire(WorldPoint::new(0.0, 0.0)).marker(),
            Some(OrderMarker::SuppressFire)
        );
    }

    #[test]
    fn angle_is_only_defined_for_directional_holds() {
        assert_eq!(Order::Defend(Angle(1.5)).angle(), Some(Angle(1.5)));
        assert_eq!(Order::Hide(Angle(-0.5)).angle(), Some(Angle(-0.5)));
        assert_eq!(Order::Idle.angle(), None);
        assert_eq!(move_order(vec![]).angle(), None);
        assert_eq!(Order::SuppressFire(WorldPoint::new(0.0, 0.0)).angle(), None);
    }

    #[test]
    fn reach_step_consumes_path_and_reports_completion() {
        let mut order = move_order(vec![
            WorldPoint::new(0.0, 0.0),
            WorldPoint::new(10.0, 0.0),
            WorldPoint::new(20.0, 0.0),
        ]);

        assert!(!order.reach_step());
        assert!(!order.reach_step());
        assert!(
            order.reach_step(),
            "last consumed point completes the order"
        );
    }

    #[test]
    fn non_move_orders_never_reach() {
        assert!(!Order::Idle.reach_step());
        assert!(!Order::Defend(Angle::zero()).reach_step());
        assert!(!Order::Hide(Angle::zero()).reach_step());
        assert!(!Order::EngageSquad(SquadUuid(3)).reach_step());
    }

    #[test]
    fn then_returns_chained_followup_order() {
        let followup = Box::new(Order::Idle);
        let chained = Order::MoveTo(
            WorldPaths::new(vec![WorldPath::new(vec![])]),
            Some(followup),
        );
        assert!(matches!(chained.then(), Some(Order::Idle)));

        assert_eq!(Order::Idle.then(), None);
        assert_eq!(Order::Defend(Angle::zero()).then(), None);
    }
}
