// DOM wiring for the Stitch Me Up web demo. Pure logic lives in lib.js.
import initWasm, { Composer } from "./pkg/stitch_wasm.js";
import {
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
} from "./lib.js";

const $ = (id) => document.getElementById(id);

const state = {
  items: [], // {id, file, name, url, width, height}
  changeToken: 0,
  busy: false,
  lastUrl: null,
  composerReady: false,
};

let nextId = 1;
let debounceTimer = null;

function showError(message) {
  const box = $("error");
  box.textContent = message;
  box.hidden = false;
}

function clearError() {
  const box = $("error");
  box.textContent = "";
  box.hidden = true;
}

function showStatus(message) {
  $("status").textContent = message;
}

function setBusy(busy) {
  state.busy = busy;
  $("busy").hidden = !busy;
}

function clearBusyIfCurrent(token) {
  if (token === state.changeToken) setBusy(false);
}

function emptyMessage() {
  return document.querySelector(".preview-empty");
}

function clearPreview() {
  if (state.lastUrl) {
    URL.revokeObjectURL(state.lastUrl);
    state.lastUrl = null;
  }
  const preview = $("preview");
  preview.hidden = true;
  preview.removeAttribute("src");
  $("meta").textContent = "";
  $("download").hidden = true;
  const empty = emptyMessage();
  if (empty) empty.hidden = false;
}

function renderThumbs() {
  const list = $("thumbs");
  list.innerHTML = "";
  state.items.forEach((item, index) => {
    const li = document.createElement("li");
    li.className = "thumb";
    li.draggable = true;
    li.dataset.id = String(item.id);

    const img = document.createElement("img");
    img.src = item.url;
    img.alt = item.name;
    img.width = 64;
    img.height = 64;

    const name = document.createElement("span");
    name.className = "thumb-name";
    name.textContent = item.name;

    const actions = document.createElement("div");
    actions.className = "thumb-actions";

    const up = document.createElement("button");
    up.type = "button";
    up.textContent = "Move up";
    up.disabled = index === 0;
    up.addEventListener("click", () => {
      state.items = moveUp(state.items, index);
      renderThumbs();
      schedule();
    });

    const down = document.createElement("button");
    down.type = "button";
    down.textContent = "Move down";
    down.disabled = index === state.items.length - 1;
    down.addEventListener("click", () => {
      state.items = moveDown(state.items, index);
      renderThumbs();
      schedule();
    });

    const remove = document.createElement("button");
    remove.type = "button";
    remove.textContent = "Remove";
    remove.addEventListener("click", () => removeItem(index));

    actions.append(up, down, remove);
    li.append(img, name, actions);

    li.addEventListener("dragstart", (event) => {
      event.dataTransfer.setData("text/plain", String(item.id));
      li.classList.add("dragging");
    });
    li.addEventListener("dragend", () => li.classList.remove("dragging"));
    li.addEventListener("dragover", (event) => {
      event.preventDefault();
      li.classList.add("drag-over");
    });
    li.addEventListener("dragleave", () => li.classList.remove("drag-over"));
    li.addEventListener("drop", (event) => {
      event.preventDefault();
      li.classList.remove("drag-over");
      const draggedId = Number(event.dataTransfer.getData("text/plain"));
      const from = state.items.findIndex((i) => i.id === draggedId);
      if (from === -1 || from === index) return;
      state.items = moveItem(state.items, from, index);
      renderThumbs();
      schedule();
    });

    list.appendChild(li);
  });
}

function removeItem(index) {
  const item = state.items[index];
  if (!item) return;
  URL.revokeObjectURL(item.url);
  state.items = removeAt(state.items, index);
  renderThumbs();
  schedule();
}

async function addFiles(fileList) {
  const files = Array.from(fileList);
  if (files.length === 0) return;

  clearError();

  const decoded = [];
  const rejections = [];
  for (const file of files) {
    try {
      const bitmap = await createImageBitmap(file);
      const pixels = pixelCount(bitmap.width, bitmap.height);
      const error = validateFile({ name: file.name, type: file.type }, pixels);
      if (error) {
        rejections.push(error);
      } else {
        decoded.push({ file, name: file.name, width: bitmap.width, height: bitmap.height, pixels });
      }
      bitmap.close();
    } catch (error) {
      rejections.push(`${file.name}: could not read image (${error.message})`);
    }
  }

  if (rejections.length > 0) {
    showError(rejections.join("; "));
  }

  if (decoded.length === 0) {
    if (rejections.length === 0) showError("No images selected.");
    return;
  }

  const check = canAddFiles(state.items.length, decoded.length);
  if (!check.ok) {
    showError(check.message);
    return;
  }

  const incomingPixels = decoded.reduce((sum, d) => sum + d.pixels, 0);
  const total = totalPixels(state.items) + incomingPixels;
  const totalError = totalLimit(total);
  if (totalError) {
    showError(totalError);
    return;
  }

  for (const d of decoded) {
    const url = URL.createObjectURL(d.file);
    state.items.push({ id: nextId++, file: d.file, name: d.name, url, width: d.width, height: d.height });
  }

  renderThumbs();
  const warning = totalWarning(totalPixels(state.items));
  showStatus(warning ?? "");
  schedule();
}

function schedule() {
  state.changeToken++;
  if (debounceTimer !== null) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    debounceTimer = null;
    compose();
  }, 250);
}

async function compose() {
  const token = state.changeToken;

  if (!state.composerReady) {
    showStatus("Loading WebAssembly…");
    clearBusyIfCurrent(token);
    return;
  }

  if (state.items.length === 0) {
    clearPreview();
    showStatus("Add images to start.");
    clearBusyIfCurrent(token);
    return;
  }

  const layout = $("layout").value;
  const columns = Number($("columns").value) || 1;
  const validation = isLayoutValid(layout, state.items.length, columns);
  if (!validation.ok) {
    showStatus(validation.message);
    clearBusyIfCurrent(token);
    return;
  }

  setBusy(true);
  clearError();

  try {
    const buffers = await Promise.all(state.items.map((item) => item.file.arrayBuffer()));
    if (token !== state.changeToken) return;

    const composer = new Composer();
    composer.set_layout(layout, columns);
    if (layout === "grid") {
      const weights = parseWeights($("weights").value);
      if (weights.length > 0) composer.set_column_weights(new Uint32Array(weights));
    }
    composer.set_spacing(Number($("spacing-x").value) || 0, Number($("spacing-y").value) || 0);
    composer.set_padding(
      Number($("pad-top").value) || 0,
      Number($("pad-right").value) || 0,
      Number($("pad-bottom").value) || 0,
      Number($("pad-left").value) || 0
    );
    composer.set_background($("background").value);
    composer.set_alignment($("alignment").value);
    composer.set_resize_mode($("resize-mode").value);
    composer.set_frame(Number($("frame-width").value) || 0, $("frame-color").value);
    composer.set_corner_radius(Number($("corner-radius").value) || 0);
    if (layout === "before-after") {
      composer.set_divider(Number($("divider-width").value) || 0, $("divider-color").value);
    }

    const png = composer.compose(buffers.map((b) => new Uint8Array(b)));
    if (token !== state.changeToken) return;

    if (state.lastUrl) URL.revokeObjectURL(state.lastUrl);
    const blob = new Blob([png], { type: "image/png" });
    state.lastUrl = URL.createObjectURL(blob);

    const bitmap = await createImageBitmap(blob);
    if (token !== state.changeToken) return;

    const preview = $("preview");
    preview.src = state.lastUrl;
    preview.hidden = false;
    const empty = emptyMessage();
    if (empty) empty.hidden = true;

    $("meta").textContent = `${bitmap.width}×${bitmap.height} · ${formatBytes(blob.size)}`;
    const download = $("download");
    download.href = state.lastUrl;
    download.download = downloadName(new Date());
    download.hidden = false;

    const warning = totalWarning(totalPixels(state.items));
    showStatus(warning ?? "");
  } catch (error) {
    if (token !== state.changeToken) return;
    showError(`Could not compose: ${error.message}`);
  } finally {
    clearBusyIfCurrent(token);
  }
}

function setLayoutControls() {
  const layout = $("layout").value;
  const isGrid = layout === "grid";
  const isBeforeAfter = layout === "before-after";
  $("columns").disabled = !isGrid;
  $("weights").disabled = !isGrid;
  $("divider-width").disabled = !isBeforeAfter;
  $("divider-color").disabled = !isBeforeAfter;
}

function disableControls() {
  document.querySelectorAll("input, select, button").forEach((el) => {
    el.disabled = true;
  });
  $("drop").setAttribute("aria-disabled", "true");
}

async function loadVersion() {
  try {
    const response = await fetch("./version.json", { cache: "no-store" });
    if (!response.ok) return;
    const data = await response.json();
    if (data && data.version) {
      const version = $("version");
      version.textContent = `v${data.version}${data.commit ? ` · ${data.commit}` : ""}`;
      version.hidden = false;
    }
  } catch {
    // Missing or unreachable version.json — leave the footer hidden.
  }
}

async function startWasm() {
  try {
    await initWasm();
    state.composerReady = true;
    showStatus("Ready.");
    if (state.items.length > 0) schedule();
  } catch (error) {
    state.composerReady = false;
    showStatus("WebAssembly module failed to load.");
    showError(
      `${error.message} — build the module first with ` +
        "cd crates/stitch-wasm && wasm-pack build --target web --out-dir ../../examples/web/pkg, " +
        "then serve this directory (see examples/web/README.md)."
    );
    disableControls();
  }
}

function wire() {
  $("files").addEventListener("change", (event) => {
    addFiles(event.target.files);
    event.target.value = "";
  });

  const drop = $("drop");
  $("choose").addEventListener("click", (event) => {
    event.stopPropagation();
    $("files").click();
  });
  drop.addEventListener("click", (event) => {
    if (event.target === $("files")) return;
    $("files").click();
  });
  drop.addEventListener("dragover", (event) => {
    event.preventDefault();
    drop.classList.add("drag-over");
  });
  drop.addEventListener("dragleave", () => drop.classList.remove("drag-over"));
  drop.addEventListener("drop", (event) => {
    event.preventDefault();
    drop.classList.remove("drag-over");
    addFiles(event.dataTransfer.files);
  });

  document.querySelectorAll("input, select").forEach((el) => {
    if (el.id === "files") return;
    el.addEventListener("input", schedule);
    el.addEventListener("change", schedule);
  });
  $("layout").addEventListener("change", () => {
    setLayoutControls();
    schedule();
  });
}

async function main() {
  wire();
  setLayoutControls();
  renderThumbs();
  loadVersion();
  await startWasm();
}

main();
