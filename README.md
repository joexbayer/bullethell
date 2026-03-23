# Realm Bullet Hell

Browser bullet-hell prototype with:
- Rust/WASM simulation
- WebGL2 renderer
- data-driven encounter content in `RON`

## Prerequisites

- Node.js 20+
- npm
- Rust toolchain via `rustup`
- `wasm32-unknown-unknown` target
- `wasm-bindgen-cli`

Install the Rust pieces if needed:

```bash
rustup toolchain install stable
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
```

Install JS dependencies:

```bash
npm install
```

## Run Dev Server

From the project root:

```bash
npm run dev
```

That will:
- pack `assets/content/game.ron` into `public/content.bin`
- build the Rust WASM module
- generate bindings into `src/generated`
- start Vite

Open the local URL printed by Vite, usually:

```text
http://127.0.0.1:5173/
```

## Production Build

```bash
npm run build
```

Preview the production bundle:

```bash
npm run preview
```

## Project Layout

- `assets/content/game.ron` encounter and pattern data
- `crates/engine-wasm` simulation/runtime
- `crates/schema` shared content schema
- `tools/content-packer` RON -> binary content compiler
- `src/main.ts` browser host and HUD
- `src/renderer` WebGL renderer and atlas generation

## Notes

- Generated output is ignored by git:
  - `dist/`
  - `target/`
  - `src/generated/`
  - `public/content.bin`
- When the player dies, the encounter now force-restarts automatically after a short delay.
