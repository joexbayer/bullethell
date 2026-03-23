import type { InputSnapshot } from "../types";
import {
  MAX_VIEW_WORLD_SIZE,
  MIN_VIEW_WORLD_SIZE,
  VIEW_WORLD_SIZE,
  WORLD_ROTATION_SPEED_DEG,
  ZOOM_STEP,
} from "../config";

export class InputController {
  private readonly pressed = new Set<string>();
  private readonly justPressed = new Set<string>();
  private pointerX = 0;
  private pointerY = 0;
  private firing = false;
  private worldRotationDeg = 0;
  private viewWorldSize = VIEW_WORLD_SIZE;

  constructor(private readonly canvas: HTMLCanvasElement) {
    window.addEventListener("keydown", (event) => {
      if (!this.pressed.has(event.code)) {
        this.justPressed.add(event.code);
      }
      this.pressed.add(event.code);
    });
    window.addEventListener("keyup", (event) => {
      this.pressed.delete(event.code);
    });
    canvas.addEventListener("pointermove", (event) => {
      const rect = canvas.getBoundingClientRect();
      const scaleX = canvas.width / rect.width;
      const scaleY = canvas.height / rect.height;
      this.pointerX = (event.clientX - rect.left) * scaleX;
      this.pointerY = (event.clientY - rect.top) * scaleY;
    });
    canvas.addEventListener("pointerdown", () => {
      this.firing = true;
    });
    window.addEventListener("pointerup", () => {
      this.firing = false;
    });
    canvas.addEventListener(
      "wheel",
      (event) => {
        event.preventDefault();
        const direction = Math.sign(event.deltaY);
        if (direction === 0) return;
        this.viewWorldSize = clamp(
          this.viewWorldSize + direction * ZOOM_STEP,
          MIN_VIEW_WORLD_SIZE,
          MAX_VIEW_WORLD_SIZE,
        );
      },
      { passive: false },
    );
  }

  advance(deltaMs: number) {
    const rotateDir = Number(this.pressed.has("KeyE")) - Number(this.pressed.has("KeyQ"));
    this.worldRotationDeg += (rotateDir * WORLD_ROTATION_SPEED_DEG * deltaMs) / 1000;
    this.worldRotationDeg = ((this.worldRotationDeg % 360) + 360) % 360;
  }

  snapshot(cameraX: number, cameraY: number): InputSnapshot {
    const rotationRad = (this.worldRotationDeg * Math.PI) / 180;

    const screenMoveX = Number(this.pressed.has("KeyD")) - Number(this.pressed.has("KeyA"));
    const screenMoveY = Number(this.pressed.has("KeyS")) - Number(this.pressed.has("KeyW"));
    const rotatedMove = rotateVector(screenMoveX, screenMoveY, rotationRad);

    const screenAimX = (this.pointerX / this.canvas.width - 0.5) * this.viewWorldSize;
    const screenAimY = (this.pointerY / this.canvas.height - 0.5) * this.viewWorldSize;
    const rotatedAim = rotateVector(screenAimX, screenAimY, rotationRad);
    const snapshot: InputSnapshot = {
      move_x: rotatedMove.x,
      move_y: rotatedMove.y,
      aim_x: cameraX + rotatedAim.x,
      aim_y: cameraY + rotatedAim.y,
      fire_held: this.firing,
      ability_pressed: this.consume("Space"),
      pause_pressed: this.consume("KeyP"),
      slow_mo_pressed: this.consume("KeyO"),
      frame_step_pressed: this.consume("BracketRight"),
      debug_toggle_pressed: false,
      world_rotation_deg: this.worldRotationDeg,
    };
    return snapshot;
  }

  getWorldRotationDeg(): number {
    return this.worldRotationDeg;
  }

  getViewWorldSize(): number {
    return this.viewWorldSize;
  }

  private consume(code: string): boolean {
    const had = this.justPressed.has(code);
    this.justPressed.delete(code);
    return had;
  }
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

function rotateVector(x: number, y: number, rotationRad: number): { x: number; y: number } {
  const cos = Math.cos(rotationRad);
  const sin = Math.sin(rotationRad);
  return {
    x: x * cos - y * sin,
    y: x * sin + y * cos,
  };
}
