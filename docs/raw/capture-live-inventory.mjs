import { chromium } from 'playwright';
import fs from 'fs';
import path from 'path';

const OUT = path.resolve(import.meta.dirname, 'live-inventory-2026-06-26');
const TS = new Date().toISOString();

const PAGES = [
  { key: 'home', url: 'https://pornsok.com/' },
  { key: 'categories', url: 'https://pornsok.com/categories' },
  { key: 'pornstars', url: 'https://pornsok.com/pornstars' },
  { key: 'channels', url: 'https://pornsok.com/channels' },
  { key: 'category-milf', url: 'https://pornsok.com/milf' },
  { key: 'video-sample', url: 'https://pornsok.com/video/dog-house-madi-collins-knows-more-sex-than-her-step-bro-decides-to-show-him-what-he-should-do.html' },
  { key: 'channel-brazzers', url: 'https://pornsok.com/channel/brazzers' },
  { key: 'pornstar-angela-white', url: 'https://pornsok.com/pornstar/angela-white' },
  { key: 'search-test', url: 'https://pornsok.com/search?q=test' },
  { key: 'legal-privacy', url: 'https://pornsok.com/page/privacy.html' },
];

const VIEWPORTS = {
  desktop: { width: 1440, height: 900 },
  mobile: { width: 390, height: 844, isMobile: true, hasTouch: true },
};

async function extractSnippet(page) {
  return page.evaluate(() => {
    const pick = (sel) => {
      const el = document.querySelector(sel);
      if (!el) return null;
      const clone = el.cloneNode(true);
      clone.querySelectorAll('script,style,noscript').forEach((n) => n.remove());
      const html = clone.outerHTML.replace(/\\s+/g, ' ').trim();
      return html.length > 4000 ? html.slice(0, 4000) + '…' : html;
    };
    const meta = (name) => document.querySelector(`meta[name="${name}"]`)?.getAttribute('content') ?? null;
    const linkCanon = document.querySelector('link[rel="canonical"]')?.getAttribute('href') ?? null;
    const h1 = [...document.querySelectorAll('h1')].map((h) => h.textContent?.trim()).filter(Boolean);
    return {
      title: document.title,
      canonical: linkCanon,
      lang: document.documentElement.lang,
      theme: document.documentElement.getAttribute('data-theme'),
      h1,
      description: meta('description'),
      header: pick('.header'),
      main: pick('main') || pick('.content') || pick('#content') || pick('.all_cats') || pick('.all_pornstars'),
      footer: pick('.footer'),
      bodyClasses: document.body.className,
      viewportWidth: window.innerWidth,
      hasFilterSection: !!document.querySelector('.filter-section'),
      hasPageNav: !!document.querySelector('.page_nav'),
      hasPlayer: !!document.querySelector('#player_container'),
      hasSearchBoxPage: !!document.querySelector('#search-page-input'),
      hasSearchGenresInput: !!document.querySelector('#search-genres-input'),
    };
  });
}

const manifest = { captured_at: TS, tool: 'playwright-chromium-headless', pages: [], errors: [] };

const browser = await chromium.launch({ headless: true });
for (const vpName of Object.keys(VIEWPORTS)) {
  const context = await browser.newContext({
    viewport: VIEWPORTS[vpName],
    userAgent: vpName === 'mobile'
      ? 'Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1'
      : undefined,
  });
  const page = await context.newPage();
  for (const { key, url } of PAGES) {
    const id = `${key}__${vpName}`;
    try {
      const resp = await page.goto(url, { waitUntil: 'domcontentloaded', timeout: 60000 });
      await page.waitForTimeout(2500);
      const status = resp?.status() ?? null;
      const snippet = await extractSnippet(page);
      const shotPath = path.join(OUT, `${id}.png`);
      await page.screenshot({ path: shotPath, fullPage: false });
      const snippetPath = path.join(OUT, `${id}.json`);
      fs.writeFileSync(snippetPath, JSON.stringify({ url, fetched_at: TS, viewport: vpName, http_status: status, ...snippet }, null, 2));
      manifest.pages.push({ key, viewport: vpName, url, final_url: page.url(), http_status: status, screenshot: `live-inventory-2026-06-26/${id}.png`, snippet: `live-inventory-2026-06-26/${id}.json` });
      console.log('ok', id, status);
    } catch (e) {
      manifest.errors.push({ key, viewport: vpName, url, error: String(e) });
      console.error('fail', id, e.message);
    }
  }
  await context.close();
}
await browser.close();
fs.writeFileSync(path.join(OUT, 'manifest.json'), JSON.stringify(manifest, null, 2));
console.log('manifest written', manifest.pages.length, 'captures');
