#!/usr/bin/env node
// collect-live-byte-instability.mjs
//
// Repeatedly samples live PornsOK responses to prove which exact byte targets
// are stable before the local replica is involved.

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  countThumbClass,
  extractCardNames,
  normalizeDynamicBytes,
  sha256,
} from './lib/html-analysis.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const LIVE_BASE = (process.env.SOK_LIVE_BASE_URL || 'https://pornsok.com').replace(/\/+$/, '');
const SAMPLES = Number(process.env.SOK_LIVE_INSTABILITY_SAMPLES || 3);
const TIMEOUT_MS = Number(process.env.SOK_PARITY_TIMEOUT_MS || 30000);
const SEARCH_TERM = process.env.SOK_PARITY_SEARCH_TERM || 'milf';

const UA =
  'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 ' +
  '(KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36 SOKLiveInstability/1.0';

const GET_ROUTES = [
  { label: 'home', path: '/', method: 'GET' },
  { label: 'categories', path: '/categories', method: 'GET' },
  { label: 'pornstars', path: '/pornstars', method: 'GET' },
  { label: 'channels', path: '/channels', method: 'GET' },
  { label: 'privacy', path: '/page/privacy.html', method: 'GET' },
];

const SEARCH_BODY = `text=${encodeURIComponent(SEARCH_TERM)}`;
const AJAX_PROBES = [
  { label: 'update_pornstars', path: '/ajax/update_pornstars', method: 'POST', body: '' },
  { label: 'update_channels', path: '/ajax/update_channels', method: 'POST', body: '' },
  { label: 'search_help', path: '/ajax/search_help', method: 'POST', body: SEARCH_BODY },
  {
    label: 'search_cats_tags_queries',
    path: '/ajax/search_cats_tags_queries',
    method: 'POST',
    body: SEARCH_BODY,
  },
];

async function fetchProbe(probe) {
  const ctl = new AbortController();
  const timer = setTimeout(() => ctl.abort(), TIMEOUT_MS);
  const headers = { 'user-agent': UA, accept: '*/*' };
  if (probe.method === 'POST') {
    headers['content-type'] = 'application/x-www-form-urlencoded';
  }
  try {
    const resp = await fetch(LIVE_BASE + probe.path, {
      method: probe.method,
      body: probe.method === 'POST' ? probe.body : undefined,
      headers,
      signal: ctl.signal,
      redirect: 'follow',
    });
    const bytes = Buffer.from(await resp.arrayBuffer());
    return {
      ok: true,
      status: resp.status,
      content_type: resp.headers.get('content-type'),
      final_url: resp.url,
      redirected: resp.redirected,
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

function analyze(result) {
  if (!result.ok) {
    return {
      status: null,
      content_type: null,
      byte_length: null,
      sha256: null,
      normalized_byte_length: null,
      normalized_sha256: null,
      thumb_vid_count: null,
      thumb_cat_count: null,
      cards: [],
      error: result.error,
    };
  }
  const html = result.bytes.toString('utf8');
  const normalized = normalizeDynamicBytes(result.bytes);
  return {
    status: result.status,
    content_type: result.content_type,
    byte_length: result.bytes.length,
    sha256: sha256(result.bytes),
    normalized_byte_length: normalized.length,
    normalized_sha256: sha256(normalized),
    thumb_vid_count: countThumbClass(html, 'vid'),
    thumb_cat_count: countThumbClass(html, 'cat'),
    cards: extractCardNames(html),
    error: null,
  };
}

function unique(values) {
  return [...new Set(values.filter((v) => v != null))];
}

function classify(probe, samples) {
  const successful = samples.filter((s) => !s.error);
  const uniqueRaw = unique(successful.map((s) => s.sha256)).length;
  const uniqueNorm = unique(successful.map((s) => s.normalized_sha256)).length;
  const uniqueBytes = unique(successful.map((s) => s.byte_length)).length;
  const allSixCat = successful.length > 0 && successful.every((s) => s.thumb_cat_count === 6);

  if (successful.length !== samples.length) return 'transport_error';
  if (uniqueRaw === 1 && uniqueNorm === 1) return 'live_stable_exact';
  if (uniqueRaw > 1 && uniqueNorm === 1) return 'live_raw_dynamic_normalized_stable';
  if (probe.path === '/ajax/update_pornstars' || probe.path === '/ajax/update_channels') {
    if (uniqueNorm > 1 && allSixCat) return 'live_random_six_card_sample';
  }
  if (uniqueNorm > 1 || uniqueBytes > 1) return 'live_content_dynamic';
  return 'live_dynamic_uncategorized';
}

async function runProbe(probe) {
  const samples = [];
  for (let i = 0; i < SAMPLES; i += 1) {
    const result = await fetchProbe(probe);
    samples.push({ index: i, ...analyze(result) });
  }

  const classification = classify(probe, samples);
  return {
    label: probe.label,
    path: probe.path,
    method: probe.method,
    classification,
    unique_sha256: unique(samples.map((s) => s.sha256)).length,
    unique_normalized_sha256: unique(samples.map((s) => s.normalized_sha256)).length,
    unique_byte_length: unique(samples.map((s) => s.byte_length)).length,
    sample_count: samples.length,
    samples,
  };
}

function pad(value, width) {
  return String(value).padEnd(width);
}

async function main() {
  const probes = [...GET_ROUTES, ...AJAX_PROBES];
  console.error(`live: ${LIVE_BASE}`);
  console.error(`samples_per_probe=${SAMPLES}`);
  console.error(pad('label', 26) + pad('raw', 8) + pad('norm', 8) + pad('bytes', 8) + 'classification');
  console.error('-'.repeat(86));

  const results = [];
  for (const probe of probes) {
    const result = await runProbe(probe);
    results.push(result);
    console.error(
      pad(result.label, 26) +
        pad(result.unique_sha256, 8) +
        pad(result.unique_normalized_sha256, 8) +
        pad(result.unique_byte_length, 8) +
        result.classification,
    );
  }

  const counts = results.reduce((acc, result) => {
    acc[result.classification] = (acc[result.classification] || 0) + 1;
    return acc;
  }, {});
  const report = {
    generated_at: new Date().toISOString(),
    tool: 'collect-live-byte-instability.mjs',
    live_base_url: LIVE_BASE,
    samples_per_probe: SAMPLES,
    summary: {
      total: results.length,
      classification_counts: counts,
    },
    probes: results,
  };

  const ts = report.generated_at.slice(0, 10);
  const outPath = process.env.SOK_LIVE_INSTABILITY_OUTPUT
    ? path.resolve(process.env.SOK_LIVE_INSTABILITY_OUTPUT)
    : path.join(__dirname, `live-byte-instability-${ts}.json`);
  fs.writeFileSync(outPath, JSON.stringify(report, null, 2) + '\n', 'utf8');

  console.error('-'.repeat(86));
  console.error(`report: ${outPath}`);
}

main().catch((e) => {
  console.error('fatal:', e);
  process.exitCode = 2;
});
