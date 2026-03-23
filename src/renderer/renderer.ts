import type { FrameMeta, RenderViews } from "../types";
import { VIEW_WORLD_SIZE } from "../config";
import type { AtlasBundle } from "./atlas";
import { createProgram } from "./shader";

const FLOATS_PER_INSTANCE = 14;

export class Renderer {
  private readonly gl: WebGL2RenderingContext;
  private readonly canvas: HTMLCanvasElement;
  private readonly wasmMemory: WebAssembly.Memory;
  private readonly program: WebGLProgram;
  private readonly vao: WebGLVertexArrayObject;
  private readonly quadBuffer: WebGLBuffer;
  private readonly instanceBuffer: WebGLBuffer;
  private readonly texture: WebGLTexture;
  private readonly mainUniforms: {
    viewSize: WebGLUniformLocation | null;
    cameraCenter: WebGLUniformLocation | null;
    worldRotationDeg: WebGLUniformLocation | null;
    atlasGrid: WebGLUniformLocation | null;
    atlas: WebGLUniformLocation | null;
  };
  private instanceCapacityBytes = 0;

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
    this.mainUniforms = {
      viewSize: gl.getUniformLocation(this.program, "uViewSize"),
      cameraCenter: gl.getUniformLocation(this.program, "uCameraCenter"),
      worldRotationDeg: gl.getUniformLocation(this.program, "uWorldRotationDeg"),
      atlasGrid: gl.getUniformLocation(this.program, "uAtlasGrid"),
      atlas: gl.getUniformLocation(this.program, "uAtlas"),
    };
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
    gl.viewport(0, 0, gl.canvas.width, gl.canvas.height);
    gl.clearColor(0.12, 0.15, 0.22, 1.0);
    gl.clear(gl.COLOR_BUFFER_BIT);

    const wasmBuffer = this.wasmMemory.buffer;
    const instances = new Float32Array(wasmBuffer, views.instance_ptr, views.instance_len);
    gl.useProgram(this.program);
    gl.bindVertexArray(this.vao);
    gl.bindBuffer(gl.ARRAY_BUFFER, this.instanceBuffer);
    this.uploadBuffer(this.instanceBuffer, instances);
    gl.uniform2f(this.mainUniforms.viewSize, viewWorldSize, viewWorldSize);
    gl.uniform2f(this.mainUniforms.cameraCenter, cameraX, cameraY);
    gl.uniform1f(this.mainUniforms.worldRotationDeg, worldRotationDeg);
    gl.uniform2f(this.mainUniforms.atlasGrid, 4, 4);
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, this.texture);
    gl.uniform1i(this.mainUniforms.atlas, 0);
    const instanceCount = Math.floor(views.instance_len / views.floats_per_instance);
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
      [2, 2],
      [3, 2],
      [4, 1],
      [5, 1],
      [6, 4],
      [7, 1],
      [8, 1],
      [9, 1],
      [10, 1],
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

uniform vec2 uViewSize;
uniform vec2 uCameraCenter;
uniform float uWorldRotationDeg;
uniform vec2 uAtlasGrid;

out vec2 vUv;
out vec4 vColor;

void main() {
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
  vUv = spriteMin + aQuadUv * spriteSize;
  vColor = iColor;
}`;

const FRAGMENT_SOURCE = `#version 300 es
precision highp float;

uniform sampler2D uAtlas;

in vec2 vUv;
in vec4 vColor;
out vec4 outColor;

void main() {
  vec4 texel = texture(uAtlas, vUv);
  outColor = texel * vColor;
}`;
