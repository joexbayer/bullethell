import type { AtlasMeta } from "../types";

export interface AtlasBundle {
  texture: HTMLCanvasElement;
  meta: AtlasMeta;
}

export function createAtlas(): AtlasBundle {
  const cols = 4;
  const rows = 4;
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

  return {
    texture: canvas,
    meta: { cols, rows },
  };
}

type SpriteShape = "tile" | "square" | "boss" | "player" | "edge" | "orb" | "diamond" | "ring" | "star" | "spike" | "shard" | "hex" | "ui_rect" | "soft_circle";

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

  if (shape === "square") {
    ctx.fillRect(-cell * 0.22, -cell * 0.22, cell * 0.44, cell * 0.44);
    ctx.strokeRect(-cell * 0.22, -cell * 0.22, cell * 0.44, cell * 0.44);
  } else if (shape === "orb") {
    ctx.beginPath();
    ctx.arc(0, 0, cell * 0.20, 0, Math.PI * 2);
    ctx.fill();
    ctx.stroke();
  } else if (shape === "diamond") {
    ctx.rotate(Math.PI / 4);
    ctx.fillRect(-cell * 0.18, -cell * 0.18, cell * 0.36, cell * 0.36);
    ctx.strokeRect(-cell * 0.18, -cell * 0.18, cell * 0.36, cell * 0.36);
  } else if (shape === "ring") {
    ctx.beginPath();
    ctx.arc(0, 0, cell * 0.21, 0, Math.PI * 2);
    ctx.fill();
    ctx.globalCompositeOperation = "destination-out";
    ctx.beginPath();
    ctx.arc(0, 0, cell * 0.11, 0, Math.PI * 2);
    ctx.fill();
    ctx.globalCompositeOperation = "source-over";
    ctx.beginPath();
    ctx.arc(0, 0, cell * 0.21, 0, Math.PI * 2);
    ctx.stroke();
  } else if (shape === "spike") {
    ctx.beginPath();
    ctx.moveTo(cell * 0.26, 0);
    ctx.lineTo(-cell * 0.16, -cell * 0.16);
    ctx.lineTo(-cell * 0.16, cell * 0.16);
    ctx.closePath();
    ctx.fill();
    ctx.stroke();
  } else if (shape === "shard") {
    ctx.beginPath();
    ctx.moveTo(cell * 0.22, 0);
    ctx.lineTo(0, -cell * 0.18);
    ctx.lineTo(-cell * 0.22, 0);
    ctx.lineTo(0, cell * 0.18);
    ctx.closePath();
    ctx.fill();
    ctx.stroke();
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
    ctx.stroke();
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
    ctx.stroke();
  } else if (shape === "player") {
    ctx.fillRect(-cell * 0.19, -cell * 0.19, cell * 0.38, cell * 0.38);
    ctx.strokeRect(-cell * 0.19, -cell * 0.19, cell * 0.38, cell * 0.38);
    ctx.clearRect(-cell * 0.06, -cell * 0.06, cell * 0.12, cell * 0.12);
  } else {
    // boss
    ctx.fillRect(-cell * 0.22, -cell * 0.22, cell * 0.44, cell * 0.44);
    ctx.strokeRect(-cell * 0.22, -cell * 0.22, cell * 0.44, cell * 0.44);
    ctx.clearRect(-cell * 0.08, -cell * 0.08, cell * 0.16, cell * 0.16);
  }
  ctx.restore();
}
