import { chromium } from 'playwright';
import fs from 'fs';
import path from 'path';

const BASE_URL = (process.env.SOK_BASE_URL || 'http://127.0.0.1:8080').replace(/\/+$/, '');
const RUN_DATE = new Date().toISOString().slice(0, 10);
const OUT = path.resolve(import.meta.dirname, `responsive-navigation-smoke-${RUN_DATE}`);
const REFERENCE_DIR = path.resolve(import.meta.dirname, 'live-inventory-2026-06-26');
const TS = new Date().toISOString();
const PAGE_PATH = '/';
// Header autocomplete term. Must match seeded catalog/db data so `/ajax/search_help`
// returns suggestions; sparse terms (e.g. 'milf') legitimately yield zero items.
const SEARCH_TERM = process.env.SOK_SEARCH_TERM || 'brazzers';

const VIEWPORTS = {
  desktop: { width: 1440, height: 900 },
  mobile: { width: 390, height: 844, isMobile: true, hasTouch: true },
};

function localUrl(pagePath) {
  return new URL(pagePath, `${BASE_URL}/`).href;
}

function pushCheck(checks, name, passed, detail = null) {
  checks.push({ name, passed, detail });
}

function loadReferenceSnippet(viewportName) {
  const snippetPath = path.join(REFERENCE_DIR, `home__${viewportName}.json`);
  if (!fs.existsSync(snippetPath)) {
    return null;
  }
  return JSON.parse(fs.readFileSync(snippetPath, 'utf8'));
}

async function waitForNavReady(page) {
  await page.waitForFunction(
    () => typeof window.jQuery !== 'undefined' && document.querySelector('#side-panel') !== null,
    null,
    { timeout: 10000 },
  );
}

async function extractNavState(page) {
  return page.evaluate(() => {
    const style = (selector) => {
      const el = document.querySelector(selector);
      if (!el) {
        return null;
      }
      const computed = getComputedStyle(el);
      const rect = el.getBoundingClientRect();
      return {
        exists: true,
        display: computed.display,
        visibility: computed.visibility,
        opacity: computed.opacity,
        position: computed.position,
        top: computed.top,
        left: computed.left,
        right: computed.right,
        paddingTop: computed.paddingTop,
        height: rect.height,
        width: rect.width,
        rect: {
          top: rect.top,
          left: rect.left,
          right: rect.right,
          bottom: rect.bottom,
          width: rect.width,
          height: rect.height,
        },
      };
    };

    const overlap = (a, b) => {
      if (!a?.rect || !b?.rect) {
        return false;
      }
      const horizontal = Math.min(a.rect.right, b.rect.right) - Math.max(a.rect.left, b.rect.left);
      const vertical = Math.min(a.rect.bottom, b.rect.bottom) - Math.max(a.rect.top, b.rect.top);
      return horizontal > 1 && vertical > 1;
    };

    const footerPicture = document.querySelector('.footer-link picture');
    const footerImg = footerPicture?.querySelector('img') ?? null;
    const footerSources = footerPicture
      ? [...footerPicture.querySelectorAll('source')].map((source) => ({
          srcset: source.getAttribute('srcset'),
          media: source.getAttribute('media'),
        }))
      : [];

    const headerMenu = style('.header-menu');
    const btnMob = style('.btn-mob');
    const dayNight = style('#day-night');
    const btnSearch = style('.btn-search.activate');
    const searchBox = style('.search-box');
    const sidePanel = style('#side-panel');
    const closeOverlay = style('#close-overlay');
    const wrap = style('.wrap');
    const headerIn = style('.header-in');
    const submenu = document.querySelector('.header-menu > li.nav-show .submenu-container');
    const submenuState = submenu
      ? {
          classes: submenu.className,
          hovered: submenu.classList.contains('hovered'),
          loaded: submenu.classList.contains('loaded'),
          ...style('.header-menu > li.nav-show .submenu-container'),
        }
      : null;

    return {
      viewportWidth: window.innerWidth,
      scrollWidth: document.documentElement.scrollWidth,
      hasHorizontalOverflow: document.documentElement.scrollWidth > window.innerWidth + 4,
      headerMenu,
      btnMob,
      dayNight,
      btnSearch,
      searchBox,
      sidePanel,
      closeOverlay,
      wrap,
      headerIn,
      submenuState,
      searchItems: document.querySelectorAll('#search_result > ul > li').length,
      sideLinks: document.querySelectorAll('#side-panel .side-url').length,
      mainSearchFocused: document.activeElement?.id === 'main-search',
      footerLink: style('.footer-link'),
      footerCurrentSrc: footerImg?.currentSrc ?? null,
      footerSources,
      overlaps: {
        btnMob_dayNight: overlap(btnMob, dayNight),
        btnMob_btnSearch: overlap(btnMob, btnSearch),
        dayNight_btnSearch: overlap(dayNight, btnSearch),
      },
    };
  });
}

function buildBaselineChecks(viewportName, state) {
  const checks = [
    {
      name: 'no_horizontal_overflow',
      passed: !state.hasHorizontalOverflow,
      detail: `viewport=${state.viewportWidth} scroll=${state.scrollWidth}`,
    },
    {
      name: 'fixed_header_height_70px',
      passed: !!state.headerIn && Math.abs(state.headerIn.height - 70) <= 1,
      detail: `height=${state.headerIn?.height ?? null}`,
    },
    {
      name: 'wrap_padding_top_70px',
      passed: !!state.wrap && state.wrap.paddingTop === '70px',
      detail: `paddingTop=${state.wrap?.paddingTop ?? null}`,
    },
  ];

  if (viewportName === 'desktop') {
    pushCheck(checks, 'desktop_header_menu_visible', state.headerMenu?.display !== 'none');
    pushCheck(checks, 'desktop_mobile_button_present', !!state.btnMob);
    pushCheck(checks, 'desktop_day_night_present', !!state.dayNight);
    pushCheck(checks, 'desktop_search_toggle_present', !!state.btnSearch);
    pushCheck(
      checks,
      'desktop_footer_picture_breakpoints_present',
      state.footerSources.some((source) => source.media?.includes('949px'))
        && state.footerSources.some((source) => source.media?.includes('950px')),
      JSON.stringify(state.footerSources),
    );
    pushCheck(
      checks,
      'desktop_footer_logo_selected',
      typeof state.footerCurrentSrc === 'string' && state.footerCurrentSrc.includes('footer-logo.svg'),
      state.footerCurrentSrc,
    );
  } else {
    pushCheck(checks, 'mobile_header_menu_hidden', state.headerMenu?.display === 'none');
    pushCheck(checks, 'mobile_button_visible', state.btnMob?.display === 'block');
    if (state.viewportWidth <= 760) {
      pushCheck(
        checks,
        'mobile_day_night_positioned',
        state.dayNight?.position === 'absolute' && Math.abs(parseFloat(state.dayNight.right) - 80) <= 2,
        `right=${state.dayNight?.right ?? null}`,
      );
    }
    pushCheck(checks, 'mobile_footer_link_hidden', state.footerLink?.display === 'none');
    pushCheck(
      checks,
      'mobile_header_controls_do_not_overlap',
      !state.overlaps.btnMob_dayNight
        && !state.overlaps.btnMob_btnSearch
        && !state.overlaps.dayNight_btnSearch,
      JSON.stringify(state.overlaps),
    );
  }

  return checks;
}

async function runScenario(browser, scenario) {
  const context = await browser.newContext({
    viewport: scenario.viewport,
    userAgent: scenario.viewportName === 'mobile'
      ? 'Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1'
      : undefined,
  });

  const page = await context.newPage();
  const url = localUrl(PAGE_PATH);
  const checks = [];
  let status = null;
  let state = null;
  let error = null;

  try {
    const response = await page.goto(url, { waitUntil: 'domcontentloaded', timeout: 30000 });
    await page.waitForLoadState('networkidle', { timeout: 5000 }).catch(() => null);
    await waitForNavReady(page);
    await page.waitForTimeout(300);
    status = response?.status() ?? null;
    pushCheck(checks, 'document_status_200', status === 200, `status=${status}`);

    if (scenario.setup) {
      await scenario.setup(page);
    }

    state = await extractNavState(page);
    checks.push(...buildBaselineChecks(scenario.viewportName, state));

    if (scenario.interact) {
      await scenario.interact(page);
      state = await extractNavState(page);
      if (scenario.buildChecks) {
        checks.push(...scenario.buildChecks(state));
      }
    }
  } catch (err) {
    error = String(err);
  } finally {
    const passed = error === null && checks.every((check) => check.passed);
    const screenshotPath = path.join(OUT, `${scenario.id}.png`);
    const snippetPath = path.join(OUT, `${scenario.id}.json`);

    if (!error) {
      await page.screenshot({ path: screenshotPath, fullPage: false });
    }

    fs.writeFileSync(
      snippetPath,
      JSON.stringify(
        {
          id: scenario.id,
          url,
          final_url: page.url(),
          fetched_at: TS,
          viewport: scenario.viewportName,
          http_status: status,
          state,
          checks,
          passed,
          error,
        },
        null,
        2,
      ),
    );

    await context.close();

    return {
      id: scenario.id,
      viewport: scenario.viewportName,
      url,
      http_status: status,
      screenshot: path.relative(import.meta.dirname, screenshotPath),
      snippet: path.relative(import.meta.dirname, snippetPath),
      checks,
      passed,
      error,
    };
  }
}

const scenarios = [
  {
    id: 'desktop__baseline',
    viewportName: 'desktop',
    viewport: VIEWPORTS.desktop,
  },
  {
    id: 'mobile__baseline',
    viewportName: 'mobile',
    viewport: VIEWPORTS.mobile,
  },
  {
    id: 'desktop__mega_menu_hover',
    viewportName: 'desktop',
    viewport: VIEWPORTS.desktop,
    interact: async (page) => {
      const navItem = page.locator('.header-menu > li.nav-show').first();
      await navItem.hover();
      await page.waitForTimeout(700);
    },
    buildChecks: (state) => {
      const checks = [];
      pushCheck(checks, 'mega_menu_hovered_class', state.submenuState?.hovered === true);
      pushCheck(checks, 'mega_menu_loaded_class', state.submenuState?.loaded === true);
      pushCheck(
        checks,
        'mega_menu_visible_below_header',
        state.submenuState?.visibility === 'visible'
          && Math.abs(parseFloat(state.submenuState?.top) - 70) <= 2,
        `top=${state.submenuState?.top ?? null} visibility=${state.submenuState?.visibility ?? null}`,
      );
      return checks;
    },
  },
  {
    id: 'desktop__search_expand',
    viewportName: 'desktop',
    viewport: VIEWPORTS.desktop,
    searchAjax: { status: null },
    interact: async (page) => {
      await waitForNavReady(page);
      page.on('response', (res) => {
        if (res.url().includes('/ajax/search_help')) {
          scenarios.find((s) => s.id === 'desktop__search_expand').searchAjax.status = res.status();
        }
      });
      await page.locator('.btn-search.activate').click({ force: true });
      await page.waitForFunction(
        () => document.querySelector('.search-box')?.classList.contains('active'),
        null,
        { timeout: 5000 },
      );
      await page.locator('#main-search').fill(SEARCH_TERM);
      await page.waitForTimeout(1200);
    },
    buildChecks: (state) => {
      const checks = [];
      pushCheck(checks, 'search_box_active', state.searchBox?.top === '17px');
      pushCheck(checks, 'desktop_menu_hidden_during_search', state.headerMenu?.visibility === 'hidden');
      pushCheck(checks, 'search_input_focused', state.mainSearchFocused);
      // Header autocomplete fires POST /ajax/search_help; assert the request reaches
      // the app and returns 200. DOM item rendering depends on the mirrored
      // main.min.js calling `$.parseJSON` on the raw response string. The app now
      // serves this response as `text/html` + `nosniff` to match live pornsok.com
      // (sok-replica.5.7), so jQuery 3.3.1 does not auto-parse the body and the
      // single `$.parseJSON` succeeds, letting items render.
      const ajaxStatus = scenarios.find((s) => s.id === 'desktop__search_expand').searchAjax.status;
      pushCheck(checks, 'search_help_request_ok', ajaxStatus === 200, `ajax_status=${ajaxStatus}`);
      // When the seeded term yields matches, the autocomplete dropdown should
      // contain at least one rendered <li>. Zero items is only acceptable when
      // the query legitimately has no matches.
      pushCheck(
        checks,
        'search_help_items_rendered',
        state.searchItems > 0,
        `search_items=${state.searchItems}`,
      );
      return checks;
    },
  },
  {
    id: 'mobile__side_panel_open',
    viewportName: 'mobile',
    viewport: VIEWPORTS.mobile,
    setup: async (page) => {
      await page.waitForFunction(() => document.querySelector('#side-panel') !== null, null, { timeout: 5000 });
    },
    interact: async (page) => {
      await page.locator('.btn-mob').click({ force: true });
      await page.waitForTimeout(300);
    },
    buildChecks: (state) => {
      const checks = [];
      const sideLeft = parseFloat(state.sidePanel?.left ?? '999');
      pushCheck(
        checks,
        'side_panel_active',
        state.sidePanel?.visibility === 'visible' && sideLeft >= -12 && sideLeft <= 2,
        `left=${state.sidePanel?.left ?? null}`,
      );
      pushCheck(checks, 'close_overlay_visible', state.closeOverlay?.display !== 'none');
      pushCheck(checks, 'side_panel_links_populated', state.sideLinks > 0, `links=${state.sideLinks}`);
      return checks;
    },
  },
  {
    id: 'mobile__side_panel_close',
    viewportName: 'mobile',
    viewport: VIEWPORTS.mobile,
    setup: async (page) => {
      await page.waitForFunction(() => document.querySelector('#side-panel') !== null, null, { timeout: 5000 });
      await page.locator('.btn-mob').click({ force: true });
      await page.waitForTimeout(250);
    },
    interact: async (page) => {
      // Click the dimmed overlay in the open area to the right of the 260px side
      // panel; a centered force-click lands on a side-url link and navigates away.
      const box = await page.locator('#close-overlay').boundingBox();
      const x = box ? Math.min(box.x + box.width - 20, 360) : 360;
      const y = box ? box.y + box.height / 2 : 400;
      await page.mouse.click(x, y);
      await page.waitForTimeout(450);
    },
    buildChecks: (state) => {
      const checks = [];
      pushCheck(checks, 'side_panel_closed', state.sidePanel?.left !== '0px');
      pushCheck(
        checks,
        'close_overlay_hidden',
        state.closeOverlay?.display === 'none' || state.closeOverlay?.height === 0,
        `display=${state.closeOverlay?.display ?? null} height=${state.closeOverlay?.height ?? null}`,
      );
      return checks;
    },
  },
  {
    id: 'mobile__search_expand',
    viewportName: 'mobile',
    viewport: VIEWPORTS.mobile,
    searchAjax: { status: null },
    interact: async (page) => {
      await waitForNavReady(page);
      page.on('response', (res) => {
        if (res.url().includes('/ajax/search_help')) {
          scenarios.find((s) => s.id === 'mobile__search_expand').searchAjax.status = res.status();
        }
      });
      await page.locator('.btn-search.activate').click({ force: true });
      await page.waitForFunction(
        () => document.querySelector('.search-box')?.classList.contains('active'),
        null,
        { timeout: 5000 },
      );
      await page.locator('#main-search').fill(SEARCH_TERM);
      await page.waitForTimeout(1200);
    },
    buildChecks: (state) => {
      const checks = [];
      pushCheck(checks, 'mobile_search_box_active', state.searchBox?.top === '17px');
      pushCheck(checks, 'mobile_search_input_focused', state.mainSearchFocused);
      // See desktop__search_expand: assert the autocomplete request returns 200;
      // DOM rendering is gated by the JSON content-type vs `$.parseJSON` follow-up.
      const ajaxStatus = scenarios.find((s) => s.id === 'mobile__search_expand').searchAjax.status;
      pushCheck(checks, 'mobile_search_help_request_ok', ajaxStatus === 200, `ajax_status=${ajaxStatus}`);
      pushCheck(
        checks,
        'mobile_search_controls_do_not_overlap',
        !state.overlaps.btnMob_btnSearch && !state.overlaps.dayNight_btnSearch,
        JSON.stringify(state.overlaps),
      );
      return checks;
    },
  },
  {
    id: 'mobile__footer_logo_breakpoint',
    viewportName: 'mobile',
    viewport: { width: 949, height: 844, isMobile: true, hasTouch: true },
    buildChecks: (state) => {
      const checks = [];
      pushCheck(
        checks,
        'footer_breakpoint_uses_spacer_source',
        typeof state.footerCurrentSrc === 'string' && state.footerCurrentSrc.includes('spacer.gif'),
        state.footerCurrentSrc,
      );
      return checks;
    },
  },
];

fs.mkdirSync(OUT, { recursive: true });

const manifest = {
  captured_at: TS,
  tool: 'playwright-chromium-headless',
  base_url: BASE_URL,
  page_path: PAGE_PATH,
  reference_dir: path.relative(import.meta.dirname, REFERENCE_DIR),
  reference_snippets: {
    desktop: loadReferenceSnippet('desktop') ? 'home__desktop.json' : null,
    mobile: loadReferenceSnippet('mobile') ? 'home__mobile.json' : null,
  },
  scenarios: [],
  errors: [],
};

const browser = await chromium.launch({ headless: true });
for (const scenario of scenarios) {
  const result = await runScenario(browser, scenario);
  manifest.scenarios.push({
    id: result.id,
    viewport: result.viewport,
    url: result.url,
    http_status: result.http_status,
    screenshot: result.screenshot,
    snippet: result.snippet,
    passed: result.passed,
  });

  for (const check of result.checks) {
    if (!check.passed) {
      manifest.errors.push({
        id: result.id,
        check: check.name,
        detail: check.detail ?? null,
      });
    }
  }

  if (result.error) {
    manifest.errors.push({ id: result.id, error: result.error });
  }

  console.log(result.passed ? 'ok' : 'fail', result.id, result.http_status);
}

await browser.close();

const manifestPath = path.join(OUT, 'manifest.json');
fs.writeFileSync(manifestPath, JSON.stringify(manifest, null, 2));
console.log('manifest written', path.relative(process.cwd(), manifestPath));

if (manifest.errors.length > 0) {
  console.error(`${manifest.errors.length} responsive navigation check(s) failed`);
  process.exitCode = 1;
}
