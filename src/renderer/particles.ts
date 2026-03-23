const MAX_PARTICLES = 2000;
const FLOATS_PER_INSTANCE = 16;
const EVENT_FLOATS = 7;

// Event types (must match Rust constants)
const EVENT_BULLET_HIT_PLAYER = 1.0;
const EVENT_SHOT_HIT_ENEMY = 2.0;
const EVENT_HELPER_DEATH = 3.0;
const EVENT_OBJECT_DEATH = 4.0;
const EVENT_GENERATOR_SEALED = 5.0;
const EVENT_BOSS_DEATH = 6.0;

// Sprite IDs (must match Rust constants)
const SPRITE_SOFT_CIRCLE = 15;

interface Particle {
  x: number;
  y: number;
  vx: number;
  vy: number;
  r: number;
  g: number;
  b: number;
  alpha: number;
  size: number;
  sizeDecay: number;
  life: number;
  maxLife: number;
  glow: number;
}

export class ParticleSystem {
  private particles: Particle[] = [];

  processEvents(eventBuffer: Float32Array): void {
    const count = Math.floor(eventBuffer.length / EVENT_FLOATS);
    for (let i = 0; i < count; i++) {
      const offset = i * EVENT_FLOATS;
      const type = eventBuffer[offset];
      const x = eventBuffer[offset + 1];
      const y = eventBuffer[offset + 2];
      const r = eventBuffer[offset + 3];
      const g = eventBuffer[offset + 4];
      const b = eventBuffer[offset + 5];

      if (type === EVENT_BULLET_HIT_PLAYER) {
        this.spawnBurst(x, y, r, g, b, 8, 4.0, 12, 0.08, 0.6);
      } else if (type === EVENT_SHOT_HIT_ENEMY) {
        this.spawnBurst(x, y, r, g, b, 5, 3.0, 10, 0.06, 0.5);
      } else if (type === EVENT_HELPER_DEATH) {
        this.spawnBurst(x, y, r, g, b, 16, 5.0, 22, 0.12, 0.7);
      } else if (type === EVENT_OBJECT_DEATH) {
        this.spawnBurst(x, y, r, g, b, 16, 5.0, 22, 0.12, 0.7);
      } else if (type === EVENT_GENERATOR_SEALED) {
        this.spawnBurst(x, y, r, g, b, 20, 4.0, 28, 0.14, 0.8);
      } else if (type === EVENT_BOSS_DEATH) {
        this.spawnBurst(x, y, r, g, b, 40, 6.0, 50, 0.18, 0.9);
        this.spawnBurst(x, y, 1.0, 1.0, 0.8, 20, 3.0, 40, 0.10, 0.7);
      }
    }
  }

  update(): void {
    let i = 0;
    while (i < this.particles.length) {
      const p = this.particles[i];
      p.x += p.vx / 60;
      p.y += p.vy / 60;
      p.vx *= 0.96;
      p.vy *= 0.96;
      p.life--;
      p.size = Math.max(0, p.size - p.sizeDecay / 60);
      if (p.life <= 0 || p.size <= 0) {
        this.particles[i] = this.particles[this.particles.length - 1];
        this.particles.pop();
      } else {
        i++;
      }
    }
  }

  writeInstances(buffer: Float32Array, offset: number): number {
    let written = 0;
    for (const p of this.particles) {
      const t = p.life / p.maxLife;
      const alpha = p.alpha * t;
      if (alpha < 0.01) continue;

      const idx = offset + written * FLOATS_PER_INSTANCE;
      if (idx + FLOATS_PER_INSTANCE > buffer.length) break;

      buffer[idx + 0] = p.x;          // pos x
      buffer[idx + 1] = p.y;          // pos y
      buffer[idx + 2] = p.size;       // width
      buffer[idx + 3] = p.size;       // height
      buffer[idx + 4] = 0;            // rotation
      buffer[idx + 5] = SPRITE_SOFT_CIRCLE; // sprite
      buffer[idx + 6] = p.r;          // color r
      buffer[idx + 7] = p.g;          // color g
      buffer[idx + 8] = p.b;          // color b
      buffer[idx + 9] = alpha;        // color a
      buffer[idx + 10] = 6.0;         // layer (on top)
      buffer[idx + 11] = 1.0;         // world rotate
      buffer[idx + 12] = 0.0;         // world spin
      buffer[idx + 13] = 0.0;         // screen lock
      buffer[idx + 14] = p.glow;      // glow
      buffer[idx + 15] = 1.0;         // blend mode (additive)

      written++;
    }
    return written;
  }

  get count(): number {
    return this.particles.length;
  }

  private spawnBurst(
    x: number,
    y: number,
    r: number,
    g: number,
    b: number,
    count: number,
    speed: number,
    life: number,
    size: number,
    glow: number,
  ): void {
    for (let i = 0; i < count; i++) {
      if (this.particles.length >= MAX_PARTICLES) break;
      const angle = Math.random() * Math.PI * 2;
      const spd = speed * (0.4 + Math.random() * 0.6);
      const lifeVariance = life * (0.7 + Math.random() * 0.3);
      this.particles.push({
        x: x + (Math.random() - 0.5) * 0.1,
        y: y + (Math.random() - 0.5) * 0.1,
        vx: Math.cos(angle) * spd,
        vy: Math.sin(angle) * spd,
        r: r + (Math.random() - 0.5) * 0.15,
        g: g + (Math.random() - 0.5) * 0.15,
        b: b + (Math.random() - 0.5) * 0.15,
        alpha: 0.7 + Math.random() * 0.3,
        size: size * (0.6 + Math.random() * 0.4),
        sizeDecay: size / lifeVariance * 0.8,
        life: Math.floor(lifeVariance),
        maxLife: Math.floor(lifeVariance),
        glow,
      });
    }
  }
}
