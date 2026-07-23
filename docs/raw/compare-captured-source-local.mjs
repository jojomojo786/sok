#!/usr/bin/env node
// compare-captured-source-local.mjs
//
// Byte-for-byte verifier for the diagnostic local source replay route.
// It compares diagnostic replay response bodies against frozen live source
// captures under docs/raw/live-source-YYYY-MM-DD. This intentionally does not
// fetch live and does not validate the production dynamic rendering path.

import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const LOCAL_BASE = (process.env.SOK_BASE_URL || 'http://127.0.0.1:8080').replace(/\/+$/, '');
const CAPTURE_DIR = path.resolve(
  process.env.SOK_CAPTURED_SOURCE_DIR || path.join(__dirname, 'live-source-2026-06-27'),
);
const REPLAY_ROUTE_PREFIX = (process.env.SOK_REPLAY_ROUTE_PREFIX || '/_diag/source-replay').replace(
  /\/+$/,
  '',
);
const REQUIRE_DYNAMIC = process.env.SOK_REQUIRE_DYNAMIC === '1';
const INCLUDE_AJAX = process.env.SOK_REPLAY_INCLUDE_AJAX !== '0';
const REPORT_ONLY = process.env.SOK_PARITY_REPORT_ONLY === '1';
const TIMEOUT_MS = Number(process.env.SOK_PARITY_TIMEOUT_MS || 30000);

const AJAX_CAPTURE_ROUTES = [
  {
    label: 'update_pornstars',
    route: '/ajax/update_pornstars',
    method: 'POST',
    body: '',
    diagnostic_route: `${REPLAY_ROUTE_PREFIX}/ajax/update_pornstars`,
    capture_file: path.join(__dirname, 'update_pornstars.body'),
  },
  {
    label: 'update_channels',
    route: '/ajax/update_channels',
    method: 'POST',
    body: '',
    diagnostic_route: `${REPLAY_ROUTE_PREFIX}/ajax/update_channels`,
    capture_file: path.join(__dirname, 'update_channels.body'),
  },
];

function sha256(bytes) {
  return crypto.createHash('sha256').update(bytes).digest('hex');
}

function loadManifest() {
  const manifestPath = path.join(CAPTURE_DIR, 'manifest.json');
  const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
  return { manifestPath, manifest };
}

function manifestRoutes(manifest) {
  const getRoutes = manifest.routes.map((route) => ({
    ...route,
    method: 'GET',
    body: null,
    capture_file: path.join(CAPTURE_DIR, route.file),
    diagnostic_route: replayRoute(route),
  }));
  return INCLUDE_AJAX ? [...getRoutes, ...AJAX_CAPTURE_ROUTES] : getRoutes;
}

async function fetchLocal(route, method = 'GET', body = null) {
  const ctl = new AbortController();
  const timer = setTimeout(() => ctl.abort(), TIMEOUT_MS);
  const headers = { accept: '*/*' };
  if (method === 'POST') {
    headers['content-type'] = 'application/x-www-form-urlencoded';
  }
  try {
    const resp = await fetch(LOCAL_BASE + route, {
      method,
      body: method === 'POST' ? (body ?? '') : undefined,
      headers,
      redirect: 'follow',
      signal: ctl.signal,
    });
    const bytes = Buffer.from(await resp.arrayBuffer());
    return {
      ok: true,
      status: resp.status,
      content_type: resp.headers.get('content-type'),
      final_url: resp.url,
      bytes,
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

function replayRoute(route) {
  return `${REPLAY_ROUTE_PREFIX}/${encodeURIComponent(route.label)}`;
}

async function runRoute(route) {
  const capturePath = route.capture_file;
  const captured = fs.readFileSync(capturePath);
  const diagnostic_route = route.diagnostic_route;
  const local = await fetchLocal(diagnostic_route);
  const production = REQUIRE_DYNAMIC
    ? await fetchLocal(route.route, route.method || 'GET', route.body ?? null)
    : null;
  const localBytes = local.ok ? local.bytes : Buffer.alloc(0);
  const productionBytes = production?.ok ? production.bytes : Buffer.alloc(0);
  const bytes_match = local.ok && captured.length === localBytes.length;
  const hash_match = local.ok && sha256(captured) === sha256(localBytes);
  const production_distinct_from_replay =
    production == null
      ? null
      : production.ok &&
        (productionBytes.length !== localBytes.length || sha256(productionBytes) !== sha256(localBytes));
  return {
    label: route.label,
    route: diagnostic_route,
    production_route: route.route,
    capture_file: capturePath,
    captured: {
      byte_length: captured.length,
      sha256: sha256(captured),
    },
    local: local.ok
      ? {
          status: local.status,
          content_type: local.content_type,
          final_url: local.final_url,
          byte_length: localBytes.length,
          sha256: sha256(localBytes),
          error: null,
        }
      : {
          status: null,
          content_type: null,
          final_url: null,
          byte_length: null,
          sha256: null,
          error: local.error,
        },
    production: production
      ? production.ok
        ? {
            status: production.status,
            content_type: production.content_type,
            final_url: production.final_url,
            byte_length: productionBytes.length,
            sha256: sha256(productionBytes),
            distinct_from_replay: production_distinct_from_replay,
            error: null,
          }
        : {
            status: null,
            content_type: null,
            final_url: null,
            byte_length: null,
            sha256: null,
            distinct_from_replay: false,
            error: production.error,
          }
      : null,
    bytes_match,
    hash_match,
    exact_parity: bytes_match && hash_match,
    accepted: bytes_match && hash_match && (!REQUIRE_DYNAMIC || production_distinct_from_replay),
  };
}

function pad(s, n) {
  return String(s).padEnd(n);
}

function fmtRow(result) {
  return [
    pad(result.label, 16),
    pad(result.route, 34),
    pad(`${result.local.status ?? 'ERR'}`, 8),
    pad(`${result.local.byte_length ?? '-'}|${result.captured.byte_length}`, 18),
    result.accepted ? 'PASS' : result.exact_parity ? 'REPLAY_ONLY' : 'FAIL',
  ].join(' ');
}

async function main() {
  const { manifestPath, manifest } = loadManifest();
  console.error('mode:    frozen_source_replay');
  console.error(`local:   ${LOCAL_BASE}`);
  console.error(`replay:  ${REPLAY_ROUTE_PREFIX}`);
  console.error(`capture: ${manifestPath}`);
  console.error(`include_ajax=${INCLUDE_AJAX}`);
  console.error(`require_dynamic=${REQUIRE_DYNAMIC}`);
  console.error(pad('label', 16) + pad('route', 34) + pad('status', 8) + pad('bytes L|C', 18) + 'result');
  console.error('-'.repeat(86));

  const results = [];
  const routes = manifestRoutes(manifest);
  for (const route of routes) {
    const result = await runRoute(route);
    results.push(result);
    console.error(fmtRow(result));
  }

  const passed = results.filter((r) => r.exact_parity).length;
  const failed = results.length - passed;
  const accepted = results.filter((r) => r.accepted).length;
  const acceptance_failed = results.length - accepted;
  const report = {
    generated_at: new Date().toISOString(),
    tool: 'compare-captured-source-local.mjs',
    mode: 'frozen_source_replay',
    local_base_url: LOCAL_BASE,
    replay_route_prefix: REPLAY_ROUTE_PREFIX,
    capture_dir: CAPTURE_DIR,
    capture_manifest: manifest,
    include_ajax: INCLUDE_AJAX,
    summary: {
      total: results.length,
      passed,
      failed,
      accepted,
      acceptance_failed,
      require_dynamic: REQUIRE_DYNAMIC,
      report_only: REPORT_ONLY,
    },
    probes: results,
  };
  const outPath = process.env.SOK_PARITY_OUTPUT
    ? path.resolve(process.env.SOK_PARITY_OUTPUT)
    : path.join(CAPTURE_DIR, 'local-captured-source-parity.json');
  fs.writeFileSync(outPath, JSON.stringify(report, null, 2) + '\n');

  console.error('-'.repeat(86));
  console.error(
    `summary: ${passed}/${results.length} exact diagnostic replay; ` +
      `${accepted}/${results.length} accepted  failed=${acceptance_failed}`,
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
