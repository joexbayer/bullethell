import init, {
  debug_command,
  export_replay,
  get_render_views,
  init_game,
  load_encounter,
  load_replay,
  start_replay,
  step,
} from "./generated/engine_wasm.js";
import wasmUrl from "./generated/engine_wasm_bg.wasm?url";
import type { FrameMeta, RenderViews } from "./types";
import { createAtlas } from "./renderer/atlas";
import { Renderer } from "./renderer/renderer";
import { InputController } from "./runtime/input";
import { BOSS_DISPLAY_NAME, CANVAS_RESOLUTION } from "./config";

const ENCOUNTER_ID = "twilight_archmage_v1";
type RunState = "ready" | "running" | "victory";

async function boot() {
  const canvas = document.querySelector<HTMLCanvasElement>("#game");
  const bossName = document.querySelector<HTMLSpanElement>("#boss-name");
  const bossState = document.querySelector<HTMLSpanElement>("#boss-state");
  const bossHpLabel = document.querySelector<HTMLSpanElement>("#boss-hp-label");
  const bossBarFill = document.querySelector<HTMLDivElement>("#boss-bar-fill");
  const objectiveLabel = document.querySelector<HTMLSpanElement>("#objective-label");
  const fpsPill = document.querySelector<HTMLDivElement>("#fps-pill");
  const timerPill = document.querySelector<HTMLDivElement>("#timer-pill");
  const promptFlash = document.querySelector<HTMLDivElement>("#prompt-flash");
  const damageFlash = document.querySelector<HTMLDivElement>("#damage-flash");
  const playerHpLabel = document.querySelector<HTMLSpanElement>("#player-hp-label");
  const playerBarFill = document.querySelector<HTMLDivElement>("#player-bar-fill");
  const playerMpLabel = document.querySelector<HTMLSpanElement>("#player-mp-label");
  const playerManaFill = document.querySelector<HTMLDivElement>("#player-mana-fill");
  const menuOverlay = document.querySelector<HTMLDivElement>("#menu-overlay");
  const menuTitle = document.querySelector<HTMLDivElement>("#menu-title");
  const menuCopy = document.querySelector<HTMLParagraphElement>("#menu-copy");
  const startButton = document.querySelector<HTMLButtonElement>("#start-button");
  if (
    !canvas ||
    !bossName ||
    !bossState ||
    !bossHpLabel ||
    !bossBarFill ||
    !objectiveLabel ||
    !fpsPill ||
    !timerPill ||
    !promptFlash ||
    !damageFlash ||
    !playerHpLabel ||
    !playerBarFill ||
    !playerMpLabel ||
    !playerManaFill ||
    !menuOverlay ||
    !menuTitle ||
    !menuCopy ||
    !startButton
  ) {
    throw new Error("missing DOM nodes");
  }
  canvas.width = CANVAS_RESOLUTION;
  canvas.height = CANVAS_RESOLUTION;
  bossName.textContent = BOSS_DISPLAY_NAME;

  const wasm = await init(wasmUrl);
  const [contentResponse] = await Promise.all([fetch(import.meta.env.BASE_URL + "content.bin")]);
  const contentBlob = new Uint8Array(await contentResponse.arrayBuffer());
  const atlas = createAtlas();
  const renderer = new Renderer(canvas, atlas, wasm.memory as WebAssembly.Memory);
  const input = new InputController(canvas);

  init_game(contentBlob, atlas.meta, {
    width: canvas.width,
    height: canvas.height,
    debug_enabled: false,
  });
  load_encounter(ENCOUNTER_ID);

  let previous = performance.now();
  let accumulator = 0;
  const fixedStepMs = 1000 / 60;
  let latestMeta: FrameMeta | null = null;
  let previousMeta: FrameMeta | null = null;
  let lastPrompt = "";
  let lastPromptAt = -30_000;
  let lastDamageFlashAt = -10_000;
  let lastHudUpdate = -1_000;
  let hudCache = "";
  let runState: RunState = "ready";
  let runStartAt = 0;
  let runElapsedMs = 0;

  const syncMenu = () => {
    timerPill.textContent = formatRunTime(runElapsedMs);
    menuOverlay.hidden = runState === "running";
    if (runState === "ready") {
      objectiveLabel.textContent = "Press Start";
      objectiveLabel.dataset.tone = "seal";
      bossState.textContent = "Waiting";
      menuTitle.textContent = runElapsedMs > 0 ? "Run Failed" : "Start Run";
      menuCopy.textContent =
        runElapsedMs > 0
          ? `You died. Last run: ${formatRunTime(runElapsedMs)}. Press Start to try again.`
          : "Begin the fight. The timer starts on button press and stops when the boss dies.";
      startButton.textContent = "Start";
    } else if (runState === "victory") {
      objectiveLabel.textContent = "Boss Down";
      objectiveLabel.dataset.tone = "victory";
      bossState.textContent = "Defeated";
      menuTitle.textContent = "Boss Down";
      menuCopy.textContent = `Clear time: ${formatRunTime(runElapsedMs)}.`;
      startButton.textContent = "Start Again";
      menuOverlay.hidden = true;
    }
  };

  const resetHudCache = () => {
    hudCache = "";
    lastHudUpdate = -1_000;
  };

  const beginRun = (now: number) => {
    load_encounter(ENCOUNTER_ID);
    latestMeta = null;
    previousMeta = null;
    accumulator = 0;
    runState = "running";
    runStartAt = now;
    runElapsedMs = 0;
    lastPrompt = "";
    lastPromptAt = -30_000;
    resetHudCache();
    syncMenu();
  };

  const failRun = () => {
    runState = "ready";
    load_encounter(ENCOUNTER_ID);
    latestMeta = null;
    previousMeta = null;
    accumulator = 0;
    lastPrompt = "You Died";
    lastPromptAt = performance.now();
    promptFlash.textContent = "You Died";
    replayAnimation(promptFlash);
    resetHudCache();
    syncMenu();
  };

  startButton.addEventListener("click", () => beginRun(performance.now()));
  syncMenu();

  const updateHud = (meta: FrameMeta, now: number) => {
    timerPill.textContent = formatRunTime(runElapsedMs);
    if (runState !== "running") {
      objectiveLabel.textContent = runState === "victory" ? "Boss Down" : "Press Start";
      objectiveLabel.dataset.tone = "seal";
      return;
    }
    const objective = getObjective(meta);
    if (now - lastHudUpdate >= 66) {
      lastHudUpdate = now;
      const nextCache = [
        Math.round(meta.fps_estimate),
        Math.ceil(meta.boss_hp),
        Math.ceil(meta.boss_max_hp),
        meta.boss_invulnerable ? 1 : 0,
        meta.boss_armored ? 1 : 0,
        objective.label,
        objective.tone,
        Math.ceil(meta.player_hp),
        Math.ceil(meta.player_max_hp),
        Math.ceil(meta.player_mp),
        Math.ceil(meta.player_max_mp),
      ].join("|");
      if (nextCache !== hudCache) {
        hudCache = nextCache;
        bossHpLabel.textContent = `${Math.ceil(meta.boss_hp)} / ${Math.ceil(meta.boss_max_hp)}`;
        bossBarFill.style.transform = `scaleX(${safeRatio(meta.boss_hp, meta.boss_max_hp)})`;
        const bossStateLabel =
          meta.support_delay_frames > 0
            ? "Recovering"
            : meta.boss_invulnerable
              ? "Invulnerable"
              : meta.boss_armored
                ? "Armored"
                : "Vulnerable";
        bossState.textContent = bossStateLabel;
        bossState.classList.toggle("invulnerable", meta.support_delay_frames > 0 || meta.boss_invulnerable);
        bossState.classList.toggle("armored", meta.support_delay_frames === 0 && !meta.boss_invulnerable && meta.boss_armored);
        objectiveLabel.textContent = objective.label;
        objectiveLabel.dataset.tone = objective.tone;
        fpsPill.textContent = `FPS ${Math.round(meta.fps_estimate)}`;
        playerHpLabel.textContent = `${Math.ceil(meta.player_hp)} / ${Math.ceil(meta.player_max_hp)}`;
        playerBarFill.style.transform = `scaleX(${safeRatio(meta.player_hp, meta.player_max_hp)})`;
        playerMpLabel.textContent = `${Math.ceil(meta.player_mp)} / ${Math.ceil(meta.player_max_mp)}`;
        playerManaFill.style.transform = `scaleX(${safeRatio(meta.player_mp, meta.player_max_mp)})`;
        timerPill.textContent = formatRunTime(runElapsedMs);
      }
    }
    const flashText = getFlashPrompt(meta);
    if (flashText && shouldFlashPrompt(flashText, now, lastPrompt, lastPromptAt)) {
      lastPrompt = flashText;
      lastPromptAt = now;
      replayAnimation(promptFlash);
      promptFlash.textContent = flashText;
    }
  };

  const frame = (now: number) => {
    const delta = Math.min(32, now - previous);
    previous = now;
    input.advance(delta);
    if (runState === "running") {
      runElapsedMs = now - runStartAt;
      accumulator += delta;
      while (accumulator >= fixedStepMs) {
        previousMeta = latestMeta;
        const snapshot = input.snapshot(latestMeta?.player_x ?? 5.5, latestMeta?.player_y ?? 10.5);
        latestMeta = step(snapshot) as FrameMeta;
        if (previousMeta && latestMeta.player_hp + 0.01 < previousMeta.player_hp && now - lastDamageFlashAt > 90) {
          lastDamageFlashAt = now;
          replayAnimation(damageFlash);
        }
        if (previousMeta && previousMeta.player_hp > 0.0 && latestMeta.player_hp <= 0.0) {
          failRun();
          break;
        }
        if (previousMeta && previousMeta.boss_hp > 0.0 && latestMeta.boss_hp <= 0.0) {
          runState = "victory";
          runElapsedMs = now - runStartAt;
          syncMenu();
          break;
        }
        accumulator -= fixedStepMs;
      }
    }
    if (latestMeta) {
      updateHud(latestMeta, now);
    }
    const views = get_render_views() as RenderViews;
    const alpha = accumulator / fixedStepMs;
    let cameraX = lerp(previousMeta?.player_x ?? latestMeta?.player_x ?? 5.5, latestMeta?.player_x ?? 5.5, alpha);
    let cameraY = lerp(previousMeta?.player_y ?? latestMeta?.player_y ?? 10.5, latestMeta?.player_y ?? 10.5, alpha);
    if (latestMeta && latestMeta.shake_amplitude > 0) {
      cameraX += (Math.random() * 2 - 1) * latestMeta.shake_amplitude;
      cameraY += (Math.random() * 2 - 1) * latestMeta.shake_amplitude;
    }
    renderer.render(
      views,
      latestMeta,
      cameraX,
      cameraY,
      input.getViewWorldSize(),
      input.getWorldRotationDeg(),
    );
    requestAnimationFrame(frame);
  };

  window.addEventListener("keydown", (event) => {
    if (event.key.toLowerCase() === "p") {
      debug_command({ type: "Pause", payload: true });
    }
    if (event.key.toLowerCase() === "o") {
      debug_command({ type: "SlowMo", payload: true });
    }
    if (event.key === "]") {
      debug_command({ type: "Step" });
    }
  });

  Object.assign(window, {
    startReplay: (seed = 1234) => start_replay(BigInt(seed)),
    exportReplay: () => export_replay(),
    loadReplay: (blob: unknown) => load_replay(blob),
  });

  requestAnimationFrame(frame);
}

function safeRatio(value: number, max: number): number {
  if (max <= 0) return 0;
  return Math.min(Math.max(value / max, 0), 1);
}

function lerp(a: number, b: number, t: number): number {
  return a + (b - a) * Math.min(Math.max(t, 0), 1);
}

function getObjective(meta: FrameMeta): { label: string; tone: string } {
  if (meta.boss_hp <= 0) {
    return { label: "Boss Down", tone: "victory" };
  }
  if (meta.phase.startsWith("seal_")) {
    return { label: "Kill Seal", tone: "seal" };
  }
  if (meta.phase === "duel") {
    return {
      label: meta.active_helpers > 1 ? "Kill Birds" : "Kill Bird",
      tone: "bird",
    };
  }
  if (meta.damage_window_frames > 0 || meta.stagger_frames > 0) {
    return { label: "Attack Boss", tone: "attack" };
  }
  if (meta.support_delay_frames > 0) {
    return { label: "Clear Bullets", tone: "seal" };
  }
  if (meta.phase === "single_bird") {
    return { label: "Kill Bird", tone: "bird" };
  }
  if (meta.phase === "dual_guard") {
    return { label: "Kill Birds", tone: "bird" };
  }
  if (meta.active_objects > 0) {
    return { label: "Kill Portals", tone: "portal" };
  }
  if (meta.active_helpers > 0) {
    return { label: meta.active_helpers > 1 ? "Kill Birds" : "Kill Bird", tone: "bird" };
  }
  return { label: "Attack Boss", tone: "attack" };
}

function getFlashPrompt(meta: FrameMeta): string {
  if (meta.message && shouldFlashMessage(meta.message)) {
    return meta.message;
  }
  const objective = getObjective(meta).label;
  if (objective === "Boss Down") {
    return objective;
  }
  if (
    objective === "Attack Boss" &&
    meta.stagger_frames === 0 &&
    meta.damage_window_frames === 0
  ) {
    return "";
  }
  return objective;
}

function shouldFlashMessage(message: string): boolean {
  return (
    message.startsWith("Phase:") ||
    message.startsWith("Generator sealed:") ||
    message === "Twilight Archmage awakens" ||
    message === "Seal a Magi-Generator" ||
    message === "Seal another Magi-Generator" ||
    message === "Final generator exposed" ||
    message === "Generator-lock duel" ||
    message === "Finale" ||
    message === "Archmage staggered" ||
    message === "Bird down: clear bullets"
  );
}

function shouldFlashPrompt(next: string, now: number, last: string, lastAt: number): boolean {
  return next !== last && now - lastAt >= 600;
}

function replayAnimation(element: HTMLElement): void {
  element.classList.remove("visible");
  void element.offsetWidth;
  element.classList.add("visible");
}

boot().catch((error) => {
  console.error(error);
  const objectiveLabel = document.querySelector<HTMLSpanElement>("#objective-label");
  if (objectiveLabel) {
    objectiveLabel.textContent = `Boot failed: ${String(error)}`;
    objectiveLabel.dataset.tone = "seal";
  }
});

function formatRunTime(totalMs: number): string {
  const bounded = Math.max(0, totalMs);
  const minutes = Math.floor(bounded / 60000);
  const seconds = Math.floor((bounded % 60000) / 1000);
  const centiseconds = Math.floor((bounded % 1000) / 10);
  return `${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}.${String(centiseconds).padStart(2, "0")}`;
}
