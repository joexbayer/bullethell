# Twilight Archmage Fight Notes

Source page: [RealmEye - Twilight Archmage](https://www.realmeye.com/wiki/twilight-archmage)  
Source revision noted on the page: Exalt Version 4.2.4.0 (July 2024)

This folder pulls the boss-fight material out of the RealmEye page into separate Markdown references:

- `phases-and-patterns.md`: regular Twilight Archmage fight flow, phase thresholds, telegraphs, and bullet-hell pattern summaries.
- `projectiles.md`: projectile families plus the normalized attack/stat tables from the page.
- `nox-hard-mode.md`: the Nox the Wild Shadow variant, including the extra mechanics and finale requirements.

## Boss Snapshot

- HP: 300,000 with adaptive boss scaling
- DEF: 45
- EXP: 75,000
- Location: The Shatters
- Hitbox size: 2
- Quest enemy
- Immune to Stasis, Stunned, Paralyzed, and Dazed
- Counts toward God Kills and Humanoid Kills

## Arena And Activation

- The boss starts dormant in the middle of a large carpeted room.
- Four walkways surround the center platform; each quadrant holds a Magi-Generator.
- Players must carry Untempered Magic from three of the four castle wings to three generators.
- Once three generators are activated, the Archmage fully appears, seals the room, teleports players in, and begins the fight.
- Nearby Accursed graves and enemies are despawned when the fight starts.

## Core Combat Identity

- Every time the Archmage charges an attack, the three generators reroll as fire or ice.
- Elemental majority decides whether the current attack is a fire or ice pattern.
- If all active generators match, the boss uses a stronger nuke-style pattern instead.
- He stays stationary while attacking and is only vulnerable during an attack.
- A pattern can be ended early if players deal roughly 10% of his max HP during it.
- The page states that each attack family alternates between two internal variants, with the first pick being random.
- Fire phases are described as faster, denser, and more intense; ice phases are slower but longer-lasting.
- All portals summoned as part of his attacks are invincible and disappear on collision or when the pattern ends.

## Fight Entities Mentioned On The Page

- Magi-Generator
- Inferno
- Blizzard
- Tempest (hard mode only)
- Fire Bomb
- Ice Sphere
- Fire Portal
- Ice Portal
