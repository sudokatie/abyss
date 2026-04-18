# Abyss

Deep sea survival where the ocean wants you dead.

## Why This Exists?

Survival games love fire, cold, and hunger. Nobody's done pressure right. The crushing weight of water is a fundamentally different threat - it doesn't care about your shelter, it ignores your walls, and it kills in ways that feel viscerally wrong. That's interesting.

## Features

- Oxygen management that gets harder the deeper you go
- Pressure zones that punish the unprepared (and the overconfident)
- 3D swimming through procedurally generated ocean trenches
- Bioluminescence as both beauty and survival tool
- Decompression sickness - ascend too fast, pay the price
- Underwater base building with airlocks (flood your base if you're careless)
- Beer-Lambert light attenuation - it gets dark, then darker, then wrong
- Creatures that get stranger with depth

## Quick Start

```bash
cargo build
cargo run
```

Requires a GPU with Vulkan/Metal/DX12 support.

## Controls

- WASD: Swim forward/back/strafe
- Space/Shift: Ascend/descend
- Mouse: Look around
- E: Interact
- Tab: Inventory

## Technical

Built on the Lattice engine. Rust, wgpu, hecs ECS, renet networking, kira audio, egui UI.

5 depth zones:
- Sunlight (0-200m): Coral, fish, warmth. You think you're safe.
- Twilight (200-500m): Dim, strange creatures, first ruins. You're not safe.
- Midnight (500-1000m): Complete darkness. Bioluminescence only. Pray.
- Abyssal (1000m+): Crushing pressure. Alien terrain. Why are you here?
- Hadal (deepest): The Trench Warden. You came this far. No going back.

## License

MIT

---

*The ocean doesn't negotiate.*
