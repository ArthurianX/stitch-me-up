import test from "node:test";
import assert from "node:assert/strict";

import {
  MAX_FILES,
  MAX_IMAGE_PIXELS,
  MAX_TOTAL_PIXELS,
  WARN_TOTAL_PIXELS,
  pixelCount,
  validateFile,
  canAddFiles,
  totalPixels,
  totalWarning,
  totalLimit,
  moveItem,
  moveUp,
  moveDown,
  removeAt,
  isLayoutValid,
  parseWeights,
  formatBytes,
  downloadName,
  isValidQuality,
  cellPresetToSize,
} from "../lib.js";

test("constants match the spec limits", () => {
  assert.equal(MAX_FILES, 24);
  assert.equal(MAX_IMAGE_PIXELS, 25_000_000);
  assert.equal(MAX_TOTAL_PIXELS, 100_000_000);
  assert.equal(WARN_TOTAL_PIXELS, 60_000_000);
});

test("pixelCount multiplies width and height", () => {
  assert.equal(pixelCount(100, 200), 20_000);
  assert.equal(pixelCount(0, 5), 0);
});

test("validateFile accepts PNG and JPEG within limits", () => {
  assert.equal(validateFile({ name: "a.png", type: "image/png" }, 1_000_000), null);
  assert.equal(validateFile({ name: "b.jpg", type: "image/jpeg" }, 25_000_000), null);
});

test("validateFile rejects unsupported types", () => {
  const gif = validateFile({ name: "a.gif", type: "image/gif" }, 1_000_000);
  assert.match(gif, /a\.gif/);
  assert.match(gif, /PNG|JPEG/i);
  const webp = validateFile({ name: "b.webp", type: "image/webp" }, 1_000_000);
  assert.match(webp, /b\.webp/);
});

test("validateFile rejects zero and oversized images", () => {
  const zero = validateFile({ name: "z.png", type: "image/png" }, 0);
  assert.match(zero, /z\.png/);
  const big = validateFile({ name: "big.png", type: "image/png" }, 25_000_001);
  assert.match(big, /big\.png/);
  assert.match(big, /25/);
});

test("canAddFiles enforces the 24-file cap", () => {
  assert.deepEqual(canAddFiles(0, 2), { ok: true, message: "" });
  assert.deepEqual(canAddFiles(22, 2), { ok: true, message: "" });
  const over = canAddFiles(23, 2);
  assert.equal(over.ok, false);
  assert.match(over.message, /24/);
  const over2 = canAddFiles(24, 1);
  assert.equal(over2.ok, false);
});

test("totalPixels sums width*height across items", () => {
  const items = [
    { width: 100, height: 100 },
    { width: 50, height: 40 },
  ];
  assert.equal(totalPixels(items), 10_000 + 2_000);
  assert.equal(totalPixels([]), 0);
});

test("totalWarning warns above 60 MP but not at or below", () => {
  assert.equal(totalWarning(60_000_000), null);
  assert.equal(totalWarning(59_999_999), null);
  const warning = totalWarning(60_000_001);
  assert.match(warning, /60/);
});

test("moveItem reorders without mutating the input", () => {
  const items = ["a", "b", "c"];
  const moved = moveItem(items, 0, 2);
  assert.deepEqual(moved, ["b", "c", "a"]);
  assert.deepEqual(items, ["a", "b", "c"]);
});

test("moveItem clamps out-of-range indices to a no-op copy", () => {
  const items = ["a", "b"];
  assert.deepEqual(moveItem(items, 5, 0), ["a", "b"]);
  assert.deepEqual(moveItem(items, 0, 9), ["a", "b"]);
  assert.deepEqual(moveItem(items, -1, 0), ["a", "b"]);
});

test("moveUp and moveDown move within bounds", () => {
  const items = ["a", "b", "c"];
  assert.deepEqual(moveUp(items, 1), ["b", "a", "c"]);
  assert.deepEqual(moveDown(items, 1), ["a", "c", "b"]);
  assert.deepEqual(moveUp(items, 0), ["a", "b", "c"]);
  assert.deepEqual(moveDown(items, 2), ["a", "b", "c"]);
});

test("removeAt removes without mutating and clamps out-of-range", () => {
  const items = ["a", "b", "c"];
  assert.deepEqual(removeAt(items, 1), ["a", "c"]);
  assert.deepEqual(items, ["a", "b", "c"]);
  assert.deepEqual(removeAt(items, 9), ["a", "b", "c"]);
  assert.deepEqual(removeAt(items, -1), ["a", "b", "c"]);
});

test("isLayoutValid requires exactly two images for before-after", () => {
  assert.equal(isLayoutValid("before-after", 2, 0).ok, true);
  assert.equal(isLayoutValid("before-after", 0, 0).ok, false);
  assert.equal(isLayoutValid("before-after", 1, 0).ok, false);
  assert.equal(isLayoutValid("before-after", 3, 0).ok, false);
  assert.match(isLayoutValid("before-after", 3, 0).message, /two/);
});

test("isLayoutValid requires columns >= 1 for grid", () => {
  assert.equal(isLayoutValid("grid", 2, 1).ok, true);
  assert.equal(isLayoutValid("grid", 2, 0).ok, false);
  assert.match(isLayoutValid("grid", 2, 0).message, /column/i);
});

test("isLayoutValid accepts other layouts with 2+ images", () => {
  assert.equal(isLayoutValid("vertical", 2, 0).ok, true);
  assert.equal(isLayoutValid("horizontal", 3, 0).ok, true);
  assert.equal(isLayoutValid("vertical", 1, 0).ok, false);
  assert.equal(isLayoutValid("horizontal", 0, 0).ok, false);
});

test("isLayoutValid rejects unknown layouts", () => {
  const result = isLayoutValid("panorama", 2, 0);
  assert.equal(result.ok, false);
  assert.match(result.message, /panorama/);
});

test("parseWeights parses positive integers and drops junk", () => {
  assert.deepEqual(parseWeights("1,2"), [1, 2]);
  assert.deepEqual(parseWeights(" 3 , 4 "), [3, 4]);
  assert.deepEqual(parseWeights(""), []);
  assert.deepEqual(parseWeights("a,b"), []);
  assert.deepEqual(parseWeights("1,0,2"), [1, 2]);
});

test("formatBytes renders human-readable sizes", () => {
  assert.equal(formatBytes(999), "999 B");
  assert.equal(formatBytes(1024), "1.0 KB");
  assert.equal(formatBytes(1.5e6), "1.4 MB");
});

test("downloadName formats a fixed date", () => {
  const date = new Date(2026, 10, 25, 9, 5, 3);
  assert.equal(downloadName(date), "stitch-me-up-20261125-090503.png");
});

test("downloadName honors a jpg extension and sanitizes others", () => {
  const date = new Date(2026, 10, 25, 9, 5, 3);
  assert.equal(downloadName(date, "jpg"), "stitch-me-up-20261125-090503.jpg");
  assert.equal(downloadName(date, "png"), "stitch-me-up-20261125-090503.png");
  assert.equal(downloadName(date, "gif"), "stitch-me-up-20261125-090503.png");
});

test("isValidQuality enforces integer 1..=100", () => {
  assert.equal(isValidQuality(0), false);
  assert.equal(isValidQuality(1), true);
  assert.equal(isValidQuality(90), true);
  assert.equal(isValidQuality(100), true);
  assert.equal(isValidQuality(101), false);
  assert.equal(isValidQuality("90"), false);
  assert.equal(isValidQuality(90.5), false);
});

test("totalLimit rejects totals over 100 MP", () => {
  assert.equal(totalLimit(100_000_000), null);
  assert.equal(totalLimit(99_999_999), null);
  const error = totalLimit(100_000_001);
  assert.match(error, /100/);
});

test("cellPresetToSize maps presets and falls back to off", () => {
  assert.deepEqual(cellPresetToSize("square"), { width: 320, height: 320 });
  assert.deepEqual(cellPresetToSize("portrait"), { width: 320, height: 400 });
  assert.equal(cellPresetToSize("off"), null);
  assert.equal(cellPresetToSize("nonsense"), null);
});
