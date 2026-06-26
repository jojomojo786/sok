import { chromium, devices } from 'playwright';
import fs from 'fs';
import path from 'path';

const BASE_URL = (process.env.SOK_BASE_URL || 'http://127.0.0.1:8080').replace(/\/+$/, '');
const RUN_DATE = new Date().toISOString().slice(0, 10);
const OUT = path.resolve(import.meta.dirname, `lazy-hover-smoke-${RUN_DATE}`);
const TS = new Date().toISOString();
const PLACEHOLDER =
  'data:image/gif;base64,R0lGODlhAQABAJAAAAAAAAAAACH5BAEUAAAALAAAAAABAAEAAAICRAEAOw==';

const VIEWPORTS = {
  desktop: { width: 1440, height: 900 },
  mobile: {
    ...devices['iPhone 13'],
    viewport: { width: 390, height: 844 },
    isMobile: true,
    hasTouch: true,
  },
};

function localUrl(pagePath) {
  return new URL(pagePath, `${BASE_URL}/`).href;
}

function buildChecks({ viewportName, card, serverCard, lazy, hover, mobile }) {
  const checks = [
    { name: 'boot_globals_present', passed: card.bootOk, detail: JSON.stringify(card.boot) },
    { name: 'representative_card_present', passed: !!card.href, detail: card.href },
    { name: 'video_preview_overlay_present', passed: card.hasVideoPreview, detail: String(card.hasVideoPreview) },
    { name: 'thumb_has_data_video', passed: !!(serverCard?.dataVideo || card.dataVideo), detail: serverCard?.dataVideo || card.dataVideo },
    { name: 'thumb_has_data_original', passed: !!serverCard?.dataOriginal, detail: serverCard?.dataOriginal },
    { name: 'thumb_starts_with_placeholder', passed: serverCard?.initialSrc === PLACEHOLDER, detail: serverCard?.initialSrc },
    { name: 'lazy_image_resolves', passed: lazy.resolved, detail: JSON.stringify(lazy) },
    { name: 'lazy_without_layout_shift', passed: lazy.layoutShiftOk, detail: JSON.stringify(lazy.layout) },
  ];

  if (viewportName === 'desktop') {
    checks.push(
      { name: 'desktop_hover_preview_video_created', passed: hover.videoCreated, detail: hover.videoSrc },
      { name: 'desktop_hover_preview_video_ready', passed: hover.videoReady, detail: String(hover.readyState) },
      { name: 'desktop_hover_preview_overlay_shown', passed: hover.overlayShown, detail: String(hover.overlayShown) },
    );
  } else {
    checks.push(
      { name: 'mobile_touch_preview_video_created', passed: mobile.videoCreated, detail: mobile.videoSrc },
      { name: 'mobile_touch_preview_overlay_shown', passed: mobile.overlayShown, detail: String(mobile.overlayShown) },
      { name: 'mobile_touch_preview_requires_tap', passed: mobile.requiresTap, detail: String(mobile.requiresTap) },
    );
  }

  return checks;
}

async function collectServerCardState(page) {
  return page.evaluate((placeholder) => {
    const card = document.querySelector('.thumb.vid');
    const cover = card?.querySelector('.thumb-cover');
    return {
      href: card?.querySelector('.thumb-in')?.getAttribute('href') ?? null,
      hasVideoPreview: !!card?.querySelector('.video-preview'),
      dataVideo: cover?.getAttribute('data-video') ?? null,
      dataOriginal: cover?.getAttribute('data-original') ?? null,
      initialSrc: cover?.getAttribute('src') ?? null,
      placeholder,
    };
  }, PLACEHOLDER);
}

async function collectCardState(page) {
  return page.evaluate((placeholder) => {
    const card = document.querySelector('.thumb.vid');
    const cover = card?.querySelector('.thumb-cover');
    const preview = card?.querySelector('.video-preview');
    const thumbImg = card?.querySelector('.thumb-img');
    return {
      bootOk:
        window.isTHUMBS_OR_PLAYER === true &&
        typeof window.myLazyLoad !== 'undefined' &&
        window.directory === '/static/fox-tpl',
      boot: {
        isTHUMBS_OR_PLAYER: window.isTHUMBS_OR_PLAYER,
        directory: window.directory,
        lazyThreshold: window.lazyThreshold,
        is_mobile: window.is_mobile,
        hasMyLazyLoad: typeof window.myLazyLoad !== 'undefined',
        rocketLoaderReady: window.__cfRLUnblockHandlers === true,
      },
      href: card?.querySelector('.thumb-in')?.getAttribute('href') ?? null,
      hasVideoPreview: !!preview,
      dataVideo: cover?.getAttribute('data-video') ?? null,
      dataOriginal: cover?.getAttribute('data-original') ?? null,
      initialSrc: cover?.getAttribute('src') ?? null,
      wasProcessed: cover?.getAttribute('data-was-processed') === 'true',
      loadedClass: cover?.classList.contains('loaded'),
      currentSrc: cover?.currentSrc || cover?.src || null,
      layout: thumbImg
        ? {
            width: thumbImg.getBoundingClientRect().width,
            height: thumbImg.getBoundingClientRect().height,
          }
        : null,
      placeholder,
    };
  }, PLACEHOLDER);
}

async function waitForLazyResolution(page, beforeLayout) {
  return page.evaluate(async (layoutBefore) => {
    const card = document.querySelector('.thumb.vid');
    const cover = card?.querySelector('.thumb-cover');
    const thumbImg = card?.querySelector('.thumb-img');
    if (!cover || !thumbImg) {
      return {
        resolved: false,
        layoutShiftOk: false,
        currentSrc: null,
        wasProcessed: false,
        loadedClass: false,
        layout: null,
      };
    }

    cover.scrollIntoView({ block: 'center', inline: 'nearest' });
    const started = performance.now();
    while (performance.now() - started < 8000) {
      const currentSrc = cover.currentSrc || cover.src || '';
      const resolved =
        cover.getAttribute('data-was-processed') === 'true' ||
        cover.classList.contains('loaded') ||
        (currentSrc && !currentSrc.startsWith('data:image/gif'));
      if (resolved) {
        const layoutAfter = {
          width: thumbImg.getBoundingClientRect().width,
          height: thumbImg.getBoundingClientRect().height,
        };
        const layoutShiftOk =
          Math.abs(layoutAfter.width - layoutBefore.width) < 2 &&
          Math.abs(layoutAfter.height - layoutBefore.height) < 2;
        return {
          resolved: true,
          layoutShiftOk,
          currentSrc,
          wasProcessed: cover.getAttribute('data-was-processed') === 'true',
          loadedClass: cover.classList.contains('loaded'),
          layout: layoutAfter,
        };
      }
      await new Promise((resolve) => setTimeout(resolve, 100));
    }

    return {
      resolved: false,
      layoutShiftOk: false,
      currentSrc: cover.currentSrc || cover.src || null,
      wasProcessed: cover.getAttribute('data-was-processed') === 'true',
      loadedClass: cover.classList.contains('loaded'),
      layout: {
        width: thumbImg.getBoundingClientRect().width,
        height: thumbImg.getBoundingClientRect().height,
      },
    };
  }, beforeLayout);
}

async function desktopHoverPreview(page) {
  const preview = page.locator('.thumb.vid .video-preview').first();
  await preview.scrollIntoViewIfNeeded();
  await preview.hover({ force: true });

  const started = Date.now();
  let state = {
    videoCreated: false,
    videoSrc: null,
    videoReady: false,
    readyState: null,
    overlayShown: false,
  };

  while (Date.now() - started < 8000) {
    state = await page.evaluate(() => {
      const overlay = document.querySelector('.thumb.vid .video-preview');
      const video = overlay?.querySelector('.video-preview__video');
      return {
        videoCreated: !!video,
        videoSrc: video?.getAttribute('src') ?? null,
        videoReady: !!video && video.readyState >= 2,
        readyState: video?.readyState ?? null,
        overlayShown: overlay?.classList.contains('show') ?? false,
      };
    });
    if (state.videoCreated && (state.overlayShown || state.videoReady)) {
      break;
    }
    await page.waitForTimeout(150);
  }

  return state;
}

async function mobileTouchPreview(page) {
  const preview = page.locator('.thumb.vid .video-preview').first();
  await preview.scrollIntoViewIfNeeded();

  const beforeTap = await page.evaluate(() => {
    const overlay = document.querySelector('.thumb.vid .video-preview');
    return {
      videoCreated: !!overlay?.querySelector('.video-preview__video'),
      overlayShown: overlay?.classList.contains('show') ?? false,
    };
  });

  await preview.tap({ force: true });
  await page.waitForTimeout(1200);

  const afterTap = await page.evaluate(() => {
    const overlay = document.querySelector('.thumb.vid .video-preview');
    const video = overlay?.querySelector('.video-preview__video');
    return {
      videoCreated: !!video,
      videoSrc: video?.getAttribute('src') ?? null,
      overlayShown: overlay?.classList.contains('show') ?? false,
    };
  });

  return {
    ...afterTap,
    requiresTap: !beforeTap.videoCreated && afterTap.videoCreated,
  };
}

fs.mkdirSync(OUT, { recursive: true });

const manifest = {
  captured_at: TS,
  tool: 'playwright-chromium-headless',
  base_url: BASE_URL,
  pages: [],
  errors: [],
};

const browser = await chromium.launch({ headless: true });

for (const [viewportName, viewport] of Object.entries(VIEWPORTS)) {
  const context = await browser.newContext(viewport);
  const page = await context.newPage();
  const id = `home__${viewportName}`;
  const url = localUrl('/');

  try {
    const response = await page.goto(url, { waitUntil: 'domcontentloaded', timeout: 30000 });
    const serverCard = await collectServerCardState(page);
    await page.waitForLoadState('networkidle', { timeout: 8000 }).catch(() => null);
    await page.waitForFunction(
      () =>
        window.__cfRLUnblockHandlers === true &&
        typeof window.myLazyLoad !== 'undefined' &&
        document.querySelectorAll('.thumb.vid .thumb-cover').length > 0,
      null,
      { timeout: 15000 },
    );

    const card = await collectCardState(page);
    const lazy = await waitForLazyResolution(page, card.layout);
    const hover =
      viewportName === 'desktop'
        ? await desktopHoverPreview(page)
        : { videoCreated: false, videoReady: false, overlayShown: false, videoSrc: null, readyState: null };
    const mobile = viewportName === 'mobile' ? await mobileTouchPreview(page) : {
      videoCreated: false,
      overlayShown: false,
      videoSrc: null,
      requiresTap: false,
    };

    const checks = buildChecks({ viewportName, card, serverCard, lazy, hover, mobile });
    const passed = checks.every((check) => check.passed);
    const screenshotPath = path.join(OUT, `${id}.png`);
    const snippetPath = path.join(OUT, `${id}.json`);

    await page.screenshot({ path: screenshotPath, fullPage: false });
    fs.writeFileSync(
      snippetPath,
      JSON.stringify(
        {
          url,
          final_url: page.url(),
          fetched_at: TS,
          viewport: viewportName,
          http_status: response?.status() ?? null,
          serverCard,
          card,
          lazy,
          hover,
          mobile,
          checks,
          passed,
        },
        null,
        2,
      ),
    );

    manifest.pages.push({
      key: 'home',
      viewport: viewportName,
      url,
      final_url: page.url(),
      http_status: response?.status() ?? null,
      screenshot: path.relative(import.meta.dirname, screenshotPath),
      snippet: path.relative(import.meta.dirname, snippetPath),
      passed,
    });

    for (const check of checks) {
      if (!check.passed) {
        manifest.errors.push({
          key: 'home',
          viewport: viewportName,
          check: check.name,
          detail: check.detail ?? null,
        });
      }
    }

    console.log(passed ? 'ok' : 'fail', id, response?.status() ?? null);
  } catch (error) {
    manifest.errors.push({
      key: 'home',
      viewport: viewportName,
      error: String(error),
    });
    console.error('fail', id, error.message);
  } finally {
    await context.close();
  }
}

await browser.close();

const manifestPath = path.join(OUT, 'manifest.json');
fs.writeFileSync(manifestPath, JSON.stringify(manifest, null, 2));
console.log('manifest written', path.relative(process.cwd(), manifestPath));

if (manifest.errors.length > 0) {
  console.error(`${manifest.errors.length} lazy/hover smoke check(s) failed`);
  process.exitCode = 1;
}
