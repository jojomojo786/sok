#!/usr/bin/env node
// refresh-live-source-snapshot.mjs
//
// Captures current full-source PornsOK GET pages for byte-parity evidence.
// By default this refreshes the canonical source directory embedded by
// src/handlers/replay.rs. If the homepage route is included, it also refreshes
// the full homepage inventory JSON used by src/handlers/home.rs.
//
// This script intentionally does not refresh update_pornstars/update_channels
// AJAX bodies: live returns independent random six-card samples, so those are
// not stable byte targets.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  countClassCombo,
  countThumbClass,
  extractBlock,
  extractCanonical,
  extractDescription,
  extractH1s,
  extractTitle,
  normalizeDynamicBytes,
  sha256,
} from './lib/html-analysis.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, '..', '..');

const LIVE_BASE = (process.env.SOK_LIVE_BASE_URL || 'https://pornsok.com').replace(/\/+$/, '');
const TIMEOUT_MS = Number(process.env.SOK_PARITY_TIMEOUT_MS || 30000);
const REPORT_ONLY = process.env.SOK_PARITY_REPORT_ONLY === '1';
const ACCEPT_CONTENT_DRIFT = process.env.SOK_ACCEPT_CONTENT_DRIFT === '1';
const REFRESH_INVENTORY = process.env.SOK_REFRESH_INVENTORY !== '0';

const CANONICAL_SOURCE_DIR = path.join(__dirname, 'live-source-2026-06-27');
const OUT_DIR = path.resolve(process.env.SOK_REFRESH_SOURCE_DIR || CANONICAL_SOURCE_DIR);
const HOME_INVENTORY_PATH = path.resolve(
  process.env.SOK_HOME_INVENTORY_PATH ||
    path.join(__dirname, 'live-inventory-2026-06-26', 'home__desktop.json'),
);
const PREVIOUS_SOURCE_DIR = process.env.SOK_PREVIOUS_SOURCE_DIR
  ? path.resolve(process.env.SOK_PREVIOUS_SOURCE_DIR)
  : fs.existsSync(path.join(OUT_DIR, 'manifest.json'))
    ? OUT_DIR
    : latestSourceDir();

const UA =
  'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 ' +
  '(KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36 SOKSourceRefresh/1.0';

const DEFAULT_ROUTES = [
  { label: 'home', route: '/', file: 'home.html' },
  { label: 'categories', route: '/categories', file: 'categories.html' },
  { label: 'pornstars', route: '/pornstars', file: 'pornstars.html' },
  { label: 'channels', route: '/channels', file: 'channels.html' },
  { label: 'privacy', route: '/page/privacy.html', file: 'privacy.html' },
];

const MIN_EXPECTED_COUNTS = new Map([
  ['home', { thumbVidCount: 50, thumbCatCount: 10 }],
  ['categories', { thumbCatCount: 100 }],
  ['pornstars', { thumbCatCount: 50 }],
  ['channels', { thumbCatCount: 50 }],
]);

function latestSourceDir() {
  const dirs = fs
    .readdirSync(__dirname, { withFileTypes: true })
    .filter((entry) => entry.isDirectory() && /^live-source-\d{4}-\d{2}-\d{2}$/.test(entry.name))
    .map((entry) => path.join(__dirname, entry.name))
    .sort();
  return dirs.at(-1) ?? null;
}

function routeLabelFromPath(route) {
  if (route === '/') return 'home';
  return route.replace(/^\/+/, '').replace(/\.html$/i, '').replace(/[^a-z0-9]+/gi, '_') || 'route';
}

function loadRoutePlan() {
  if (process.env.SOK_CAPTURE_ROUTES) {
    return process.env.SOK_CAPTURE_ROUTES.split(',')
      .map((raw) => raw.trim())
      .filter(Boolean)
      .map((raw) => {
        const [maybeLabel, maybeRoute] = raw.includes('=') ? raw.split('=', 2) : [null, raw];
        const route = maybeRoute.startsWith('/') ? maybeRoute : `/${maybeRoute}`;
        const label = maybeLabel || routeLabelFromPath(route);
        return { label, route, file: `${label}.html` };
      });
  }

  const manifestPath = PREVIOUS_SOURCE_DIR ? path.join(PREVIOUS_SOURCE_DIR, 'manifest.json') : null;
  if (!manifestPath || !fs.existsSync(manifestPath)) return DEFAULT_ROUTES;

  const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
  return manifest.routes.map(({ label, route, file }) => ({ label, route, file }));
}

async function fetchRoute(route) {
  const ctl = new AbortController();
  const timer = setTimeout(() => ctl.abort(), TIMEOUT_MS);
  try {
    const resp = await fetch(LIVE_BASE + route.route, {
      headers: { 'user-agent': UA, accept: '*/*' },
      redirect: 'follow',
      signal: ctl.signal,
    });
    const bytes = Buffer.from(await resp.arrayBuffer());
    return {
      status: resp.status,
      content_type: resp.headers.get('content-type'),
      final_url: resp.url,
      redirected: resp.redirected,
      bytes,
    };
  } finally {
    clearTimeout(timer);
  }
}

function analyzeHtml(bytes) {
  const html = bytes.toString('utf8');
  const normalized = normalizeDynamicBytes(bytes);
  return {
    byte_length: bytes.length,
    sha256: sha256(bytes),
    normalized_byte_length: normalized.length,
    normalized_sha256: sha256(normalized),
    title: extractTitle(html),
    canonical: extractCanonical(html),
    h1: extractH1s(html),
    thumbVidCount: countThumbClass(html, 'vid'),
    thumbCatCount: countThumbClass(html, 'cat'),
    tagHoverCount: (html.match(/\bonMouseMove=/g) || []).length,
  };
}

function validateLivePage(route, result, analysis) {
  if (result.status !== 200) {
    throw new Error(`${route.label} ${route.route} returned HTTP ${result.status}`);
  }
  if (!/^text\/html\b/i.test(result.content_type || '')) {
    throw new Error(`${route.label} ${route.route} returned non-HTML content-type ${result.content_type}`);
  }
  if (!analysis.title || /just a moment|attention required|cloudflare/i.test(analysis.title)) {
    throw new Error(`${route.label} ${route.route} does not look like a real PornsOK page title`);
  }
  if (!analysis.canonical?.startsWith(`${LIVE_BASE}/`)) {
    throw new Error(`${route.label} ${route.route} has unexpected canonical ${analysis.canonical}`);
  }
  const minimums = MIN_EXPECTED_COUNTS.get(route.label);
  if (minimums?.thumbVidCount && analysis.thumbVidCount < minimums.thumbVidCount) {
    throw new Error(`${route.label} ${route.route} has only ${analysis.thumbVidCount} video thumbs`);
  }
  if (minimums?.thumbCatCount && analysis.thumbCatCount < minimums.thumbCatCount) {
    throw new Error(`${route.label} ${route.route} has only ${analysis.thumbCatCount} category thumbs`);
  }
}

function previousRouteByLabel() {
  const manifestPath = PREVIOUS_SOURCE_DIR ? path.join(PREVIOUS_SOURCE_DIR, 'manifest.json') : null;
  if (!manifestPath || !fs.existsSync(manifestPath)) return new Map();

  const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
  return new Map(
    manifest.routes.map((route) => {
      const filePath = path.join(PREVIOUS_SOURCE_DIR, route.file);
      const bytes = fs.existsSync(filePath) ? fs.readFileSync(filePath) : null;
      const normalized = bytes ? normalizeDynamicBytes(bytes) : null;
      return [
        route.label,
        {
          ...route,
          sha256: bytes ? sha256(bytes) : route.sha256,
          byte_length: bytes ? bytes.length : route.byte_length,
          normalized_sha256: normalized ? sha256(normalized) : route.normalized_sha256,
          normalized_byte_length: normalized ? normalized.length : route.normalized_byte_length,
        },
      ];
    }),
  );
}

function classifyDrift(previous, current) {
  if (!previous) return 'new_route';
  if (previous.sha256 === current.sha256) return 'unchanged_exact';
  if (previous.normalized_sha256 === current.normalized_sha256) return 'cloudflare_dynamic_only';
  return 'content_drift';
}

function assertEmbeddedTargets() {
  const replayPath = path.join(REPO_ROOT, 'src', 'handlers', 'replay.rs');
  const homePath = path.join(REPO_ROOT, 'src', 'handlers', 'home.rs');
  const replay = fs.readFileSync(replayPath, 'utf8');
  const home = fs.readFileSync(homePath, 'utf8');
  const sourceNeedle = `docs/raw/${path.basename(OUT_DIR)}/`;
  const inventoryRelative = path
    .relative(REPO_ROOT, HOME_INVENTORY_PATH)
    .split(path.sep)
    .join('/');

  if (!replay.includes(sourceNeedle)) {
    throw new Error(`${replayPath} does not embed ${sourceNeedle}; refusing silent stale refresh`);
  }
  if (REFRESH_INVENTORY && !home.includes(inventoryRelative)) {
    throw new Error(`${homePath} does not embed ${inventoryRelative}; refusing silent stale inventory refresh`);
  }
}

function htmlAttr(html, tagName, attrName) {
  const tag = String(html).match(new RegExp(`<${tagName}\\b[^>]*>`, 'i'))?.[0];
  if (!tag) return null;
  return tag.match(new RegExp(`\\b${attrName}\\s*=\\s*["']([^"']*)["']`, 'i'))?.[1] ?? null;
}

function buildHomeInventory(homeBytes, fetchedAt) {
  const html = homeBytes.toString('utf8');
  const main = extractBlock(html, 'main');
  if (!main) throw new Error('home source does not contain <main>');
  if (main.endsWith('…') || main.length <= 4000) {
    throw new Error('home inventory main would be truncated or unexpectedly small');
  }

  const existing = fs.existsSync(HOME_INVENTORY_PATH)
    ? JSON.parse(fs.readFileSync(HOME_INVENTORY_PATH, 'utf8'))
    : {};
  const data = {
    ...existing,
    url: `${LIVE_BASE}/`,
    fetched_at: fetchedAt,
    viewport: existing.viewport || 'desktop',
    http_status: 200,
    title: extractTitle(html) ?? existing.title ?? null,
    canonical: extractCanonical(html) ?? existing.canonical ?? null,
    lang: htmlAttr(html, 'html', 'lang') ?? existing.lang ?? null,
    theme: htmlAttr(html, 'html', 'data-theme') ?? existing.theme ?? null,
    description: extractDescription(html) ?? existing.description ?? null,
    h1: extractH1s(html),
    header: extractBlock(html, 'header') ?? existing.header ?? null,
    main,
    footer: extractBlock(html, 'footer') ?? existing.footer ?? null,
    bodyClasses: htmlAttr(html, 'body', 'class') ?? existing.bodyClasses ?? '',
    viewportWidth: existing.viewportWidth || 1440,
    fullMainCapture: true,
    sourceBytes: homeBytes.length,
    mainBytes: Buffer.byteLength(main),
    thumbVidCount: countThumbClass(html, 'vid'),
    thumbCatCount: countThumbClass(html, 'cat'),
    tagHoverCount: countClassCombo(main, ['tag', 'hover']) || (main.match(/\bonMouseMove=/g) || []).length,
    hasFilterSection: html.includes('filter-section'),
    hasPageNav: html.includes('page_nav'),
    hasPlayer: html.includes('player_container'),
    hasSearchBoxPage: html.includes('search-page-input'),
    hasSearchGenresInput: html.includes('search-genres-input'),
  };

  if (data.sourceBytes !== homeBytes.length) {
    throw new Error('home inventory sourceBytes mismatch');
  }
  if (data.mainBytes !== Buffer.byteLength(data.main)) {
    throw new Error('home inventory mainBytes mismatch');
  }
  if (data.thumbVidCount < 50 || data.thumbCatCount < 10) {
    throw new Error(
      `home inventory counts look wrong: ${data.thumbVidCount} video thumbs, ${data.thumbCatCount} category thumbs`,
    );
  }

  return data;
}

function writeTempSourceDir(tempDir, manifest, routeBytes) {
  fs.mkdirSync(tempDir, { recursive: true });
  for (const route of manifest.routes) {
    fs.writeFileSync(path.join(tempDir, route.file), routeBytes.get(route.label));
  }
  fs.writeFileSync(path.join(tempDir, 'manifest.json'), JSON.stringify(manifest, null, 2) + '\n', 'utf8');

  for (const route of manifest.routes) {
    const diskBytes = fs.readFileSync(path.join(tempDir, route.file));
    if (diskBytes.length !== route.byte_length || sha256(diskBytes) !== route.sha256) {
      throw new Error(`${route.file} changed while writing temp source dir`);
    }
  }
}

function replaceDirectory(tempDir, targetDir) {
  const parent = path.dirname(targetDir);
  fs.mkdirSync(parent, { recursive: true });
  const backupDir = path.join(parent, `.${path.basename(targetDir)}.bak-${process.pid}-${Date.now()}`);

  if (fs.existsSync(targetDir)) fs.renameSync(targetDir, backupDir);
  try {
    fs.renameSync(tempDir, targetDir);
    if (fs.existsSync(backupDir)) fs.rmSync(backupDir, { recursive: true, force: true });
  } catch (error) {
    if (!fs.existsSync(targetDir) && fs.existsSync(backupDir)) fs.renameSync(backupDir, targetDir);
    throw error;
  }
}

function writeFileAtomically(filePath, text) {
  const tempPath = path.join(
    path.dirname(filePath),
    `.${path.basename(filePath)}.tmp-${process.pid}-${Date.now()}`,
  );
  fs.writeFileSync(tempPath, text, 'utf8');
  fs.renameSync(tempPath, filePath);
}

async function main() {
  if (OUT_DIR === CANONICAL_SOURCE_DIR) assertEmbeddedTargets();
  if (
    OUT_DIR !== CANONICAL_SOURCE_DIR &&
    REFRESH_INVENTORY &&
    !process.env.SOK_HOME_INVENTORY_PATH &&
    !REPORT_ONLY
  ) {
    throw new Error(
      'non-canonical source refresh would overwrite the canonical home inventory; set SOK_HOME_INVENTORY_PATH, SOK_REFRESH_INVENTORY=0, or SOK_PARITY_REPORT_ONLY=1',
    );
  }

  const fetchedAt = new Date().toISOString();
  const routes = loadRoutePlan();
  const previousByLabel = previousRouteByLabel();
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'sok-live-source-'));
  const routeBytes = new Map();
  const manifestRoutes = [];
  const driftRoutes = [];

  console.error(`live:      ${LIVE_BASE}`);
  console.error(`snapshot:  ${OUT_DIR}`);
  console.error(`previous:  ${PREVIOUS_SOURCE_DIR ?? '(none)'}`);
  console.error(`home inv:  ${REFRESH_INVENTORY ? HOME_INVENTORY_PATH : '(disabled)'}`);
  console.error(`mode:      ${REPORT_ONLY ? 'report-only' : 'write'}${ACCEPT_CONTENT_DRIFT ? ' accept-content-drift' : ''}`);

  let homeBytes = null;
  for (const route of routes) {
    const result = await fetchRoute(route);
    const analysis = analyzeHtml(result.bytes);
    validateLivePage(route, result, analysis);

    const entry = {
      label: route.label,
      route: route.route,
      status: result.status,
      content_type: result.content_type,
      file: route.file,
      byte_length: analysis.byte_length,
      sha256: analysis.sha256,
      normalized_byte_length: analysis.normalized_byte_length,
      normalized_sha256: analysis.normalized_sha256,
      fetched_at: fetchedAt,
      final_url: result.final_url,
      redirected: result.redirected,
      title: analysis.title,
      canonical: analysis.canonical,
      h1: analysis.h1,
      thumb_vid_count: analysis.thumbVidCount,
      thumb_cat_count: analysis.thumbCatCount,
      tag_hover_count: analysis.tagHoverCount,
    };
    const previous = previousByLabel.get(route.label);
    const drift = classifyDrift(previous, entry);
    manifestRoutes.push(entry);
    routeBytes.set(route.label, result.bytes);
    driftRoutes.push({
      label: route.label,
      route: route.route,
      status: drift,
      previous_sha256: previous?.sha256 ?? null,
      current_sha256: entry.sha256,
      previous_normalized_sha256: previous?.normalized_sha256 ?? null,
      current_normalized_sha256: entry.normalized_sha256,
    });
    if (route.label === 'home') homeBytes = result.bytes;
    console.error(
      `${route.label.padEnd(12)} ${String(entry.byte_length).padStart(8)} ${drift.padEnd(24)} ${entry.sha256}`,
    );
  }

  if (!homeBytes && REFRESH_INVENTORY) {
    throw new Error('route plan did not include home, but inventory refresh is enabled');
  }

  const contentDriftRoutes = driftRoutes.filter((route) => route.status === 'content_drift');
  const driftReport = {
    generated_at: fetchedAt,
    tool: 'refresh-live-source-snapshot.mjs',
    live_base_url: LIVE_BASE,
    previous_source_dir: PREVIOUS_SOURCE_DIR,
    target_source_dir: OUT_DIR,
    report_only: REPORT_ONLY,
    accept_content_drift: ACCEPT_CONTENT_DRIFT,
    summary: {
      total: driftRoutes.length,
      content_drift: contentDriftRoutes.length,
      status_counts: driftRoutes.reduce((counts, route) => {
        counts[route.status] = (counts[route.status] || 0) + 1;
        return counts;
      }, {}),
    },
    routes: driftRoutes,
  };

  const homeInventory = REFRESH_INVENTORY ? buildHomeInventory(homeBytes, fetchedAt) : null;
  const manifest = {
    generated_at: fetchedAt,
    live_base_url: LIVE_BASE,
    user_agent: UA,
    source_dir: path.basename(OUT_DIR),
    routes: manifestRoutes,
    refreshed_inventory: homeInventory
      ? {
          home: {
            path: HOME_INVENTORY_PATH,
            sourceBytes: homeInventory.sourceBytes,
            mainBytes: homeInventory.mainBytes,
            thumbVidCount: homeInventory.thumbVidCount,
            thumbCatCount: homeInventory.thumbCatCount,
            tagHoverCount: homeInventory.tagHoverCount,
          },
        }
      : null,
  };

  writeTempSourceDir(tempDir, manifest, routeBytes);
  fs.writeFileSync(path.join(tempDir, 'drift-report.json'), JSON.stringify(driftReport, null, 2) + '\n', 'utf8');

  if (contentDriftRoutes.length > 0 && !REPORT_ONLY && !ACCEPT_CONTENT_DRIFT) {
    throw new Error(
      `normalized content drift detected for ${contentDriftRoutes
        .map((route) => route.label)
        .join(', ')}; rerun with SOK_ACCEPT_CONTENT_DRIFT=1 to update artifacts intentionally`,
    );
  }

  if (REPORT_ONLY) {
    const reportPath = process.env.SOK_REFRESH_REPORT_OUTPUT
      ? path.resolve(process.env.SOK_REFRESH_REPORT_OUTPUT)
      : path.join(__dirname, `live-source-refresh-drift-${fetchedAt.slice(0, 10)}.json`);
    fs.writeFileSync(reportPath, JSON.stringify(driftReport, null, 2) + '\n', 'utf8');
    fs.rmSync(tempDir, { recursive: true, force: true });
    console.error(`report:    ${reportPath}`);
    return;
  }

  replaceDirectory(tempDir, OUT_DIR);
  if (homeInventory) {
    writeFileAtomically(HOME_INVENTORY_PATH, JSON.stringify(homeInventory, null, 2) + '\n');
  }
  console.error(`manifest:  ${path.join(OUT_DIR, 'manifest.json')}`);
  console.error(`drift:     ${path.join(OUT_DIR, 'drift-report.json')}`);
}

main().catch((error) => {
  console.error('fatal:', error.message || error);
  process.exitCode = 1;
});
