use std::collections::HashMap;
use crate::pokemon::Pokemon;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ItemKind {
    Potion,
    SuperPotion,
    FullRestore,
    Revive,
    Ether,
    MaxEther,
}

impl ItemKind {
    pub fn name(&self) -> &'static str {
        match self {
            ItemKind::Potion => "Potion",
            ItemKind::SuperPotion => "Super Potion",
            ItemKind::FullRestore => "Full Restore",
            ItemKind::Revive => "Revive",
            ItemKind::Ether => "Ether",
            ItemKind::MaxEther => "Max Ether",
        }
    }

    pub fn price(&self) -> u32 {
        match self {
            ItemKind::Potion => 300,
            ItemKind::SuperPotion => 700,
            ItemKind::FullRestore => 3000,
            ItemKind::Revive => 1500,
            ItemKind::Ether => 1200,
            ItemKind::MaxEther => 2000,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ItemKind::Potion => "Ripristina 20 HP",
            ItemKind::SuperPotion => "Ripristina 50 HP",
            ItemKind::FullRestore => "Ripristina tutti gli HP",
            ItemKind::Revive => "Ripristina un Pokémon KO a metà HP",
            ItemKind::Ether => "Ripristina 10 PP a tutte le mosse",
            ItemKind::MaxEther => "Ripristina tutti i PP a tutte le mosse",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Inventory {
    pub items: HashMap<ItemKind, u32>,
    pub money: u32,
}

#[derive(Debug, PartialEq)]
pub enum UseItemError {
    NotOwned,
    InvalidTarget,
}

impl Inventory {
    pub fn new(starting_money: u32) -> Self {
        Self { items: HashMap::new(), money: starting_money }
    }

    pub fn quantity(&self, kind: &ItemKind) -> u32 {
        *self.items.get(kind).unwrap_or(&0)
    }

    pub fn add(&mut self, kind: ItemKind, qty: u32) {
        *self.items.entry(kind).or_insert(0) += qty;
    }

    pub fn buy(&mut self, kind: ItemKind, qty: u32) -> bool {
        let cost = kind.price() * qty;
        if self.money < cost {
            return false;
        }
        self.money -= cost;
        self.add(kind, qty);
        true
    }

    /// Usa un item su un Pokémon. Restituisce Err se l'item non è in possesso
    /// o se il target non è valido (es. Revive su Pokémon non KO).
    pub fn use_item(&mut self, kind: &ItemKind, target: &mut Pokemon, move_index: Option<usize>) -> Result<(), UseItemError> {
        if self.quantity(kind) == 0 {
            return Err(UseItemError::NotOwned);
        }
        match kind {
            ItemKind::Potion => {
                if target.is_fainted() {
                    return Err(UseItemError::InvalidTarget);
                }
                target.heal(20);
            }
            ItemKind::SuperPotion => {
                if target.is_fainted() {
                    return Err(UseItemError::InvalidTarget);
                }
                target.heal(50);
            }
            ItemKind::FullRestore => {
                if target.is_fainted() {
                    return Err(UseItemError::InvalidTarget);
                }
                target.full_heal();
            }
            ItemKind::Revive => {
                if !target.is_fainted() {
                    return Err(UseItemError::InvalidTarget);
                }
                let half = target.max_hp() / 2;
                target.current_hp = half.max(1);
            }
            ItemKind::Ether => {
                for m in &mut target.moves {
                    m.current_pp = (m.current_pp + 10).min(m.max_pp);
                }
            }
            ItemKind::MaxEther => {
                for m in &mut target.moves {
                    m.current_pp = m.max_pp;
                }
            }
        }
        *self.items.get_mut(kind).unwrap() -= 1;
        if self.quantity(kind) == 0 {
            self.items.remove(kind);
        }
        Ok(())
    }

    pub fn earn(&mut self, amount: u32) {
        self.money += amount;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pokemon::Stats;
    use crate::types::Type;
    use crate::moves::{Move, MoveCategory};

    fn pikachu_pieno() -> Pokemon {
        Pokemon::new("Pikachu", Type::Electric, None, Stats::new(35, 55, 40, 50, 50, 90), 112, 50)
    }

    fn pikachu_ko() -> Pokemon {
        let mut p = pikachu_pieno();
        p.current_hp = 0;
        p
    }

    #[test]
    fn buy_scala_i_soldi() {
        let mut inv = Inventory::new(1000);
        assert!(inv.buy(ItemKind::Potion, 2));
        assert_eq!(inv.money, 400);
        assert_eq!(inv.quantity(&ItemKind::Potion), 2);
    }

    #[test]
    fn buy_fallisce_senza_soldi() {
        let mut inv = Inventory::new(100);
        assert!(!inv.buy(ItemKind::FullRestore, 1));
        assert_eq!(inv.money, 100);
    }

    #[test]
    fn potion_cura_20_hp() {
        let mut inv = Inventory::new(1000);
        inv.add(ItemKind::Potion, 1);
        let mut p = pikachu_pieno();
        p.take_damage(30);
        let hp_prima = p.current_hp;
        inv.use_item(&ItemKind::Potion, &mut p, None).unwrap();
        assert_eq!(p.current_hp, hp_prima + 20);
        assert_eq!(inv.quantity(&ItemKind::Potion), 0);
    }

    #[test]
    fn potion_non_funziona_su_ko() {
        let mut inv = Inventory::new(1000);
        inv.add(ItemKind::Potion, 1);
        let mut p = pikachu_ko();
        assert_eq!(inv.use_item(&ItemKind::Potion, &mut p, None), Err(UseItemError::InvalidTarget));
        assert_eq!(inv.quantity(&ItemKind::Potion), 1);
    }

    #[test]
    fn revive_funziona_solo_su_ko() {
        let mut inv = Inventory::new(2000);
        inv.add(ItemKind::Revive, 1);
        let mut p = pikachu_ko();
        inv.use_item(&ItemKind::Revive, &mut p, None).unwrap();
        assert!(p.current_hp > 0);
        assert_eq!(p.current_hp, p.max_hp() / 2);
    }

    #[test]
    fn revive_fallisce_su_pokemon_vivo() {
        let mut inv = Inventory::new(2000);
        inv.add(ItemKind::Revive, 1);
        let mut p = pikachu_pieno();
        let result = inv.use_item(&ItemKind::Revive, &mut p, None);
        assert!(result.is_err());
        assert_eq!(inv.quantity(&ItemKind::Revive), 1); // non consumato
    }

    #[test]
    fn ether_ripristina_10pp_a_tutte_le_mosse() {
        let mut inv = Inventory::new(2000);
        inv.add(ItemKind::Ether, 1);
        let mut p = pikachu_pieno();
        let mut m1 = Move::new("Thunderbolt", Type::Electric, MoveCategory::Special, 90, 100, 15);
        let mut m2 = Move::new("Quick Attack", Type::Normal, MoveCategory::Physical, 40, 100, 30);
        m1.current_pp = 3;
        m2.current_pp = 5;
        p.add_move(m1);
        p.add_move(m2);
        inv.use_item(&ItemKind::Ether, &mut p, None).unwrap();
        assert_eq!(p.moves[0].current_pp, 13);
        assert_eq!(p.moves[1].current_pp, 15);
    }

    #[test]
    fn max_ether_ripristina_pp_completi_a_tutte_le_mosse() {
        let mut inv = Inventory::new(2000);
        inv.add(ItemKind::MaxEther, 1);
        let mut p = pikachu_pieno();
        let mut m1 = Move::new("Thunderbolt", Type::Electric, MoveCategory::Special, 90, 100, 15);
        let mut m2 = Move::new("Quick Attack", Type::Normal, MoveCategory::Physical, 40, 100, 30);
        m1.current_pp = 3;
        m2.current_pp = 5;
        p.add_move(m1);
        p.add_move(m2);
        inv.use_item(&ItemKind::MaxEther, &mut p, None).unwrap();
        assert_eq!(p.moves[0].current_pp, p.moves[0].max_pp);
        assert_eq!(p.moves[1].current_pp, p.moves[1].max_pp);
    }
}
