import type { AtlasMeta, FrameMeta, RenderViews } from "../types";
import { VIEW_WORLD_SIZE } from "../config";
import type { AtlasBundle } from "./atlas";
import { createProgram } from "./shader";
import { ParticleSystem } from "./particles";

const FLOATS_PER_INSTANCE = 16;

export class Renderer {
  private readonly gl: WebGL2RenderingContext;
  private readonly canvas: HTMLCanvasElement;
  private readonly wasmMemory: WebAssembly.Memory;
  private readonly program: WebGLProgram;
  private readonly vao: WebGLVertexArrayObject;
  private readonly quadBuffer: WebGLBuffer;
  private readonly instanceBuffer: WebGLBuffer;
  private readonly texture: WebGLTexture;
  private readonly atlasMeta: AtlasMeta;
  private readonly mainUniforms: {
    viewSize: WebGLUniformLocation | null;
    cameraCenter: WebGLUniformLocation | null;
    worldRotationDeg: WebGLUniformLocation | null;
    atlasGrid: WebGLUniformLocation | null;
    atlas: WebGLUniformLocation | null;
    time: WebGLUniformLocation | null;
    passBlendMode: WebGLUniformLocation | null;
  };
  private instanceCapacityBytes = 0;
  private frameCount = 0;
  private readonly particles: ParticleSystem;
  private particleBuffer: Float32Array;

  constructor(canvas: HTMLCanvasElement, atlas: AtlasBundle, wasmMemory: WebAssembly.Memory) {
    const gl = canvas.getContext("webgl2");
    if (!gl) {
      throw new Error("WebGL2 unavailable");
    }
    this.canvas = canvas;
    this.wasmMemory = wasmMemory;
    this.gl = gl;
    this.program = createProgram(gl, VERTEX_SOURCE, FRAGMENT_SOURCE);
    const vao = gl.createVertexArray();
    const quadBuffer = gl.createBuffer();
    const instanceBuffer = gl.createBuffer();
    const texture = gl.createTexture();
    if (!vao || !quadBuffer || !instanceBuffer || !texture) {
      throw new Error("failed to allocate GL resources");
    }
    this.vao = vao;
    this.quadBuffer = quadBuffer;
    this.instanceBuffer = instanceBuffer;
    this.texture = texture;
    this.atlasMeta = atlas.meta;
    this.mainUniforms = {
      viewSize: gl.getUniformLocation(this.program, "uViewSize"),
      cameraCenter: gl.getUniformLocation(this.program, "uCameraCenter"),
      worldRotationDeg: gl.getUniformLocation(this.program, "uWorldRotationDeg"),
      atlasGrid: gl.getUniformLocation(this.program, "uAtlasGrid"),
      atlas: gl.getUniformLocation(this.program, "uAtlas"),
      time: gl.getUniformLocation(this.program, "uTime"),
      passBlendMode: gl.getUniformLocation(this.program, "uPassBlendMode"),
    };
    this.particles = new ParticleSystem();
    this.particleBuffer = new Float32Array(2000 * FLOATS_PER_INSTANCE);
    this.initGeometry();
    this.initTexture(atlas);
  }

  render(
    views: RenderViews,
    meta: FrameMeta | null,
    cameraX: number,
    cameraY: number,
    viewWorldSize: number,
    worldRotationDeg: number,
  ) {
    const { gl } = this;
    this.frameCount++;
    gl.viewport(0, 0, gl.canvas.width, gl.canvas.height);
    gl.clearColor(0.06, 0.07, 0.10, 1.0);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.enable(gl.BLEND);
    gl.depthMask(false);

    const wasmBuffer = this.wasmMemory.buffer;

    // Process events for particles
    if (views.event_len > 0) {
      const events = new Float32Array(wasmBuffer, views.event_ptr, views.event_len);
      this.particles.processEvents(events);
    }
    this.particles.update();

    // Combine WASM instances + particle instances
    const wasmInstances = new Float32Array(wasmBuffer, views.instance_ptr, views.instance_len);
    const particleCount = this.particles.count;
    const totalFloats = views.instance_len + particleCount * FLOATS_PER_INSTANCE;

    let combined: Float32Array;
    if (particleCount > 0) {
      // Ensure particle buffer is large enough
      const neededParticleFloats = particleCount * FLOATS_PER_INSTANCE;
      if (this.particleBuffer.length < neededParticleFloats) {
        this.particleBuffer = new Float32Array(neededParticleFloats * 2);
      }
      const writtenParticles = this.particles.writeInstances(this.particleBuffer, 0);
      const particleFloats = writtenParticles * FLOATS_PER_INSTANCE;

      combined = new Float32Array(views.instance_len + particleFloats);
      combined.set(wasmInstances);
      combined.set(this.particleBuffer.subarray(0, particleFloats), views.instance_len);
    } else {
      combined = wasmInstances;
    }

    gl.useProgram(this.program);
    gl.bindVertexArray(this.vao);
    gl.bindBuffer(gl.ARRAY_BUFFER, this.instanceBuffer);
    this.uploadBuffer(this.instanceBuffer, combined);
    gl.uniform2f(this.mainUniforms.viewSize, viewWorldSize, viewWorldSize);
    gl.uniform2f(this.mainUniforms.cameraCenter, cameraX, cameraY);
    gl.uniform1f(this.mainUniforms.worldRotationDeg, worldRotationDeg);
    gl.uniform2f(this.mainUniforms.atlasGrid, this.atlasMeta.cols, this.atlasMeta.rows);
    gl.uniform1f(this.mainUniforms.time, this.frameCount / 60.0);
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, this.texture);
    gl.uniform1i(this.mainUniforms.atlas, 0);
    const instanceCount = Math.floor(combined.length / FLOATS_PER_INSTANCE);

    // Pass 1: Normal alpha blending (blend_mode == 0.0)
    gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
    gl.uniform1f(this.mainUniforms.passBlendMode, 0.0);
    gl.drawArraysInstanced(gl.TRIANGLES, 0, 6, instanceCount);

    // Pass 2: Additive blending (blend_mode == 1.0)
    gl.blendFunc(gl.SRC_ALPHA, gl.ONE);
    gl.uniform1f(this.mainUniforms.passBlendMode, 1.0);
    gl.drawArraysInstanced(gl.TRIANGLES, 0, 6, instanceCount);
  }

  private initGeometry() {
    const { gl } = this;
    const quad = new Float32Array([
      -0.5, -0.5, 0, 0,
      0.5, -0.5, 1, 0,
      0.5, 0.5, 1, 1,
      -0.5, -0.5, 0, 0,
      0.5, 0.5, 1, 1,
      -0.5, 0.5, 0, 1,
    ]);

    gl.bindVertexArray(this.vao);
    gl.bindBuffer(gl.ARRAY_BUFFER, this.quadBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, quad, gl.STATIC_DRAW);

    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 16, 0);
    gl.enableVertexAttribArray(1);
    gl.vertexAttribPointer(1, 2, gl.FLOAT, false, 16, 8);

    gl.bindBuffer(gl.ARRAY_BUFFER, this.instanceBuffer);
    const stride = FLOATS_PER_INSTANCE * 4;
    const descriptors = [
      [2, 2],   // iPos
      [3, 2],   // iSize
      [4, 1],   // iRotationDeg
      [5, 1],   // iSprite
      [6, 4],   // iColor
      [7, 1],   // iLayer
      [8, 1],   // iWorldRotate
      [9, 1],   // iWorldSpin
      [10, 1],  // iScreenLock
      [11, 1],  // iGlow
      [12, 1],  // iBlendMode
    ] as const;
    let offset = 0;
    for (const [location, size] of descriptors) {
      gl.enableVertexAttribArray(location);
      gl.vertexAttribPointer(location, size, gl.FLOAT, false, stride, offset);
      gl.vertexAttribDivisor(location, 1);
      offset += size * 4;
    }
  }

  private initTexture(atlas: AtlasBundle) {
    const { gl } = this;
    gl.bindTexture(gl.TEXTURE_2D, this.texture);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.pixelStorei(gl.UNPACK_PREMULTIPLY_ALPHA_WEBGL, false);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, atlas.texture);
  }

  private uploadBuffer(
    buffer: WebGLBuffer,
    data: Float32Array,
  ) {
    const { gl } = this;
    const byteLength = data.byteLength;
    const currentCapacity = this.instanceCapacityBytes;
    if (byteLength > currentCapacity) {
      let nextCapacity = Math.max(4096, currentCapacity || 4096);
      while (nextCapacity < byteLength) {
        nextCapacity *= 2;
      }
      gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
      gl.bufferData(gl.ARRAY_BUFFER, nextCapacity, gl.DYNAMIC_DRAW);
      this.instanceCapacityBytes = nextCapacity;
    }
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bufferSubData(gl.ARRAY_BUFFER, 0, data);
  }
}

const VERTEX_SOURCE = `#version 300 es
precision highp float;

layout(location = 0) in vec2 aQuadPos;
layout(location = 1) in vec2 aQuadUv;
layout(location = 2) in vec2 iPos;
layout(location = 3) in vec2 iSize;
layout(location = 4) in float iRotationDeg;
layout(location = 5) in float iSprite;
layout(location = 6) in vec4 iColor;
layout(location = 7) in float iLayer;
layout(location = 8) in float iWorldRotate;
layout(location = 9) in float iWorldSpin;
layout(location = 10) in float iScreenLock;
layout(location = 11) in float iGlow;
layout(location = 12) in float iBlendMode;

uniform vec2 uViewSize;
uniform vec2 uCameraCenter;
uniform float uWorldRotationDeg;
uniform vec2 uAtlasGrid;
uniform float uTime;
uniform float uPassBlendMode;

out vec2 vUv;
out vec4 vColor;
out vec2 vLocalUv;
out float vGlow;
out float vBlendMode;

void main() {
  // Discard instances that don't belong to this pass
  if (abs(iBlendMode - uPassBlendMode) > 0.5) {
    gl_Position = vec4(2.0, 2.0, 2.0, 1.0);
    return;
  }

  float rotation = radians(iRotationDeg + uWorldRotationDeg * iWorldSpin);
  mat2 localRot = mat2(cos(rotation), -sin(rotation), sin(rotation), cos(rotation));
  float worldRotation = radians(uWorldRotationDeg);
  mat2 viewRot = mat2(cos(worldRotation), -sin(worldRotation), sin(worldRotation), cos(worldRotation));
  vec2 local = localRot * (aQuadPos * iSize);
  vec2 centered = mix(iPos - uCameraCenter, viewRot * (iPos - uCameraCenter), iWorldRotate);
  centered = mix(centered, vec2(0.0), iScreenLock);
  vec2 clip = ((centered + local) / uViewSize) * 2.0;
  clip.y *= -1.0;
  gl_Position = vec4(clip.x, clip.y, iLayer * 0.01, 1.0);

  float col = mod(iSprite, uAtlasGrid.x);
  float row = floor(iSprite / uAtlasGrid.x);
  vec2 spriteMin = vec2(col, row) / uAtlasGrid;
  vec2 spriteSize = vec2(1.0) / uAtlasGrid;
  // Inset UVs by half a texel to prevent bleeding between atlas cells
  vec2 halfTexel = vec2(0.5 / 512.0);
  vUv = spriteMin + halfTexel + aQuadUv * (spriteSize - 2.0 * halfTexel);
  vLocalUv = aQuadUv;
  vColor = iColor;
  vGlow = iGlow;
  vBlendMode = iBlendMode;
}`;

const FRAGMENT_SOURCE = `#version 300 es
precision highp float;

uniform sampler2D uAtlas;
uniform float uTime;

in vec2 vUv;
in vec4 vColor;
in vec2 vLocalUv;
in float vGlow;
in float vBlendMode;
out vec4 outColor;

void main() {
  vec4 texel = texture(uAtlas, vUv);
  vec4 base = texel * vColor;

  if (vGlow > 0.0) {
    float dist = length(vLocalUv - 0.5) * 2.0;
    float glow = exp(-dist * dist * 3.0) * vGlow;
    float pulse = 1.0 + sin(uTime * 4.0) * 0.15;
    glow *= pulse;
    base.rgb += vColor.rgb * glow;
    base.a = max(base.a, glow * 0.4);
  }

  outColor = base;
}`;
