const { test, expect } = require("@playwright/test");
const { readFileSync } = require("node:fs");
const { dirname, join } = require("node:path");

const here = __dirname;
const fixture = (name) => join(here, "fixtures", name);

const GIF_B64 = "R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7";

async function waitReady(page) {
  await expect(page.locator("#status")).toHaveText(/Ready/, { timeout: 15_000 });
}

async function upload(page, files) {
  await page.locator("#files").setInputFiles(files);
}

async function waitPreview(page) {
  await expect(page.locator("#preview")).toBeVisible({ timeout: 15_000 });
}

test.describe("web demo", () => {
  test("loads and composes two images", async ({ page }) => {
    await page.goto("./");
    await waitReady(page);

    await upload(page, [fixture("a.png"), fixture("b.png")]);
    await waitPreview(page);

    await expect(page.locator("#meta")).toHaveText(/^\d+×\d+ · /);
    await expect(page.locator("#download")).toHaveAttribute(
      "download",
      /^stitch-me-up-\d{8}-\d{6}\.png$/
    );
  });

  test("layout switches recompose", async ({ page }) => {
    await page.goto("./");
    await waitReady(page);
    await upload(page, [fixture("a.png"), fixture("b.png")]);
    await waitPreview(page);

    const metaBefore = await page.locator("#meta").textContent();
    await page.locator("#layout").selectOption("grid");
    await expect
      .poll(async () => await page.locator("#meta").textContent(), { timeout: 15_000 })
      .not.toBe(metaBefore);

    const gridMeta = await page.locator("#meta").textContent();
    await page.locator("#layout").selectOption("before-after");
    await expect
      .poll(async () => await page.locator("#meta").textContent(), { timeout: 15_000 })
      .not.toBe(gridMeta);

    await expect(page.locator("#error")).toBeHidden();
  });

  test("before-after rejects a third image", async ({ page }) => {
    await page.goto("./");
    await waitReady(page);
    await upload(page, [fixture("a.png"), fixture("b.png")]);
    await waitPreview(page);

    await page.locator("#layout").selectOption("before-after");
    await expect(page.locator("#error")).toBeHidden();

    await upload(page, [fixture("c.png")]);
    await expect(page.locator("#status")).toContainText("exactly two", { timeout: 15_000 });
    await expect(page.locator("#preview")).toBeVisible();
  });

  test("failed decode keeps the previous preview alive", async ({ page }) => {
    await page.addInitScript(() => {
      const original = window.createImageBitmap.bind(window);
      window.__rejectNextBitmap = false;
      window.createImageBitmap = (...args) => {
        if (window.__rejectNextBitmap) {
          window.__rejectNextBitmap = false;
          return Promise.reject(new Error("decode failed"));
        }
        return original(...args);
      };
    });

    await page.goto("./");
    await waitReady(page);
    await upload(page, [fixture("a.png"), fixture("b.png")]);
    await waitPreview(page);

    const srcBefore = await page.locator("#preview").getAttribute("src");
    expect(srcBefore).toBeTruthy();

    await page.evaluate(() => {
      window.__rejectNextBitmap = true;
    });
    await page.locator("#spacing-x").fill("4");

    await expect(page.locator("#error")).toContainText("Could not compose", { timeout: 15_000 });

    const srcAfter = await page.locator("#preview").getAttribute("src");
    expect(srcAfter).toBe(srcBefore);

    const status = await page.evaluate(async (src) => {
      const response = await fetch(src);
      return response.status;
    }, srcBefore);
    expect(status).toBe(200);
  });

  test("upload notices survive a successful compose", async ({ page }) => {
    await page.goto("./");
    await waitReady(page);

    await upload(page, [
      { name: "a.png", mimeType: "image/png", buffer: readFileSync(fixture("a.png")) },
      { name: "b.png", mimeType: "image/png", buffer: readFileSync(fixture("b.png")) },
      {
        name: "x.gif",
        mimeType: "image/gif",
        buffer: Buffer.from(GIF_B64, "base64"),
      },
    ]);

    await expect(page.locator("#upload-errors")).toContainText("x.gif", { timeout: 15_000 });
    await waitPreview(page);

    await expect(page.locator("#upload-errors")).toBeVisible();
    await expect(page.locator("#error")).toBeHidden();
  });

  test("jpeg download switches format", async ({ page }) => {
    await page.goto("./");
    await waitReady(page);
    await upload(page, [fixture("a.png"), fixture("b.png")]);
    await waitPreview(page);

    await expect(page.locator("#quality")).toBeDisabled();

    await page.locator("#format").selectOption("jpeg");
    await expect(page.locator("#quality")).toBeEnabled();
    await expect(page.locator("#meta")).toHaveText(/^\d+×\d+ · /, { timeout: 15_000 });
    await expect(page.locator("#download")).toHaveAttribute("download", /\.jpg$/, { timeout: 15_000 });

    const src = await page.locator("#preview").getAttribute("src");
    const contentType = await page.evaluate(async (s) => {
      const res = await fetch(s);
      return res.headers.get("content-type");
    }, src);
    expect(contentType).toMatch(/image\/jpeg/);

    await page.locator("#format").selectOption("png");
    await expect(page.locator("#quality")).toBeDisabled();
  });

  test("missing wasm module shows guidance", async ({ page }) => {
    await page.route("**/pkg/stitch_wasm.js", (route) => route.abort());
    await page.goto("./");

    await expect(page.locator("#error")).toBeVisible({ timeout: 15_000 });
    await expect(page.locator("#error")).toContainText(/build the module|failed to load/i);
  });

  test("drop is refused after init failure", async ({ page }) => {
    await page.route("**/pkg/stitch_wasm_bg.wasm", (route) => route.abort());
    await page.goto("./");

    await expect(page.locator("#error")).toBeVisible({ timeout: 15_000 });

    const b64 = readFileSync(fixture("a.png")).toString("base64");
    await page.evaluate((base64) => {
      const bytes = Uint8Array.from(atob(base64), (c) => c.charCodeAt(0));
      const file = new File([bytes], "a.png", { type: "image/png" });
      const dt = new DataTransfer();
      dt.items.add(file);
      const event = new DragEvent("drop", { bubbles: true, cancelable: true, dataTransfer: dt });
      document.getElementById("drop").dispatchEvent(event);
    }, b64);

    await expect(page.locator(".thumb")).toHaveCount(0, { timeout: 15_000 });
    await expect(page.locator("#upload-errors")).toContainText(/failed to load/i);
  });
});
