# Twilight Archmage Projectile Catalog

Source page: [RealmEye - Twilight Archmage](https://www.realmeye.com/wiki/twilight-archmage)

This file condenses the attack tables from the page into grouped projectile families. If a projectile name appears with multiple variants on the page, the rows below merge those variants into one line.

Notes:

- `?` means the source table did not specify a range.
- `Boss` includes rows used by the standard Twilight Archmage fight and any extra boss-table rows marked as Nox-only on the page.
- Tempest's Paralyze stars are described in the phase text, but do not have a numeric stat row on the page's projectile table.

## Quick Status / Threat Index

- Slowed: `Frost Twirl` (100 stationary variant), `Blue Ball of Hurt`, `Frost Burst` from Ice Portals
- Silenced: `Frost Twirl` (220 rotating variant), `Flame Explosion` and `Flame Explosion Animated`, `Ball of Substantial Hurt` (0-damage Nox variant), `Frost Missile` from Ice Portals, `Frost Explosion` from Ice Portals, `Light Blue AoE`
- Sick + Bleeding: `Flame Vortex`, `Dissipating Flame Wave`, Fire Portal `Flame Explosion`, Fire Portal `Flame Burst`
- Quiet: `Frost Twirl` (195 turning variant)
- Armor Broken: `Flame Vortex` (250 damage turning variant)
- Pet Stasis: Ice Portal `Frost Magic`, Ice Portal `Frost Twirl` (Nox-only variant)
- Armor-piercing families called out by the page: `Frost Twirl` stationary variant, several `Flame Explosion` variants, `Dissipating Flame Wave`, `Frost Missile` decelerating variant, `Frost Star`, `Frost Vortex`, Ice Portal `Frost Explosion`, Ice Portal `Frost Missile`, Nox-only Ice Portal `Frost Twirl`

## Boss Projectiles

| Projectile | Damage | Conditions | Speed | Range | Behavior / Notes |
|---|---:|---|---|---|---|
| Ball of Substantial Hurt | 0, 250 | Silenced for 2s on the 0-damage Nox variant | 0.5, 1, 1.5, 7.5, 8 | 29.49 to 32 | Large orb shot. Standard boss version accelerates after 0.4s to max speed 5, 7, or 9. Nox also has silence-only variants. |
| Blue Ball of Hurt | 100 | Slowed for 3s | 4 | 24 | Wavy shot with amplitude 3 and frequency 1. |
| Blue Shard | 225 | None | 5.5 | 33 | Spawned by Ice Spheres. |
| Directed Flame | 110 | None | 3.5 | 18.18 | Accelerates after 0.7s at 4 tiles/sec^2 to max speed 10. |
| Dissipating Flame Wave | 150 | Sick for 2s, Bleeding for 2s | 2 | 2.6 | Armor piercing, very short-lived wave. |
| Flame Explosion | 120, 130 | Silenced for 2s and Sick for 1s on some variants | 5, 7, 8.5 | 15, 31.5, 38.25, ? | Armor piercing. Faces direction. Appears as straight fire blasts and delayed circular-motion variants. |
| Flame Explosion Animated | 250 | Silenced for 2s, Sick for 1s | 1.5 to 7 | ? | Nox-only fire blast that enters circular motion after 2s, turning either +60 or -60 degrees before circling. |
| Flame Twirl | 185 | None | 3, 4 | ? | Turning fireball/twirl family. Some variants start still and accelerate rapidly to 3.5, then curve clockwise or counterclockwise. |
| Flame Vortex | 240, 250 | Sick for 3s and Bleeding for 3s, or Armor Broken for 4s | 3 | 15, ? | Standard vortex is a 240-damage status shot. The 250-damage version turns at -110 degrees/sec and inflicts Armor Broken. |
| Frost Burst | 140 | None | 0 | 12.89 to 14.99 | Starts stationary, then accelerates after 0.2 to 0.8s to max speed 3.5. |
| Frost Explosion | 120 | None | 6 | 15 | Straight ice blast used as part of beam / portal pressure patterns. |
| Frost Missile | 100, 140, 180 | None | 0, 5, 7, 7.5 | 10.03 to 25.46 | Includes stationary-start missiles that accelerate to 6, plus Nox-only wavy armor-piercing variants and a decelerating armor-piercing variant. |
| Frost Star | 175, 250 | None | 2.5, 3, 5 | 3 to 25, ? | Armor piercing star family. Includes decelerating stars that stop, straight heavy stars, and delayed circular-motion stars. |
| Frost Twirl | 100, 195, 220 | Slowed for 3.4s, Quiet for 4s, Silenced for 5s | 0, 4, 4.5, 6 | 0, ? | Covers stationary spinner rings, turning quiet-inducing streams, and heavier silencing rotating streams. Some variants are armor piercing and have lifetime 3s or 6s. |
| Frost Vortex | 270 | None | 3.5 | ? | Armor piercing, faces direction, begins turning +/-90 degrees after 4.2s and stops turning at 4.4s. |
| Frost Wave | 120 | None | 3.5 | 15.75 | Straight piercing ice wave. |

## Fire Portal Projectiles

| Projectile | Damage | Conditions | Speed | Range | Behavior / Notes |
|---|---:|---|---|---|---|
| Flame Burst | 120, 240 | Sick for 2.6s, Bleeding for 2.6s | 3, 3.5, 4 | 3.9, 15.2, 24.5 | Standard Fire Portal shot family. Short-range and mid-range burst variants share the same status package. |
| Flame Explosion | 130 | Sick for 2.6s, Bleeding for 2.6s | 1.5 | 49.63 | Portal-fired long-range fire shot. The page lists acceleration `-4 tiles/sec^2 after 1.4s` and `Max. Speed: 8`. |

## Ice Portal Projectiles

| Projectile | Damage | Conditions | Speed | Range | Behavior / Notes |
|---|---:|---|---|---|---|
| Ball of Substantial Hurt | 250 | None | 4 | 12 | Wavy portal projectile with amplitude 5 and frequency 0.5. |
| Frost Burst | 170 | Slowed for 4s | 4 | 4.4 | Short-range slowing burst from Ice Portals. |
| Frost Explosion | 145, 160, 250 | Silenced for 3.8s on the 145-damage row | 2, 3, 6 | 16.2, 20, 21 | Armor-piercing ice blast family. The 250-damage row is marked Nox-only on the page. |
| Frost Magic | 150 | Pet Stasis for 5s | 2.5 | 7.5 | Boomerang shot that faces direction before returning. |
| Frost Missile | 105 | Silenced for 3.8s | 7.5 | 28.46 | Armor-piercing missile that decelerates after 0.1s to a minimum speed of 3. |
| Frost Twirl | 150, 185 | Pet Stasis for 5s on the 150-damage Nox row | 3, 3.5, 4, 5 | 10.62, ? | Includes standard turning twirls at 60 degrees/sec and a Nox-only armor-piercing, decelerating Pet Stasis variant. |
| Frost Vortex | 240 | None | 4 | 24 | Nox-only Ice Portal vortex. |
| Light Blue AoE | 160 | Silenced for 2s | Not listed | Not listed | AoE pulse with radius 1. |

## Raw Table Scope

The page's combat section includes three numeric attack tables:

- Boss attacks
- Fire Portal attacks
- Ice Portal attacks

Those tables are the basis for the grouped rows above. The phase descriptions in `phases-and-patterns.md` and `nox-hard-mode.md` explain where the projectile families actually appear in the fight and what bullet-hell formations they create.
