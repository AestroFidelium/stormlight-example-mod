//! `example` — a minimal, complete Stormlight gameplay mod.
//!
//! One unit a player can drive, with two abilities that between them touch most
//! of what a mod declares: an interned vocabulary, a resource pool, a projectile
//! with an on-hit payload, and a timed status effect.
//!
//! - **spark** — a skillshot: spend energy to throw a straight-line missile that
//!   damages the first enemy it touches.
//! - **guard** — a self-cast: spend energy to raise the caster's armor for a few
//!   seconds.
//!
//! Health, damage, cooldowns and costs are not bare literals. The mod declares
//! once what an *average* unit has (a [`Baseline`], as curves over level) and
//! writes those numbers as shares of it: `base.damage(pct(18))` is "about six
//! hits against an average unit", at every level. Retuning the baseline retunes
//! every one of them.
//!
//! Nothing here runs per frame. The mod *describes* its content once, at load, and
//! the engine interprets those descriptors on the authoritative server. Every
//! handle below is local to this mod; the host remaps them to global ids when it
//! adopts the registration.

// `no_std` for the wasm guest; `std` on the host so the native tests link it.
#![cfg_attr(target_arch = "wasm32", no_std)]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use stormlight_mod_sdk::abi::abilities::{AbilityDescriptor, CastSpec, Cost, Params, Targeting};
use stormlight_mod_sdk::abi::behaviors::{
    BuffSpec, ModOp, Modifier, Reapply, StackScope, Stacking,
};
use stormlight_mod_sdk::abi::common::{Affiliation, ImpactTarget, TargetFilter};
use stormlight_mod_sdk::abi::conditions::Condition;
use stormlight_mod_sdk::abi::descriptors::Curve;
use stormlight_mod_sdk::abi::ids::{
    AbilityId, BuffId, DamageTypeId, ParamId, ResourceId, Slot, StatId, UnitId,
};
use stormlight_mod_sdk::abi::impacts::{DamageFlags, Impact, SpawnAnchor, SpawnPattern};
use stormlight_mod_sdk::abi::math::Value;
use stormlight_mod_sdk::abi::missiles::{BodyDescriptor, BodyFlags, BodyKind, CollisionSpec};
use stormlight_mod_sdk::abi::units::{ResourcePool, UnitDescriptor};
use stormlight_mod_sdk::balance::{Baseline, BaselineSpec, flat, pct, share_of};
use stormlight_mod_sdk::context::ModContext;
use stormlight_mod_sdk::register_mod;

register_mod!(|ctx: &mut ModContext| {
    // Names, not numbers. A name the engine reserves ("cooldown", "move_speed",
    // "armor") collapses onto the engine's own id at adoption, so the numbers
    // below reach the same systems the engine drives.
    let cooldown = ctx.param("cooldown");
    let move_speed = ctx.stat("move_speed");
    let armor = ctx.stat("armor");
    let energy = ctx.resource("energy");
    let arcane = ctx.damage_type("arcane");

    // What an average unit has: the scale health, damage and cooldowns are read on.
    let base = Baseline::declare(
        ctx,
        BaselineSpec {
            health: Curve { points: vec![[1.0, 500.0], [20.0, 1400.0]] },
            move_speed: flat(4.5),
            cooldown: flat(6.0),
        },
    );

    // `playable` is an engine-reserved capability class: tagging a unit with a
    // tag in it is how a mod says "a player may drive this".
    let (hero, _) = ctx.register_tag_class("hero", "playable");

    let guarded = ctx.buff("guarded", guarded(armor));
    let spark = ctx.ability("spark", spark(&base, cooldown, energy, arcane));
    let guard = ctx.ability("guard", guard(&base, cooldown, energy, guarded));

    ctx.unit(
        "sentinel",
        UnitDescriptor {
            id: UnitId(0),
            // A little frailer than average, and exactly as quick.
            health: base.health(pct(90)),
            stats: vec![(move_speed, base.move_speed(pct(100))), (armor, Value::Const(0.0))],
            tags: vec![hero],
            abilities: vec![(Slot(0), spark), (Slot(1), guard)],
            resources: vec![ResourcePool {
                id: energy,
                max: Value::Const(ENERGY),
                regen: Value::Const(10.0),
            }],
            grant_slots: Vec::new(),
            talents: Vec::new(),
            talent_tree: None,
            respawn: None,
            progression: None,
            turn_rate: None,
            tasks: Vec::new(),
            attack: None,
        },
    );
});

/// The energy pool. Costs are shares of it, so resizing it keeps every cast
/// affordable the same number of times.
const ENERGY: f32 = 100.0;

/// **spark** — throw a missile along the aimed direction.
fn spark(
    base: &Baseline,
    cooldown: ParamId,
    energy: ResourceId,
    arcane: DamageTypeId,
) -> AbilityDescriptor {
    let missile = BodyDescriptor {
        kind: BodyKind::Missile {
            speed: Value::Const(18.0),
            range: Value::Const(12.0),
            homing: false,
            pierce: Value::Const(0.0),
        },
        // What happens to whatever it touches: the payload travels with the body.
        // About six hits to down an average unit, at any level.
        on_hit: vec![Impact::Damage {
            amount: base.damage(pct(18)),
            dtype: arcane,
            target: ImpactTarget::ResolvedTarget,
            flags: DamageFlags::default(),
        }],
        collision: CollisionSpec {
            filter: TargetFilter {
                affiliation: Affiliation::Enemies,
                require_tags: Vec::new(),
                exclude_tags: Vec::new(),
                include_dead: false,
            },
            pierce: Value::Const(0.0),
            through_walls: false,
        },
        height: Value::Const(1.0),
        on_spawn: Vec::new(),
        on_expire: Vec::new(),
        flags: BodyFlags::default(),
    };

    AbilityDescriptor {
        id: AbilityId(0),
        // Half the average cooldown: the button pressed most often.
        params: Params(vec![(cooldown, base.cooldown(pct(50)))]),
        targeting: Targeting::Vector,
        // A short wind-up the caster can walk through: readable, never rooting.
        cast: CastSpec::Cast { time: Value::Const(0.2), movable: true },
        cost: vec![Cost::Resource { res: energy, amount: share_of(pct(25), Value::Const(ENERGY)) }],
        on_cast: vec![Impact::Spawn {
            body: missile,
            at: SpawnAnchor::Caster,
            count: Value::Const(1.0),
            pattern: SpawnPattern::Single,
        }],
        cast_gate: Condition::Always,
        on_cast_start: Vec::new(),
        tags: Vec::new(),
    }
}

/// **guard** — apply [`guarded`] to the caster.
fn guard(
    base: &Baseline,
    cooldown: ParamId,
    energy: ResourceId,
    guarded: BuffId,
) -> AbilityDescriptor {
    AbilityDescriptor {
        id: AbilityId(0),
        params: Params(vec![(cooldown, base.cooldown(pct(160)))]),
        targeting: Targeting::SelfCast,
        cast: CastSpec::Instant,
        cost: vec![Cost::Resource { res: energy, amount: share_of(pct(40), Value::Const(ENERGY)) }],
        on_cast: vec![Impact::ApplyModifiers {
            buff: guarded,
            stacks: Value::Const(1.0),
            duration_override: None,
            target: ImpactTarget::Caster,
        }],
        cast_gate: Condition::Always,
        on_cast_start: Vec::new(),
        tags: Vec::new(),
    }
}

/// **guarded** — +30 armor for four seconds. Pressing again refreshes the timer
/// rather than stacking, and the effect ends with the unit's death.
fn guarded(armor: StatId) -> BuffSpec {
    BuffSpec {
        id: BuffId(0),
        duration: Some(Value::Const(4.0)),
        stacking: Stacking { on_reapply: Reapply::RefreshDuration, scope: StackScope::Global },
        max_stacks: 1,
        modifiers: vec![Modifier { stat: armor, op: ModOp::AddFlat, value: Value::Const(30.0) }],
        drop_on_death: true,
        tags: Vec::new(),
        reactions: Vec::new(),
        on_apply: Vec::new(),
        on_expire: Vec::new(),
        on_remove: Vec::new(),
    }
}
