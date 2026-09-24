//! Every button the unit is given can be pressed.
//!
//! For any slot of the declared loadout: the ability bound there is one the mod
//! actually registered, and every resource it costs is a pool the unit carries,
//! deep enough to pay for it from full. A cost against a pool the unit lacks, or
//! one larger than the pool, is a button that never fires — the host accepts it,
//! so only a test catches it.

use bolero::{TypeGenerator, check};
use stormlight_mod_sdk::abi::abilities::Cost;
use stormlight_mod_sdk::abi::math::Value;

use stormlight_mod_example::__stormlight_registration;

#[derive(Debug, TypeGenerator)]
struct Scenario {
    unit: u8,
    slot: u8,
}

#[test]
fn every_bound_ability_exists_and_is_affordable_from_full() {
    let reg = __stormlight_registration();
    assert!(!reg.units.is_empty(), "the mod declares no unit");

    check!().with_type::<Scenario>().for_each(|s| {
        let unit = &reg.units[usize::from(s.unit) % reg.units.len()];
        assert!(!unit.abilities.is_empty(), "a unit with no abilities");
        let (_, bound) = unit.abilities[usize::from(s.slot) % unit.abilities.len()];

        let ability = reg
            .abilities
            .iter()
            .find(|a| a.id == bound)
            .unwrap_or_else(|| panic!("slot binds unregistered ability {bound:?}"));

        for cost in &ability.cost {
            let Cost::Resource { res, amount } = cost.clone() else { continue };
            let pool = unit
                .resources
                .iter()
                .find(|p| p.id == res)
                .unwrap_or_else(|| panic!("ability costs a pool the unit lacks: {res:?}"));
            let (Value::Const(price), Value::Const(max)) = (amount, pool.max.clone()) else {
                panic!("the example declares constant costs and pools");
            };
            assert!(price > 0.0, "a cost that pays nothing");
            assert!(price <= max, "costs {price}, but the pool holds only {max}");
        }
    });
}
