import { readFileSync } from "node:fs";
import { inflateSync, deflateSync } from "node:zlib";

const buf = readFileSync("src-tauri/icons/icon.png");

// Parse PNG chunks
let pos = 8;
let width = 0, height = 0, bitDepth = 0, colorType = 0;
const idat = [];
while (pos < buf.length) {
  const len = buf.readUInt32BE(pos);
  const type = buf.toString("ascii", pos + 4, pos + 8);
  const data = buf.subarray(pos + 8, pos + 8 + len);
  if (type === "IHDR") {
    width = data.readUInt32BE(0);
    height = data.readUInt32BE(4);
    bitDepth = data[8];
    colorType = data[9];
  } else if (type === "IDAT") {
    idat.push(data);
  } else if (type === "IEND") break;
  pos += 12 + len;
}

console.log(`size=${width}x${height} bitDepth=${bitDepth} colorType=${colorType}`);
if (bitDepth !== 8 || colorType !== 6) {
  console.log("Unsupported format for this quick script");
  process.exit(1);
}

const DOWNSCALE = process.argv.includes("--downscale");

const raw = inflateSync(Buffer.concat(idat));
const bpp = 4;
const stride = width * bpp;
const pixels = Buffer.alloc(height * stride);

// Unfilter
for (let y = 0; y < height; y++) {
  const filter = raw[y * (stride + 1)];
  const rowStart = y * (stride + 1) + 1;
  for (let x = 0; x < stride; x++) {
    const cur = raw[rowStart + x];
    const left = x >= bpp ? pixels[y * stride + x - bpp] : 0;
    const up = y > 0 ? pixels[(y - 1) * stride + x] : 0;
    const upLeft = y > 0 && x >= bpp ? pixels[(y - 1) * stride + x - bpp] : 0;
    let val;
    switch (filter) {
      case 0: val = cur; break;
      case 1: val = cur + left; break;
      case 2: val = cur + up; break;
      case 3: val = cur + Math.floor((left + up) / 2); break;
      case 4: {
        const p = left + up - upLeft;
        const pa = Math.abs(p - left), pb = Math.abs(p - up), pc = Math.abs(p - upLeft);
        const pred = pa <= pb && pa <= pc ? left : pb <= pc ? up : upLeft;
        val = cur + pred;
        break;
      }
      default: val = cur;
    }
    pixels[y * stride + x] = val & 0xff;
  }
}

if (DOWNSCALE) {
  // Downscale to 256x256 with 2x2 box filter, re-encode as PNG (filter 0), write + base64
  const S = 256;
  const out = Buffer.alloc(S * S * 4);
  for (let y = 0; y < S; y++) {
    for (let x = 0; x < S; x++) {
      let r = 0, g = 0, b = 0, a = 0;
      for (let dy = 0; dy < 2; dy++) {
        for (let dx = 0; dx < 2; dx++) {
          const idx = ((y * 2 + dy) * width + (x * 2 + dx)) * 4;
          const pa = pixels[idx + 3];
          r += pixels[idx] * pa; g += pixels[idx + 1] * pa; b += pixels[idx + 2] * pa;
          a += pa;
        }
      }
      const oi = (y * S + x) * 4;
      out[oi] = a > 0 ? Math.round(r / a) : 0;
      out[oi + 1] = a > 0 ? Math.round(g / a) : 0;
      out[oi + 2] = a > 0 ? Math.round(b / a) : 0;
      out[oi + 3] = Math.round(a / 4);
    }
  }
  // Encode PNG: try each filter type, keep the smallest result
  const filtered = (ft) => {
    const rawOut = Buffer.alloc((S * 4 + 1) * S);
    for (let y = 0; y < S; y++) {
      rawOut[y * (S * 4 + 1)] = ft;
      for (let x = 0; x < S * 4; x++) {
        const cur = out[y * S * 4 + x];
        const left = x >= 4 ? out[y * S * 4 + x - 4] : 0;
        const up = y > 0 ? out[(y - 1) * S * 4 + x] : 0;
        const upLeft = y > 0 && x >= 4 ? out[(y - 1) * S * 4 + x - 4] : 0;
        let pred;
        switch (ft) {
          case 1: pred = left; break;
          case 2: pred = up; break;
          case 3: pred = Math.floor((left + up) / 2); break;
          case 4: {
            const p = left + up - upLeft;
            const pa = Math.abs(p - left), pb = Math.abs(p - up), pc = Math.abs(p - upLeft);
            pred = pa <= pb && pa <= pc ? left : pb <= pc ? up : upLeft;
            break;
          }
          default: pred = 0;
        }
        rawOut[y * (S * 4 + 1) + 1 + x] = (cur - pred) & 0xff;
      }
    }
    return deflateSync(rawOut, { level: 9 });
  };
  let idat = filtered(0), bestFt = 0;
  for (let ft = 1; ft <= 4; ft++) {
    const cand = filtered(ft);
    if (cand.length < idat.length) { idat = cand; bestFt = ft; }
  }
  console.log(`best filter: ${bestFt} (idat ${idat.length} bytes)`);
  const chunk = (type, data) => {
    const c = Buffer.alloc(8 + data.length + 4);
    c.writeUInt32BE(data.length, 0);
    c.write(type, 4, "ascii");
    data.copy(c, 8);
    const crcTable = [];
    for (let n = 0; n < 256; n++) {
      let cc = n;
      for (let k = 0; k < 8; k++) cc = cc & 1 ? 0xedb88320 ^ (cc >>> 1) : cc >>> 1;
      crcTable[n] = cc >>> 0;
    }
    let crc = 0xffffffff;
    for (const byte of Buffer.concat([Buffer.from(type, "ascii"), data])) {
      crc = crcTable[(crc ^ byte) & 0xff] ^ (crc >>> 8);
    }
    c.writeUInt32BE((crc ^ 0xffffffff) >>> 0, c.length - 4);
    return c;
  };
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(S, 0); ihdr.writeUInt32BE(S, 4);
  ihdr[8] = 8; ihdr[9] = 6; // 8-bit RGBA
  const png = Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk("IHDR", ihdr),
    chunk("IDAT", idat),
    chunk("IEND", Buffer.alloc(0)),
  ]);
  const { writeFileSync } = await import("node:fs");
  writeFileSync("website/icon.png", png);
  console.log(`website/icon.png written: ${png.length} bytes`);
  console.log(`base64 length: ${Math.ceil(png.length / 3) * 4}`);
  console.log("---BASE64---");
  console.log(png.toString("base64"));
  process.exit(0);
}

// Bucket colors (skip transparent/near-transparent), quantize to /16 buckets
const buckets = new Map();
for (let i = 0; i < pixels.length; i += 4) {
  const a = pixels[i + 3];
  if (a < 128) continue;
  const r = pixels[i], g = pixels[i + 1], b = pixels[i + 2];
  const key = `${r >> 4},${g >> 4},${b >> 4}`;
  const e = buckets.get(key) || { count: 0, r: 0, g: 0, b: 0 };
  e.count++; e.r += r; e.g += g; e.b += b;
  buckets.set(key, e);
}

const sorted = [...buckets.entries()].sort((a, b) => b[1].count - a[1].count);
const total = sorted.reduce((s, [, v]) => s + v.count, 0);
console.log(`opaque pixels: ${total}`);
console.log("Top 24 color buckets (hex, share):");
for (const [key, v] of sorted.slice(0, 24)) {
  const [rq, gq, bq] = key.split(",").map(Number);
  const hex = "#" + [v.r / v.count, v.g / v.count, v.b / v.count]
    .map(c => Math.round(c).toString(16).padStart(2, "0"))
    .join("");
  const pct = ((v.count / total) * 100).toFixed(2);
  const lum = (0.2126 * (v.r / v.count) + 0.7152 * (v.g / v.count) + 0.0722 * (v.b / v.count)) / 255;
  console.log(`${hex}  ${pct}%  lum=${lum.toFixed(2)}  (quantized ${rq},${gq},${bq})`);
}
