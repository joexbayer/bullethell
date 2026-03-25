import type { AtlasMeta } from "../types";
import bossSpriteUrl from "../../assets/archmage1.png?url";
import deadBossSpriteUrl from "../../assets/archmage2.png?url";
import infernoBirdSpriteUrl from "../../assets/birdfire.png?url";
import blizzardBirdSpriteUrl from "../../assets/birdfrost.png?url";
import firePortalSpriteUrl from "../../assets/fireportal.png?url";
import icePortalSpriteUrl from "../../assets/portalice.png?url";

export interface AtlasBundle {
  texture: HTMLCanvasElement;
  meta: AtlasMeta;
}

export async function createAtlas(): Promise<AtlasBundle> {
  const cols = 4;
  const rows = 6;
  const cell = 128;
  const canvas = document.createElement("canvas");
  canvas.width = cols * cell;
  canvas.height = rows * cell;
  const ctx = canvas.getContext("2d");
  if (!ctx) {
    throw new Error("2D context unavailable");
  }

  ctx.clearRect(0, 0, canvas.width, canvas.height);
  drawSprite(ctx, cell, 0, 0, "#5f697d", "tile");       // 0: tile
  drawSprite(ctx, cell, 1, 0, "#ff9f6a", "orb");        // 1: orb
  drawSprite(ctx, cell, 2, 0, "#9eecff", "diamond");    // 2: diamond
  drawSprite(ctx, cell, 3, 0, "#ffd879", "square");     // 3: generator core
  drawSprite(ctx, cell, 0, 1, "#d0f4ff", "ring");       // 4: ring
  drawSprite(ctx, cell, 1, 1, "#b18dff", "boss");       // 5: boss
  drawSprite(ctx, cell, 2, 1, "#ffffff", "player");      // 6: player
  drawSprite(ctx, cell, 3, 1, "#d5e4ff", "square");     // 7: player shot
  drawSprite(ctx, cell, 0, 2, "#a878ff", "star");        // 8: star
  drawSprite(ctx, cell, 1, 2, "#b7c3d6", "edge");       // 9: edge wall
  drawSprite(ctx, cell, 2, 2, "#ff7b57", "spike");      // 10: spike
  drawSprite(ctx, cell, 3, 2, "#7ecfff", "shard");      // 11: shard
  drawSprite(ctx, cell, 0, 3, "#ffd16a", "hex");        // 12: hex
  drawSprite(ctx, cell, 1, 3, "#73ff9f", "ring");       // 13: generator ring
  drawSprite(ctx, cell, 2, 3, "#ffffff", "ui_rect");    // 14: ui_rect
  drawSprite(ctx, cell, 3, 3, "#ffffff", "soft_circle"); // 15: soft circle (particles)
  drawSprite(ctx, cell, 0, 4, "#ff8a57", "inferno_bird"); // 16: inferno bird
  drawSprite(ctx, cell, 1, 4, "#8fdcff", "blizzard_bird"); // 17: blizzard bird
  drawSprite(ctx, cell, 2, 4, "#ffb76b", "portal");     // 18: fire portal
  drawSprite(ctx, cell, 3, 4, "#a9e9ff", "portal");     // 19: ice portal
  drawSprite(ctx, cell, 0, 5, "#8f6ca8", "boss");       // 20: dead boss
  drawSprite(ctx, cell, 1, 5, "#ffffff", "ring");       // 21: reserved
  drawSprite(ctx, cell, 2, 5, "#ffffff", "ring");       // 22: reserved
  drawSprite(ctx, cell, 3, 5, "#ffffff", "ring");       // 23: reserved

  const spriteOverrides = await Promise.all([
    loadOptionalSprite(bossSpriteUrl),
    loadOptionalSprite(deadBossSpriteUrl),
    loadOptionalSprite(infernoBirdSpriteUrl),
    loadOptionalSprite(blizzardBirdSpriteUrl),
    loadOptionalSprite(firePortalSpriteUrl),
    loadOptionalSprite(icePortalSpriteUrl),
  ]);
  const [
    bossSprite,
    deadBossSprite,
    infernoBirdSprite,
    blizzardBirdSprite,
    firePortalSprite,
    icePortalSprite,
  ] = spriteOverrides;
  if (bossSprite) {
    drawImageSprite(ctx, bossSprite, cell, 1, 1);
  }
  if (deadBossSprite) {
    drawImageSprite(ctx, deadBossSprite, cell, 0, 5);
  }
  if (infernoBirdSprite) {
    drawImageSprite(ctx, infernoBirdSprite, cell, 0, 4);
  }
  if (blizzardBirdSprite) {
    drawImageSprite(ctx, blizzardBirdSprite, cell, 1, 4);
  }
  if (firePortalSprite) {
    drawImageSprite(ctx, firePortalSprite, cell, 2, 4);
  }
  if (icePortalSprite) {
    drawImageSprite(ctx, icePortalSprite, cell, 3, 4);
  }

  return {
    texture: canvas,
    meta: { cols, rows },
  };
}

async function loadOptionalSprite(src: string): Promise<HTMLImageElement | null> {
  return new Promise((resolve) => {
    const image = new Image();
    image.decoding = "async";
    image.onload = () => resolve(image);
    image.onerror = () => resolve(null);
    image.src = src;
  });
}

function drawImageSprite(
  ctx: CanvasRenderingContext2D,
  image: CanvasImageSource,
  cell: number,
  col: number,
  row: number,
) {
  const x = col * cell;
  const y = row * cell;
  const width = image instanceof HTMLImageElement ? image.naturalWidth : cell;
  const height = image instanceof HTMLImageElement ? image.naturalHeight : cell;
  const inner = cell - 16;
  const scale = Math.min(inner / width, inner / height);
  const drawWidth = width * scale;
  const drawHeight = height * scale;
  const dx = x + (cell - drawWidth) * 0.5;
  const dy = y + (cell - drawHeight) * 0.5;

  ctx.clearRect(x, y, cell, cell);
  ctx.save();
  ctx.imageSmoothingEnabled = false;
  ctx.drawImage(image, dx, dy, drawWidth, drawHeight);
  ctx.restore();
}

type SpriteShape =
  | "tile"
  | "square"
  | "boss"
  | "player"
  | "edge"
  | "orb"
  | "diamond"
  | "ring"
  | "star"
  | "spike"
  | "shard"
  | "hex"
  | "ui_rect"
  | "soft_circle"
  | "inferno_bird"
  | "blizzard_bird"
  | "portal";

const PROJECTILE_SHAPES: SpriteShape[] = ["orb", "diamond", "spike", "shard", "hex", "star"];

function drawSprite(
  ctx: CanvasRenderingContext2D,
  cell: number,
  col: number,
  row: number,
  color: string,
  shape: SpriteShape,
) {
  const x = col * cell;
  const y = row * cell;
  const cx = x + cell / 2;
  const cy = y + cell / 2;
  ctx.fillStyle = color;

  if (shape === "tile" || shape === "edge") {
    ctx.fillRect(x, y, cell, cell);
    ctx.strokeStyle = shape === "edge" ? "rgba(255,255,255,0.55)" : "rgba(255,255,255,0.16)";
    ctx.lineWidth = 6;
    ctx.strokeRect(x + 3, y + 3, cell - 6, cell - 6);
    if (shape === "edge") {
      ctx.fillStyle = "rgba(255,255,255,0.14)";
      ctx.fillRect(x + 12, y + 12, cell - 24, cell - 24);
    }
    return;
  }

  if (shape === "ui_rect") {
    ctx.fillStyle = "#ffffff";
    ctx.fillRect(x + 8, y + 28, cell - 16, cell - 56);
    return;
  }

  if (shape === "soft_circle") {
    const gradient = ctx.createRadialGradient(cx, cy, 0, cx, cy, cell * 0.42);
    gradient.addColorStop(0, "rgba(255,255,255,1.0)");
    gradient.addColorStop(0.3, "rgba(255,255,255,0.6)");
    gradient.addColorStop(0.7, "rgba(255,255,255,0.15)");
    gradient.addColorStop(1, "rgba(255,255,255,0.0)");
    ctx.fillStyle = gradient;
    ctx.fillRect(x, y, cell, cell);
    return;
  }

  ctx.save();
  ctx.translate(cx, cy);

  // Draw glow halo for projectile shapes
  if (PROJECTILE_SHAPES.includes(shape)) {
    const glowGradient = ctx.createRadialGradient(0, 0, 0, 0, 0, cell * 0.38);
    glowGradient.addColorStop(0, "rgba(255,255,255,0.45)");
    glowGradient.addColorStop(0.4, "rgba(255,255,255,0.12)");
    glowGradient.addColorStop(1, "rgba(255,255,255,0.0)");
    ctx.fillStyle = glowGradient;
    ctx.beginPath();
    ctx.arc(0, 0, cell * 0.38, 0, Math.PI * 2);
    ctx.fill();
  }

  ctx.fillStyle = "#ffffff";
  ctx.strokeStyle = "rgba(15, 22, 34, 0.85)";
  ctx.lineWidth = 5;

  if (shape === "portal") {
    const outerGradient = ctx.createRadialGradient(0, 0, cell * 0.05, 0, 0, cell * 0.34);
    outerGradient.addColorStop(0, "rgba(255,255,255,0.95)");
    outerGradient.addColorStop(0.45, "rgba(255,255,255,0.25)");
    outerGradient.addColorStop(1, "rgba(255,255,255,0.0)");
    ctx.fillStyle = outerGradient;
    ctx.beginPath();
    ctx.arc(0, 0, cell * 0.34, 0, Math.PI * 2);
    ctx.fill();

    ctx.strokeStyle = "#ffffff";
    ctx.lineWidth = 7;
    ctx.beginPath();
    ctx.arc(0, 0, cell * 0.21, 0, Math.PI * 2);
    ctx.stroke();

    ctx.lineWidth = 5;
    ctx.beginPath();
    ctx.moveTo(-cell * 0.05, -cell * 0.30);
    ctx.bezierCurveTo(cell * 0.14, -cell * 0.18, cell * 0.14, cell * 0.18, -cell * 0.05, cell * 0.30);
    ctx.stroke();

    ctx.beginPath();
    ctx.moveTo(cell * 0.05, -cell * 0.30);
    ctx.bezierCurveTo(-cell * 0.14, -cell * 0.18, -cell * 0.14, cell * 0.18, cell * 0.05, cell * 0.30);
    ctx.stroke();
    ctx.restore();
    return;
  }

  if (shape === "inferno_bird" || shape === "blizzard_bird") {
    const wingY = shape === "inferno_bird" ? -cell * 0.03 : cell * 0.01;
    ctx.beginPath();
    ctx.moveTo(-cell * 0.30, wingY);
    ctx.quadraticCurveTo(-cell * 0.10, -cell * 0.30, 0, -cell * 0.08);
    ctx.quadraticCurveTo(cell * 0.10, -cell * 0.30, cell * 0.30, wingY);
    ctx.quadraticCurveTo(cell * 0.08, cell * 0.04, 0, cell * 0.02);
    ctx.quadraticCurveTo(-cell * 0.08, cell * 0.04, -cell * 0.30, wingY);
    ctx.closePath();
    ctx.fill();
    ctx.stroke();

    ctx.beginPath();
    ctx.moveTo(-cell * 0.12, cell * 0.02);
    ctx.lineTo(0, cell * 0.26);
    ctx.lineTo(cell * 0.12, cell * 0.02);
    ctx.closePath();
    ctx.fill();

    ctx.beginPath();
    ctx.arc(0, -cell * 0.03, cell * 0.08, 0, Math.PI * 2);
    ctx.fill();

    ctx.fillStyle = "rgba(15, 22, 34, 0.85)";
    ctx.beginPath();
    ctx.arc(0, -cell * 0.05, cell * 0.025, 0, Math.PI * 2);
    ctx.fill();

    ctx.fillStyle = "#ffffff";
    ctx.beginPath();
    if (shape === "inferno_bird") {
      ctx.moveTo(cell * 0.02, cell * 0.00);
      ctx.lineTo(cell * 0.16, cell * 0.04);
      ctx.lineTo(cell * 0.03, cell * 0.10);
    } else {
      ctx.moveTo(cell * 0.00, cell * 0.14);
      ctx.lineTo(-cell * 0.06, cell * 0.24);
      ctx.lineTo(cell * 0.06, cell * 0.24);
    }
    ctx.closePath();
    ctx.fill();
    ctx.restore();
    return;
  }

  if (shape === "square") {
    ctx.fillRect(-cell * 0.22, -cell * 0.22, cell * 0.44, cell * 0.44);
    ctx.strokeRect(-cell * 0.22, -cell * 0.22, cell * 0.44, cell * 0.44);
  } else if (shape === "orb") {
    ctx.beginPath();
    ctx.arc(0, 0, cell * 0.20, 0, Math.PI * 2);
    ctx.fill();
  } else if (shape === "diamond") {
    ctx.rotate(Math.PI / 4);
    ctx.fillRect(-cell * 0.18, -cell * 0.18, cell * 0.36, cell * 0.36);
  } else if (shape === "ring") {
    ctx.beginPath();
    ctx.arc(0, 0, cell * 0.21, 0, Math.PI * 2);
    ctx.fill();
    ctx.globalCompositeOperation = "destination-out";
    ctx.beginPath();
    ctx.arc(0, 0, cell * 0.11, 0, Math.PI * 2);
    ctx.fill();
    ctx.globalCompositeOperation = "source-over";
  } else if (shape === "spike") {
    ctx.beginPath();
    ctx.moveTo(cell * 0.26, 0);
    ctx.lineTo(-cell * 0.16, -cell * 0.16);
    ctx.lineTo(-cell * 0.16, cell * 0.16);
    ctx.closePath();
    ctx.fill();
  } else if (shape === "shard") {
    ctx.beginPath();
    ctx.moveTo(cell * 0.22, 0);
    ctx.lineTo(0, -cell * 0.18);
    ctx.lineTo(-cell * 0.22, 0);
    ctx.lineTo(0, cell * 0.18);
    ctx.closePath();
    ctx.fill();
  } else if (shape === "hex") {
    ctx.beginPath();
    for (let i = 0; i < 6; i += 1) {
      const angle = (Math.PI * 2 * i) / 6;
      const px = Math.cos(angle) * cell * 0.21;
      const py = Math.sin(angle) * cell * 0.21;
      if (i === 0) ctx.moveTo(px, py);
      else ctx.lineTo(px, py);
    }
    ctx.closePath();
    ctx.fill();
  } else if (shape === "star") {
    ctx.beginPath();
    for (let i = 0; i < 10; i += 1) {
      const angle = (-Math.PI / 2) + (Math.PI * i) / 5;
      const radius = i % 2 === 0 ? cell * 0.22 : cell * 0.10;
      const px = Math.cos(angle) * radius;
      const py = Math.sin(angle) * radius;
      if (i === 0) ctx.moveTo(px, py);
      else ctx.lineTo(px, py);
    }
    ctx.closePath();
    ctx.fill();
  } else if (shape === "player") {
    ctx.fillRect(-cell * 0.19, -cell * 0.19, cell * 0.38, cell * 0.38);
    ctx.strokeRect(-cell * 0.19, -cell * 0.19, cell * 0.38, cell * 0.38);
    ctx.clearRect(-cell * 0.06, -cell * 0.06, cell * 0.12, cell * 0.12);
  } else {
    // boss
    ctx.beginPath();
    ctx.moveTo(0, -cell * 0.30);
    ctx.lineTo(cell * 0.20, -cell * 0.16);
    ctx.lineTo(cell * 0.28, cell * 0.04);
    ctx.lineTo(cell * 0.16, cell * 0.26);
    ctx.lineTo(0, cell * 0.34);
    ctx.lineTo(-cell * 0.16, cell * 0.26);
    ctx.lineTo(-cell * 0.28, cell * 0.04);
    ctx.lineTo(-cell * 0.20, -cell * 0.16);
    ctx.closePath();
    ctx.fill();
    ctx.stroke();

    ctx.clearRect(-cell * 0.06, -cell * 0.10, cell * 0.12, cell * 0.12);
    ctx.fillStyle = "#ffffff";
    ctx.fillRect(-cell * 0.18, -cell * 0.02, cell * 0.10, cell * 0.05);
    ctx.fillRect(cell * 0.08, -cell * 0.02, cell * 0.10, cell * 0.05);
  }
  ctx.restore();
}
