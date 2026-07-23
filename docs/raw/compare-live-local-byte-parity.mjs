#!/usr/bin/env node
// compare-live-local-byte-parity.mjs
//
// Deterministic live-vs-local byte parity verifier for the PornsOK replica.
// Probes representative GET routes and AJAX endpoints on a local Actix server
// and the live site, then reports exact byte/sha256 parity per probe.
//
// HTML extraction (title/h1/canonical/thumb counts) is regex-based on the raw
// response bytes -- intentionally dependency-free, no DOM parser. Counts and
// metadata are reported alongside the strict byte/hash pass/fail.
//
// Env:
//   SOK_BASE_URL              local base (default http://127.0.0.1:8080)
//   SOK_LIVE_BASE_URL         live base (default https://pornsok.com)
//   SOK_PARITY_EXTRA_ROUTES   comma-separated extra GET paths (video/sample routes)
//   SOK_PARITY_SEARCH_TERM    deterministic term for AJAX search probes (default milf)
//   SOK_PARITY_SKIP_SEARCH    "1" skips /ajax/search_help + /ajax/search_cats_tags_queries
//   SOK_PARITY_OUTPUT         override output JSON path
//   SOK_PARITY_REPORT_ONLY    "1" report only, never exit non-zero
//   SOK_PARITY_TIMEOUT_MS     per-request timeout (default 30000)
//   SOK_PARITY_SOURCE         "live" (default) or "artifact"
//   SOK_CAPTURED_SOURCE_DIR   captured source dir for artifact mode
//   SOK_PARITY_ACCEPT_NORMALIZED "1" accepts normalized parity for exit status
//
// Exits non-zero when any probe lacks exact byte/hash parity, unless
// SOK_PARITY_REPORT_ONLY=1. See docs/raw/README.md.

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  countThumbClass,
  extractCanonical,
  extractH1s,
  extractTitle,
  normalizeDynamicBytes,
  sha256,
} from './lib/html-analysis.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const LOCAL_BASE = (process.env.SOK_BASE_URL || 'http://127.0.0.1:8080').replace(/\/+$/, '');
const LIVE_BASE = (process.env.SOK_LIVE_BASE_URL || 'https://pornsok.com').replace(/\/+$/, '');
const SOURCE_MODE = (process.env.SOK_PARITY_SOURCE || 'live').trim().toLowerCase();
const CAPTURE_DIR = path.resolve(
  process.env.SOK_CAPTURED_SOURCE_DIR || path.join(__dirname, 'live-source-2026-06-27'),
);
const SEARCH_TERM = process.env.SOK_PARITY_SEARCH_TERM || 'milf';
const SKIP_SEARCH = process.env.SOK_PARITY_SKIP_SEARCH === '1';
const REPORT_ONLY = process.env.SOK_PARITY_REPORT_ONLY === '1';
const TIMEOUT_MS = Number(process.env.SOK_PARITY_TIMEOUT_MS || 30000);
const ARTIFACT_SOURCE = SOURCE_MODE === 'artifact';
const ACCEPT_NORMALIZED = process.env.SOK_PARITY_ACCEPT_NORMALIZED === '1';

if (!['live', 'artifact'].includes(SOURCE_MODE)) {
  console.error(`fatal: unsupported SOK_PARITY_SOURCE=${SOURCE_MODE}; expected "live" or "artifact"`);
  process.exit(2);
}

// Realistic desktop UA so the live edge does not rewrite/block default fetch clients.
const UA =
  'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 ' +
  '(KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36 SOKParityVerifier/1.0';

// --- Probe definitions -----------------------------------------------------

const GET_ROUTES = [
  { label: 'home', path: '/' },
  { label: 'categories', path: '/categories' },
  { label: 'pornstars', path: '/pornstars' },
  { label: 'channels', path: '/channels' },
  { label: 'privacy', path: '/page/privacy.html' },
];

const EXTRA = (process.env.SOK_PARITY_EXTRA_ROUTES || '')
  .split(',')
  .map((s) => s.trim())
  .filter(Boolean)
  .map((p, i) => ({ label: `extra-${i + 1}`, path: p.startsWith('/') ? p : `/${p}` }));

const SEARCH_BODY = `text=${encodeURIComponent(SEARCH_TERM)}`;

// Payloads mirror docs/raw/*.meta.txt: update_* take an empty POST body;
// search_* take a urlencoded text= field. All deterministic.
const AJAX_PROBES = [
  { label: 'update_pornstars', path: '/ajax/update_pornstars', method: 'POST', body: '' },
  { label: 'update_channels', path: '/ajax/update_channels', method: 'POST', body: '' },
];
if (!SKIP_SEARCH) {
  AJAX_PROBES.push(
    { label: 'search_help', path: '/ajax/search_help', method: 'POST', body: SEARCH_BODY },
    {
      label: 'search_cats_tags_queries',
      path: '/ajax/search_cats_tags_queries',
      method: 'POST',
      body: SEARCH_BODY,
    },
  );
}

// --- HTTP ------------------------------------------------------------------

async function fetchProbe(base, probe) {
  const url = base + probe.path;
  const ctl = new AbortController();
  const timer = setTimeout(() => ctl.abort(), TIMEOUT_MS);
  const headers = { 'user-agent': UA, accept: '*/*' };
  if (probe.method === 'POST') {
    headers['content-type'] = 'application/x-www-form-urlencoded';
  }
  try {
    const resp = await fetch(url, {
      method: probe.method || 'GET',
      body: probe.method === 'POST' ? probe.body : undefined,
      headers,
      signal: ctl.signal,
      redirect: 'follow',
    });
    const buf = Buffer.from(await resp.arrayBuffer());
    return {
      ok: true,
      status: resp.status,
      status_text: resp.statusText,
      content_type: resp.headers.get('content-type'),
      final_url: resp.url,
      redirected: resp.redirected,
      bytes: buf,
    };
  } catch (e) {
    return {
      ok: false,
      error: e?.name === 'AbortError' ? `timeout after ${TIMEOUT_MS}ms` : String(e?.message ?? e),
    };
  } finally {
    clearTimeout(timer);
  }
}

function loadCapturedSourceManifest() {
  const manifestPath = path.join(CAPTURE_DIR, 'manifest.json');
  const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
  const routesByPath = new Map(manifest.routes.map((route) => [route.route, route]));
  const routesByLabel = new Map(manifest.routes.map((route) => [route.label, route]));
  return { manifest, manifestPath, routesByPath, routesByLabel };
}

function fetchCapturedSourceProbe(probe, capturedSource) {
  if ((probe.method || 'GET') !== 'GET') {
    return {
      ok: false,
      error: 'artifact source mode only supports captured GET route source',
    };
  }

  const route = capturedSource.routesByPath.get(probe.path) || capturedSource.routesByLabel.get(probe.label);
  if (!route) {
    return {
      ok: false,
      error: `no captured source route for ${probe.label} ${probe.path}`,
    };
  }

  const capturePath = path.join(CAPTURE_DIR, route.file);
  return {
    ok: true,
    status: route.status ?? 200,
    status_text: 'Captured',
    content_type: route.content_type ?? 'text/html; charset=UTF-8',
    final_url: capturePath,
    redirected: false,
    bytes: fs.readFileSync(capturePath),
  };
}

// --- Heuristic HTML/JSON extractors (no DOM dependency) --------------------

function analyzeSide(result) {
  if (!result.ok) {
    return {
      status: null,
      content_type: null,
      byte_length: null,
      sha256: null,
      normalized_byte_length: null,
      normalized_sha256: null,
      final_url: null,
      redirected: null,
      title: null,
      h1: [],
      canonical: null,
      thumb_vid_count: null,
      thumb_cat_count: null,
      error: result.error,
    };
  }
  const bytes = result.bytes;
  const normalized = normalizeDynamicBytes(bytes);
  const html = bytes.toString('utf8');
  // Treat as HTML for metadata extraction only when served as text/html or the
  // body opens with a tag; JSON endpoints report null metadata + zero counts.
  const isHtml = /text\/html/i.test(result.content_type || '') || /^\s*</.test(html);
  return {
    status: result.status,
    content_type: result.content_type,
    byte_length: bytes.length,
    sha256: sha256(bytes),
    normalized_byte_length: normalized.length,
    normalized_sha256: sha256(normalized),
    final_url: result.final_url,
    redirected: result.redirected,
    title: isHtml ? extractTitle(html) : null,
    h1: isHtml ? extractH1s(html) : [],
    canonical: isHtml ? extractCanonical(html) : null,
    thumb_vid_count: countThumbClass(html, 'vid'),
    thumb_cat_count: countThumbClass(html, 'cat'),
    error: null,
  };
}

// --- Exact-parity audit classification -------------------------------------

function dynamicByteDelta(side) {
  if (side.byte_length == null || side.normalized_byte_length == null) return null;
  return side.byte_length - side.normalized_byte_length;
}

function contentTypeFamily(contentType) {
  return String(contentType || '').split(';', 1)[0].trim().toLowerCase();
}

function isRandomUpdateWidget(probe) {
  return probe.path === '/ajax/update_pornstars' || probe.path === '/ajax/update_channels';
}

function sameResponseShape(local, live) {
  return (
    local.status === live.status &&
    contentTypeFamily(local.content_type) === contentTypeFamily(live.content_type) &&
    local.title === live.title &&
    JSON.stringify(local.h1) === JSON.stringify(live.h1) &&
    local.canonical === live.canonical &&
    local.thumb_vid_count === live.thumb_vid_count &&
    local.thumb_cat_count === live.thumb_cat_count
  );
}

function auditProbe(probe, local, live, exact_parity, normalized_parity) {
  const local_dynamic_byte_delta = dynamicByteDelta(local);
  const live_dynamic_byte_delta = dynamicByteDelta(live);
  const response_shape_match = sameResponseShape(local, live);
  const blockers = [];
  let status = 'content_mismatch';
  let interpretation = 'Response bytes and normalized bytes differ; inspect route content/template drift.';

  if (local.error || live.error) {
    status = 'transport_error';
    interpretation = 'One side failed to fetch; exact parity is not measurable.';
  } else if (exact_parity) {
    status = 'exact';
    interpretation = 'Raw byte length and SHA-256 match.';
  } else if (normalized_parity) {
    status = 'normalized_dynamic_only';
    blockers.push('cloudflare_edge_dynamic_bytes');
    interpretation =
      'Raw bytes differ, but configured Cloudflare/dynamic-byte normalization produces identical bytes and SHA-256.';
    if ((local_dynamic_byte_delta ?? 0) !== 0 || (live_dynamic_byte_delta ?? 0) !== 0) {
      blockers.push('cloudflare_tokenized_script_attrs_or_paths');
    }
    if (probe.path === '/page/privacy.html') {
      blockers.push('cloudflare_email_obfuscation_token');
    }
  } else if (
    isRandomUpdateWidget(probe) &&
    local.status === 200 &&
    live.status === 200 &&
    local.thumb_cat_count === 6 &&
    live.thumb_cat_count === 6
  ) {
    status = 'independent_random_sample';
    blockers.push('ajax_random_six_card_sample');
    interpretation =
      'Both sides returned live-shaped six-card HTML fragments, but exact bytes depend on independent random samples.';
  } else if (response_shape_match) {
    status = 'shape_match_content_mismatch';
    interpretation =
      'High-level metadata/counts match, but normalized bytes still differ; inspect deterministic markup/content.';
  }

  return {
    status,
    blockers,
    interpretation,
    local_dynamic_byte_delta,
    live_dynamic_byte_delta,
    response_shape_match,
  };
}

// --- Run -------------------------------------------------------------------

async function runProbe(probe, capturedSource) {
  // Fetch local and source concurrently to minimize dynamic-content time skew in live mode.
  const sourceFetch =
    ARTIFACT_SOURCE && (probe.method || 'GET') === 'GET'
      ? Promise.resolve(fetchCapturedSourceProbe(probe, capturedSource))
      : fetchProbe(LIVE_BASE, probe);
  const [local, source] = await Promise.all([fetchProbe(LOCAL_BASE, probe), sourceFetch]);
  const localSide = analyzeSide(local);
  const sourceSide = analyzeSide(source);
  const bothOk = localSide.error === null && sourceSide.error === null;
  const bytes_match = bothOk && localSide.byte_length === sourceSide.byte_length;
  const hash_match = bothOk && localSide.sha256 === sourceSide.sha256;
  const normalized_bytes_match =
    bothOk && localSide.normalized_byte_length === sourceSide.normalized_byte_length;
  const normalized_hash_match =
    bothOk && localSide.normalized_sha256 === sourceSide.normalized_sha256;
  const exact_parity = bytes_match && hash_match;
  const normalized_parity = normalized_bytes_match && normalized_hash_match;
  const exact_audit = auditProbe(probe, localSide, sourceSide, exact_parity, normalized_parity);
  return {
    kind: probe.method || 'GET',
    label: probe.label,
    path: probe.path,
    local: localSide,
    live: sourceSide,
    source: sourceSide,
    bytes_match,
    hash_match,
    normalized_bytes_match,
    normalized_hash_match,
    normalized_parity,
    exact_parity,
    exact_audit,
  };
}

function pad(s, n) {
  return String(s).padEnd(n);
}

function fmtRow(probe) {
  const l = probe.local;
  const v = probe.source || probe.live;
  const mark = probe.exact_parity
    ? 'PASS'
    : probe.normalized_parity
      ? 'NORM'
      : probe.exact_audit?.status === 'independent_random_sample'
        ? 'RAND'
        : 'FAIL';
  return [
    pad(probe.label, 26),
    pad(probe.path, 34),
    pad(`${l.status ?? 'ERR'}->${v.status ?? 'ERR'}`, 11),
    pad(`${l.byte_length ?? '-'}|${v.byte_length ?? '-'}`, 17),
    mark,
  ].join(' ');
}

async function main() {
  const capturedSource = ARTIFACT_SOURCE ? loadCapturedSourceManifest() : null;
  const routes = [...GET_ROUTES, ...EXTRA];
  const ajaxProbes = ARTIFACT_SOURCE ? [] : AJAX_PROBES;
  const probes = [...routes.map((r) => ({ ...r, method: 'GET' })), ...ajaxProbes];
  const sourceLabel = ARTIFACT_SOURCE ? 'artifact' : 'live';
  const sourceBase = ARTIFACT_SOURCE ? capturedSource.manifestPath : LIVE_BASE;

  console.error(`local: ${LOCAL_BASE}`);
  console.error(`${sourceLabel}: ${sourceBase}`);
  console.error(
    `probes: ${probes.length} (${routes.length} GET, ${ajaxProbes.length} AJAX)  source=${SOURCE_MODE}  report_only=${REPORT_ONLY}`,
  );
  console.error(pad('label', 26) + pad('path', 34) + pad(`local->${sourceLabel}`, 11) + pad('bytes L|S', 17) + 'result');
  console.error('-'.repeat(96));

  const results = [];
  for (const probe of probes) {
    // Sequential to keep live request volume low and output streaming.
    const r = await runProbe(probe, capturedSource);
    results.push(r);
    console.error(fmtRow(r));
  }

  const passed = results.filter((r) => r.exact_parity).length;
  const normalized_passed = results.filter((r) => r.normalized_parity).length;
  const accepted = results.filter(
    (r) => r.exact_parity || (ACCEPT_NORMALIZED && r.normalized_parity),
  ).length;
  const failed = results.length - passed;
  const acceptance_failed = results.length - accepted;
  const audit_status_counts = results.reduce((counts, r) => {
    const status = r.exact_audit?.status || 'unknown';
    counts[status] = (counts[status] || 0) + 1;
    return counts;
  }, {});
  const summary = {
    total: results.length,
    passed,
    failed,
    normalized_passed,
    accepted,
    acceptance_failed,
    acceptance_mode: ACCEPT_NORMALIZED ? 'exact_or_normalized' : 'exact',
    report_only: REPORT_ONLY,
    audit_status_counts,
  };

  const ts = new Date().toISOString();
  const dateTag = ts.slice(0, 10); // UTC date, consistent with other dated docs/raw outputs
  const defaultOut = path.resolve(__dirname, `local-byte-parity-${dateTag}.json`);
  const defaultArtifactOut = path.resolve(__dirname, `local-artifact-byte-parity-${dateTag}.json`);
  const outPath = process.env.SOK_PARITY_OUTPUT
    ? path.resolve(process.env.SOK_PARITY_OUTPUT)
    : ARTIFACT_SOURCE
      ? defaultArtifactOut
      : defaultOut;

  const report = {
    generated_at: ts,
    tool: 'compare-live-local-byte-parity.mjs',
    source_mode: SOURCE_MODE,
    source_base: sourceBase,
    local_base_url: LOCAL_BASE,
    live_base_url: LIVE_BASE,
    captured_source_dir: ARTIFACT_SOURCE ? CAPTURE_DIR : null,
    captured_source_manifest: ARTIFACT_SOURCE ? capturedSource.manifest : null,
    config: {
      search_term: SEARCH_TERM,
      skip_search: SKIP_SEARCH,
      report_only: REPORT_ONLY,
      timeout_ms: TIMEOUT_MS,
      extra_routes: EXTRA.map((e) => e.path),
      source_mode: SOURCE_MODE,
      accept_normalized: ACCEPT_NORMALIZED,
    },
    summary,
    probes: results,
  };

  fs.mkdirSync(path.dirname(outPath), { recursive: true });
  fs.writeFileSync(outPath, JSON.stringify(report, null, 2) + '\n', 'utf8');

  console.error('-'.repeat(96));
  console.error(
    `summary: ${passed}/${results.length} exact parity; ` +
      `${accepted}/${results.length} accepted (${summary.acceptance_mode})  ` +
      `failed=${acceptance_failed}`,
  );
  console.error(`report:  ${outPath}`);

  if (!REPORT_ONLY && acceptance_failed > 0) {
    process.exitCode = 1;
  }
}

main().catch((e) => {
  console.error('fatal:', e);
  process.exitCode = 2;
});
