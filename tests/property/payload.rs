//! A thrown body hits the other side, never its own.
//!
//! For any ability of the mod, every missile it spawns collides only with
//! enemies, and does something when it lands. A missile filtered to allies (or to
//! everyone) would strike its own caster the frame it spawns.

use bolero::{TypeGenerator, check};
use stormlight_mod_sdk::abi::common::Affiliation;
use stormlight_mod_sdk::abi::impacts::Impact;

use stormlight_mod_example::__stormlight_registration;

#[derive(Debug, TypeGenerator)]
struct Scenario {
    ability: u8,
}

#[test]
fn every_missile_targets_enemies_and_lands_with_an_effect() {
    let reg = __stormlight_registration();
    assert!(!reg.abilities.is_empty(), "the mod declares no ability");

    check!().with_type::<Scenario>().for_each(|s| {
        let ability = &reg.abilities[usize::from(s.ability) % reg.abilities.len()];
        for impact in &ability.on_cast {
            let Impact::Spawn { body, .. } = impact else { continue };
            assert_eq!(body.collision.filter.affiliation, Affiliation::Enemies);
            assert!(!body.collision.filter.include_dead, "a missile stopped by a corpse");
            assert!(!body.on_hit.is_empty(), "a missile that lands and does nothing");
        }
    });
}
