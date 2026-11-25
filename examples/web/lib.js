// Pure logic for the Stitch Me Up web demo. No DOM access here — app.js owns the DOM.

export const MAX_FILES = 24;
export const MAX_IMAGE_PIXELS = 25_000_000;
export const MAX_TOTAL_PIXELS = 100_000_000;
export const WARN_TOTAL_PIXELS = 60_000_000;

export function pixelCount(width, height) {
  return width * height;
}

// Returns null when the file is acceptable, otherwise a short human message naming it.
export function validateFile({ name, type }, pixels) {
  if (type !== "image/png" && type !== "image/jpeg") {
    return `${name}: only PNG and JPEG images are supported (got ${type || "unknown type"})`;
  }
  if (!Number.isFinite(pixels) || pixels <= 0) {
    return `${name}: could not read image dimensions`;
  }
  if (pixels > MAX_IMAGE_PIXELS) {
    return `${name}: image is too large (${pixels.toLocaleString()} pixels; max ${MAX_IMAGE_PIXELS.toLocaleString()})`;
  }
  return null;
}

export function canAddFiles(currentCount, incomingCount) {
  if (currentCount + incomingCount > MAX_FILES) {
    return {
      ok: false,
      message: `too many files: ${currentCount + incomingCount} would exceed the ${MAX_FILES}-file limit`,
    };
  }
  return { ok: true, message: "" };
}

export function totalPixels(items) {
  return items.reduce((sum, item) => sum + pixelCount(item.width, item.height), 0);
}

export function totalWarning(total) {
  if (total > WARN_TOTAL_PIXELS) {
    return `Large input: ${total.toLocaleString()} pixels total (over ${WARN_TOTAL_PIXELS.toLocaleString()}); composition may be slow`;
  }
  return null;
}

function clampIndex(index, length) {
  if (!Number.isInteger(index) || index < 0 || index >= length) return null;
  return index;
}

export function moveItem(items, from, to) {
  const fromIndex = clampIndex(from, items.length);
  const toIndex = clampIndex(to, items.length);
  if (fromIndex === null || toIndex === null) return [...items];
  const next = [...items];
  const [item] = next.splice(fromIndex, 1);
  next.splice(toIndex, 0, item);
  return next;
}

export function moveUp(items, index) {
  return moveItem(items, index, index - 1);
}

export function moveDown(items, index) {
  return moveItem(items, index, index + 1);
}

export function removeAt(items, index) {
  const fromIndex = clampIndex(index, items.length);
  if (fromIndex === null) return [...items];
  const next = [...items];
  next.splice(fromIndex, 1);
  return next;
}

export function isLayoutValid(layout, imageCount, columns) {
  if (layout === "before-after") {
    if (imageCount !== 2) {
      return {
        ok: false,
        message: `before-after layout requires exactly two images (got ${imageCount})`,
      };
    }
    return { ok: true, message: "" };
  }
  if (layout === "grid") {
    if (!Number.isInteger(columns) || columns < 1) {
      return { ok: false, message: "grid layout requires at least one column" };
    }
    if (imageCount < 2) {
      return { ok: false, message: "grid layout requires at least two images" };
    }
    return { ok: true, message: "" };
  }
  if (layout === "vertical" || layout === "horizontal") {
    if (imageCount < 2) {
      return { ok: false, message: "at least two images are required" };
    }
    return { ok: true, message: "" };
  }
  return { ok: false, message: `unknown layout "${layout}"` };
}

export function parseWeights(text) {
  if (typeof text !== "string") return [];
  return text
    .split(",")
    .map((token) => token.trim())
    .filter((token) => token.length > 0)
    .map((token) => Number(token))
    .filter((value) => Number.isInteger(value) && value >= 1);
}

export function formatBytes(bytes) {
  if (!Number.isFinite(bytes) || bytes < 0) return "0 B";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

export function downloadName(date) {
  const pad = (value) => String(value).padStart(2, "0");
  const stamp = `${date.getFullYear()}${pad(date.getMonth() + 1)}${pad(date.getDate())}-${pad(date.getHours())}${pad(date.getMinutes())}${pad(date.getSeconds())}`;
  return `stitch-me-up-${stamp}.png`;
}
