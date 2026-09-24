//! No orphan numbers: every health, damage and cooldown the mod declares is a
//! share of its declared baseline, not a literal.
//!
//! A literal is invisible to a global retune and drifts off the level curve the
//! moment progression moves it. So for any unit and any ability, the number must
//! reach one of the mod's `baseline.*` curves.

use bolero::{TypeGenerator, check};
use stormlight_mod_sdk::abi::descriptors::Registration;
use stormlight_mod_sdk::abi::ids::CurveId;
use stormlight_mod_sdk::abi::impacts::Impact;
use stormlight_mod_sdk::abi::math::Value;

use stormlight_mod_example::__stormlight_registration;

#[derive(Debug, TypeGenerator)]
struct Scenario {
    unit: u8,
    ability: u8,
}

/// Whether `value` reads one of the registration's `baseline.*` curves.
fn reads_baseline(reg: &Registration, value: &Value) -> bool {
    let is_baseline = |id: &CurveId| {
        reg.names.curves.get(usize::from(id.0)).is_some_and(|n| n.starts_with("baseline."))
    };
    match value {
        Value::Curve(id, x) => is_baseline(id) || reads_baseline(reg, x),
        Value::Bin(_, a, b) => reads_baseline(reg, a) || reads_baseline(reg, b),
        Value::Clamp { v, lo, hi } => [v, lo, hi].iter().any(|x| reads_baseline(reg, x)),
        Value::Const(_) | Value::Read(_) | Value::ScaleCtx => false,
    }
}

#[test]
fn every_health_damage_and_cooldown_is_a_share_of_the_baseline() {
    let reg = __stormlight_registration();
    let cooldown = reg.names.params.iter().position(|n| n == "cooldown");

    check!().with_type::<Scenario>().for_each(|s| {
        let unit = &reg.units[usize::from(s.unit) % reg.units.len()];
        assert!(reads_baseline(&reg, &unit.health), "unit health is a literal");

        let ability = &reg.abilities[usize::from(s.ability) % reg.abilities.len()];
        for (param, value) in &ability.params.0 {
            if Some(usize::from(param.0)) == cooldown {
                assert!(reads_baseline(&reg, value), "a cooldown is a literal");
            }
        }
        for impact in &ability.on_cast {
            let Impact::Spawn { body, .. } = impact else { continue };
            for hit in &body.on_hit {
                if let Impact::Damage { amount, .. } = hit {
                    assert!(reads_baseline(&reg, amount), "missile damage is a literal");
                }
            }
        }
    });
}
