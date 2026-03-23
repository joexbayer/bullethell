import type { AtlasMeta } from "../types";

export interface AtlasBundle {
  texture: HTMLCanvasElement;
  meta: AtlasMeta;
}

export function createAtlas(): AtlasBundle {
  const cols = 4;
  const rows = 4;
  const cell = 64;
  const canvas = document.createElement("canvas");
  canvas.width = cols * cell;
  canvas.height = rows * cell;
  const ctx = canvas.getContext("2d");
  if (!ctx) {
    throw new Error("2D context unavailable");
  }

  ctx.clearRect(0, 0, canvas.width, canvas.height);
  drawSprite(ctx, cell, 0, 0, "#5f697d", "tile");
  drawSprite(ctx, cell, 1, 0, "#ff9f6a", "orb");
  drawSprite(ctx, cell, 2, 0, "#9eecff", "diamond");
  drawSprite(ctx, cell, 3, 0, "#ffd879", "square");
  drawSprite(ctx, cell, 0, 1, "#d0f4ff", "ring");
  drawSprite(ctx, cell, 1, 1, "#b18dff", "boss");
  drawSprite(ctx, cell, 2, 1, "#ffffff", "player");
  drawSprite(ctx, cell, 3, 1, "#d5e4ff", "square");
  drawSprite(ctx, cell, 0, 2, "#a878ff", "star");
  drawSprite(ctx, cell, 1, 2, "#b7c3d6", "edge");
  drawSprite(ctx, cell, 2, 2, "#ff7b57", "spike");
  drawSprite(ctx, cell, 3, 2, "#7ecfff", "shard");
  drawSprite(ctx, cell, 0, 3, "#ffd16a", "hex");
  drawSprite(ctx, cell, 1, 3, "#73ff9f", "ring");
  drawSprite(ctx, cell, 2, 3, "#ffffff", "ui_rect");
  drawSprite(ctx, cell, 3, 3, "#ffffff", "ui_rect");

  return {
    texture: canvas,
    meta: { cols, rows },
  };
}

function drawSprite(
  ctx: CanvasRenderingContext2D,
  cell: number,
  col: number,
  row: number,
  color: string,
  shape: "tile" | "square" | "boss" | "player" | "edge" | "orb" | "diamond" | "ring" | "star" | "spike" | "shard" | "hex" | "ui_rect",
) {
  const x = col * cell;
  const y = row * cell;
  const cx = x + cell / 2;
  const cy = y + cell / 2;
  ctx.fillStyle = color;
  if (shape === "tile" || shape === "edge") {
    ctx.fillRect(x, y, cell, cell);
    ctx.strokeStyle = shape === "edge" ? "rgba(255,255,255,0.55)" : "rgba(255,255,255,0.16)";
    ctx.lineWidth = 4;
    ctx.strokeRect(x + 2, y + 2, cell - 4, cell - 4);
    if (shape === "edge") {
      ctx.fillStyle = "rgba(255,255,255,0.14)";
      ctx.fillRect(x + 8, y + 8, cell - 16, cell - 16);
    }
    return;
  }
  if (shape === "ui_rect") {
    ctx.fillStyle = "#ffffff";
    ctx.fillRect(x + 6, y + 18, cell - 12, cell - 36);
    return;
  }
  ctx.save();
  ctx.translate(cx, cy);
  ctx.fillStyle = "#ffffff";
  ctx.strokeStyle = "rgba(15, 22, 34, 0.85)";
  ctx.lineWidth = 4;
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
    ctx.fillRect(-cell * 0.22, -cell * 0.22, cell * 0.44, cell * 0.44);
    ctx.strokeRect(-cell * 0.22, -cell * 0.22, cell * 0.44, cell * 0.44);
    ctx.clearRect(-cell * 0.08, -cell * 0.08, cell * 0.16, cell * 0.16);
  }
  ctx.restore();
}
