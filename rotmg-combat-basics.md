# Realm of the Mad God Combat Basics

Researched from the linked sources on 2026-03-23.

## What RotMG Combat Is

Realm of the Mad God combat is real-time, aim-based, and permanent-death. In practice, that means:

- You move constantly while aiming and firing in real time.
- You are usually managing three things at once: your position, incoming bullet patterns, and your ability/resource timing.
- Survival matters more than squeezing out a little extra damage, because death deletes the character and everything it was carrying.

## The Core Combat Loop

At the most basic level, every fight is:

1. Stay at a safe range for your class and weapon.
2. Aim your shots while dodging enemy projectiles.
3. Use your class ability when it creates value: burst, mobility, healing, crowd control, armor break, expose, stun, etc.
4. Watch HP, status effects, and escape options.
5. Back off, potion, or nexus before a recoverable mistake becomes a dead character.

Two practical consequences come from this:

- RotMG is more about pattern recognition and movement discipline than standing still and trading damage.
- Greedy play gets punished hard. If you are unsure whether you can tank something, assume you should dodge it.

## Your Combat Stats

The main combat stats from the character-stat reference are:

- `HP`: your health pool. At 0 HP, you die.
- `MP`: your mana pool. Class abilities spend MP.
- `ATT`: increases weapon damage only. It does not increase most ability damage.
- `DEF`: reduces incoming damage point-for-point, but only up to a 90% reduction cap. Every hit still deals at least 10% of its listed damage.
- `SPD`: increases movement speed.
- `DEX`: increases attack speed.
- `VIT`: increases HP regeneration and reduces time spent in the in-combat recovery penalty state.
- `WIS`: increases MP regeneration and improves many class abilities through WisMod thresholds.

Useful formulas from the stat page:

- Weapon damage multiplier: `0.5 + ATT / 50`
- Defense rule: `incoming damage - DEF`, with a floor at 10% of the shot's damage
- Base movement speed: `4 tiles/sec`, scaled upward by `SPD`
- Base attack speed: `1.5 attacks/sec`, scaled upward by `DEX`

What this means in actual play:

- `ATT` and `DEX` are your main weapon-DPS stats.
- `DEF`, `SPD`, and `VIT` are often what keep a character alive long enough to matter.
- `WIS` matters a lot more on ability-heavy classes than new players often expect.

## Damage, Defense, And Why Big Shots Still Hurt

Defense in RotMG is strong against many small hits and much weaker against heavy-damage attacks.

- If you are being chipped by lots of low-damage bullets, `DEF` can massively cut that pressure.
- If you are being hit by large boss shots, defense still helps, but the minimum-10%-damage rule means you cannot become truly tanky just by stacking DEF.
- Armor-piercing attacks ignore defense entirely, so bullet color or boss familiarity matters.

That is why endgame combat usually feels like dodging first and tanking second.

## Vital Combat: Recovery Is Worse Right After You Get Hit

Modern RotMG uses the `Vital Combat` system.

- After taking enough damage to trigger it, you enter `in-combat (IC)`.
- In IC, regeneration from `VIT` and `WIS` is cut to 50%.
- Pet-based HP/MP recovery is delayed by 2 more seconds.
- If you avoid taking enough damage for a period of time, you return to `out-of-combat (OOC)`, where recovery works normally again.

Important details:

- The amount of damage needed to trigger IC scales with `DEF`.
- Time spent in IC starts at 7 seconds and is reduced by Vitality at `0.04 seconds per VIT`, before exaltation reductions.
- High `VIT` does not just improve raw regeneration; it also gets you back to normal recovery faster.

Practical takeaway:

- A single mistake is often survivable.
- Repeated chip damage is what kills you, because it keeps you in IC and ruins your recovery.

## Abilities Matter As Much As Weapons

RotMG combat is not just primary-fire DPS.

- Weapons give your constant damage output.
- Abilities create the class identity: stun, heal, speed, invisibility, trap, expose, burst damage, decoys, summons, dashes, and more.
- `WIS` increases the strength of many abilities, often through thresholds rather than perfectly smooth scaling.

The important beginner lesson is simple:

- Do not save abilities forever.
- Do not spam them blindly either.
- Use them to solve the current problem: survival, control, burst, or team support.

## Status Effects That Matter Most In Combat

These are the ones that most directly change whether you should keep fighting or disengage.

### Dangerous Negative Statuses

- `Sick`: stops natural HP regeneration and also stops outside HP recovery. This is one of the most dangerous statuses in the game.
- `Silenced`: you cannot use your ability.
- `Quiet`: all MP regeneration stops.
- `Stunned`: you cannot use your weapon, though abilities still work.
- `Paralyzed`: you cannot move.
- `Slowed`: your movement speed drops to base speed, effectively wiping out your SPD advantage.
- `Weak`: your ATT is treated as 0 for damage calculation.
- `Armor Broken`: removes the benefit of positive DEF.
- `Bleeding`: drains HP over time and stops VIT-based HP regen, though it cannot kill by itself.
- `Unstable`: adds random shot deviation, which wrecks accuracy.
- `Petrify`: you cannot move or use weapons, but most abilities still work and you take 10% less damage.

### Important Positive Statuses

- `Damaging`: weapon damage dealt increases by 25%.
- `Berserk`: attack speed increases by 25%.
- `Speedy`: movement speed increases by 50%.
- `Healing`: adds 20 HP/sec regeneration.
- `Armored`: increases DEF by 50% during damage calculations.
- `Exposed` on enemies: reduces enemy DEF by 20 and can push it below zero.

### Invulnerability Terms That Matter In Boss Fights

- `Invulnerable`: the target takes no attack damage, but can still be affected by status effects and environmental damage.
- `Invincible`: attacks pass through harmlessly and do not connect at all.
- `Stasis` on enemies: they cannot move or attack, but they also cannot be hit or damaged while stasised.

## Positioning Basics

Most RotMG deaths come from bad positioning, not from misunderstanding formulas.

- Fight near the edge of your safe weapon range instead of hugging enemies for no reason.
- Keep moving. Straight-line panic movement is easy for spiral, shotgun, and predictive shots to punish.
- Leave yourself an exit lane. Getting cornered is how small mistakes become deaths.
- Learn whether a pattern is asking you to rotate, sidestep, back off, or cut through a gap.
- Respect armor-piercing and status-inflicting shots more than ordinary chip damage.

For newer players, longer-range play is usually more forgiving. That is an inference from how the stat and status systems interact with bullet-hell movement, not a direct line from the sources.

## Survival Habits That Matter Immediately

The getting-started guidance and the combat rules above point to a few non-negotiable habits:

- Put `Return to Nexus` on a key you can hit instantly.
- Keep health and mana potions on easy keys near movement.
- Do not teleport blind into crowded high-level areas just because many players are there.
- If your HP is low and you are still in IC, assume your recovery is worse than it looks.
- Back out when you are `Sick`, `Silenced`, `Paralyzed`, or boxed into bad terrain.
- Treat every new dungeon or boss as lethal until you learn which shots are the real threats.

## Simple Mental Model

If you want one compact way to think about RotMG combat, use this:

- `Dodge first`
- `Maintain recovery`
- `Use ability with purpose`
- `Know which statuses force a retreat`
- `Nexus before greed kills the character`

## Sources

- [RealmEye - Getting Started](https://www.realmeye.com/wiki/getting-started)
- [RealmEye - Character Stats](https://www.realmeye.com/wiki/character-stats)
- [RealmEye - Status Effects](https://www.realmeye.com/wiki/status-effects)
- [RealmEye - Vital Combat](https://www.realmeye.com/wiki/vital-combat)
- [RealmEye - Weapons](https://www.realmeye.com/wiki/weapons)
- [RealmEye - Classes](https://www.realmeye.com/wiki/classes)
