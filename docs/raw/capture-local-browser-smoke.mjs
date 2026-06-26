import { chromium } from 'playwright';
import fs from 'fs';
import path from 'path';

const BASE_URL = (process.env.SOK_BASE_URL || 'http://127.0.0.1:8080').replace(/\/+$/, '');
const RUN_DATE = new Date().toISOString().slice(0, 10);
const OUT = path.resolve(import.meta.dirname, `local-browser-smoke-${RUN_DATE}`);
const REFERENCE_DIR = path.resolve(import.meta.dirname, 'live-inventory-2026-06-26');
const TS = new Date().toISOString();

const PAGES = [
  { key: 'home', path: '/' },
  { key: 'categories', path: '/categories' },
  { key: 'pornstars', path: '/pornstars' },
  {
    key: 'video-sample',
    path: '/video/dog-house-madi-collins-knows-more-sex-than-her-step-bro-decides-to-show-him-what-he-should-do.html',
  },
];

const VIEWPORTS = {
  desktop: { width: 1440, height: 900 },
  mobile: { width: 390, height: 844, isMobile: true, hasTouch: true },
};

function localUrl(pagePath) {
  return new URL(pagePath, `${BASE_URL}/`).href;
}

function loadReferenceMap() {
  const manifestPath = path.join(REFERENCE_DIR, 'manifest.json');
  if (!fs.existsSync(manifestPath)) {
    return new Map();
  }

  const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
  return new Map(
    manifest.pages.map((entry) => {
      const snippetPath = path.resolve(import.meta.dirname, entry.snippet);
      return [
        `${entry.key}:${entry.viewport}`,
        {
          ...entry,
          snippet_json: fs.existsSync(snippetPath)
            ? JSON.parse(fs.readFileSync(snippetPath, 'utf8'))
            : null,
        },
      ];
    }),
  );
}

function trimHtml(html) {
  if (!html) {
    return null;
  }
  const compact = html.replace(/\s+/g, ' ').trim();
  return compact.length > 3000 ? `${compact.slice(0, 3000)}...` : compact;
}

async function extractSnippet(page) {
  return page.evaluate(() => {
    const meta = (name) =>
      document.querySelector(`meta[name="${name}"]`)?.getAttribute('content') ?? null;
    const htmlOf = (selector) => document.querySelector(selector)?.outerHTML ?? null;
    const bool = (selector) => !!document.querySelector(selector);

    const viewportWidth = window.innerWidth;
    const scrollWidth = document.documentElement.scrollWidth;

    return {
      title: document.title,
      canonical: document.querySelector('link[rel="canonical"]')?.getAttribute('href') ?? null,
      lang: document.documentElement.lang,
      theme: document.documentElement.getAttribute('data-theme'),
      h1: [...document.querySelectorAll('h1')]
        .map((h) => h.textContent?.trim())
        .filter(Boolean),
      description: meta('description'),
      header: htmlOf('.header'),
      main: htmlOf('main') || htmlOf('.content') || htmlOf('#content'),
      footer: htmlOf('.footer'),
      bodyClasses: document.body.className,
      viewportWidth,
      scrollWidth,
      hasHorizontalOverflow: scrollWidth > viewportWidth + 4,
      hasFilterSection: bool('.filter-section'),
      hasPageNav: bool('.page_nav'),
      hasPlayer: bool('#player_container, #player_container2'),
      hasSearchBoxPage: bool('#search-page-input'),
      hasSearchGenresInput: bool('#search-genres-input'),
      hasDesktopNav: bool('.header-menu'),
      hasMobileButton: bool('.btn-mob'),
      thumbCount: document.querySelectorAll('.thumb').length,
      videoThumbCount: document.querySelectorAll('.thumb.vid').length,
      hasVideoPreview: bool('.video-preview'),
    };
  });
}

function buildChecks({ key, viewportName, status, snippet, reference, assetFailures }) {
  const checks = [
    { name: 'document_status_200', passed: status === 200, detail: `status=${status}` },
    { name: 'has_header', passed: !!snippet.header },
    { name: 'has_main', passed: !!snippet.main },
    { name: 'has_footer', passed: !!snippet.footer },
    { name: 'has_h1', passed: snippet.h1.length > 0, detail: JSON.stringify(snippet.h1) },
    {
      name: 'no_horizontal_overflow',
      passed: !snippet.hasHorizontalOverflow,
      detail: `viewport=${snippet.viewportWidth} scroll=${snippet.scrollWidth}`,
    },
    {
      name: 'no_failed_local_assets',
      passed: assetFailures.length === 0,
      detail: `${assetFailures.length} failed local asset response(s)`,
    },
  ];

  if (viewportName === 'desktop') {
    checks.push({ name: 'desktop_nav_present', passed: snippet.hasDesktopNav });
  }

  if (key === 'home') {
    checks.push({ name: 'home_has_video_cards', passed: snippet.videoThumbCount > 0 });
    checks.push({ name: 'home_has_video_preview_layers', passed: snippet.hasVideoPreview });
  }

  if (key === 'categories') {
    checks.push({ name: 'categories_have_search_input', passed: snippet.hasSearchGenresInput });
    checks.push({ name: 'categories_have_cards', passed: snippet.thumbCount > 0 });
  }

  if (key === 'pornstars') {
    checks.push({ name: 'pornstars_have_cards', passed: snippet.thumbCount > 0 });
  }

  if (key === 'video-sample') {
    checks.push({ name: 'video_has_player_shell', passed: snippet.hasPlayer });
  }

  const ref = reference?.snippet_json;
  if (ref) {
    const refSignals = [
      ['reference_header_presence', !!ref.header, !!snippet.header],
      ['reference_main_presence', !!ref.main, !!snippet.main],
      ['reference_footer_presence', !!ref.footer, !!snippet.footer],
      ['reference_h1_presence', (ref.h1 ?? []).length > 0, snippet.h1.length > 0],
      ['reference_filter_presence', !!ref.hasFilterSection, !!snippet.hasFilterSection],
      ['reference_page_nav_presence', !!ref.hasPageNav, !!snippet.hasPageNav],
      ['reference_search_genres_presence', !!ref.hasSearchGenresInput, !!snippet.hasSearchGenresInput],
    ];

    for (const [name, expected, actual] of refSignals) {
      if (expected) {
        checks.push({ name, passed: actual === expected, detail: `expected=${expected} actual=${actual}` });
      }
    }
  }

  return checks;
}

function isSameOriginAsset(responseUrl, baseOrigin, resourceType) {
  const criticalTypes = new Set(['stylesheet', 'script', 'image', 'font']);
  if (!criticalTypes.has(resourceType)) {
    return false;
  }

  try {
    return new URL(responseUrl).origin === baseOrigin;
  } catch {
    return false;
  }
}

fs.mkdirSync(OUT, { recursive: true });

const referenceMap = loadReferenceMap();
const manifest = {
  captured_at: TS,
  tool: 'playwright-chromium-headless',
  base_url: BASE_URL,
  reference_dir: path.relative(import.meta.dirname, REFERENCE_DIR),
  pages: [],
  errors: [],
};

const baseOrigin = new URL(BASE_URL).origin;
const browser = await chromium.launch({ headless: true });

for (const [viewportName, viewport] of Object.entries(VIEWPORTS)) {
  const context = await browser.newContext({
    viewport,
    userAgent: viewportName === 'mobile'
      ? 'Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1'
      : undefined,
  });

  const page = await context.newPage();
  for (const smokePage of PAGES) {
    const id = `${smokePage.key}__${viewportName}`;
    const url = localUrl(smokePage.path);
    const assetFailures = [];

    const onResponse = (response) => {
      const request = response.request();
      if (isSameOriginAsset(response.url(), baseOrigin, request.resourceType()) && response.status() >= 400) {
        assetFailures.push({
          url: response.url(),
          status: response.status(),
          resource_type: request.resourceType(),
        });
      }
    };

    const onRequestFailed = (request) => {
      if (isSameOriginAsset(request.url(), baseOrigin, request.resourceType())) {
        assetFailures.push({
          url: request.url(),
          status: null,
          resource_type: request.resourceType(),
          failure: request.failure()?.errorText ?? 'request failed',
        });
      }
    };

    page.on('response', onResponse);
    page.on('requestfailed', onRequestFailed);

    try {
      const response = await page.goto(url, { waitUntil: 'domcontentloaded', timeout: 30000 });
      await page.waitForLoadState('networkidle', { timeout: 5000 }).catch(() => null);
      await page.waitForTimeout(250);

      const status = response?.status() ?? null;
      const snippet = await extractSnippet(page);
      const reference = referenceMap.get(`${smokePage.key}:${viewportName}`) ?? null;
      const checks = buildChecks({
        key: smokePage.key,
        viewportName,
        status,
        snippet,
        reference,
        assetFailures,
      });
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
            http_status: status,
            reference: reference
              ? {
                screenshot: reference.screenshot,
                snippet: reference.snippet,
              }
              : null,
            ...snippet,
            header: trimHtml(snippet.header),
            main: trimHtml(snippet.main),
            footer: trimHtml(snippet.footer),
            asset_failures: assetFailures,
            checks,
            passed,
          },
          null,
          2,
        ),
      );

      manifest.pages.push({
        key: smokePage.key,
        viewport: viewportName,
        url,
        final_url: page.url(),
        http_status: status,
        screenshot: path.relative(import.meta.dirname, screenshotPath),
        snippet: path.relative(import.meta.dirname, snippetPath),
        reference_screenshot: reference?.screenshot ?? null,
        reference_snippet: reference?.snippet ?? null,
        passed,
      });

      for (const check of checks) {
        if (!check.passed) {
          manifest.errors.push({
            key: smokePage.key,
            viewport: viewportName,
            check: check.name,
            detail: check.detail ?? null,
          });
        }
      }

      console.log(passed ? 'ok' : 'fail', id, status);
    } catch (error) {
      manifest.errors.push({
        key: smokePage.key,
        viewport: viewportName,
        error: String(error),
      });
      console.error('fail', id, error.message);
    } finally {
      page.off('response', onResponse);
      page.off('requestfailed', onRequestFailed);
    }
  }

  await context.close();
}

await browser.close();

const manifestPath = path.join(OUT, 'manifest.json');
fs.writeFileSync(manifestPath, JSON.stringify(manifest, null, 2));
console.log('manifest written', path.relative(process.cwd(), manifestPath));

if (manifest.errors.length > 0) {
  console.error(`${manifest.errors.length} browser smoke check(s) failed`);
  process.exitCode = 1;
}
