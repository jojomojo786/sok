import crypto from 'node:crypto';

export function sha256(bytes) {
  return crypto.createHash('sha256').update(bytes).digest('hex');
}

export function normalizeDynamicBytes(bytes) {
  const text = Buffer.isBuffer(bytes) ? bytes.toString('utf8') : String(bytes);
  return Buffer.from(
    text
      .replace(/data-cf-modified-[a-f0-9-]+=""/gi, 'data-cf-modified-__=""')
      .replace(/data-cfemail="[a-f0-9]+"/gi, 'data-cfemail="__"')
      .replace(/data-cf-settings="[a-f0-9]+-\|49"/gi, 'data-cf-settings="__-|49"')
      .replace(/type="[a-f0-9]+-text\/javascript"/gi, 'type="CF-text/javascript"')
      .replace(/type="text\/javascript"/gi, 'type="CF-text/javascript"')
      .replace(
        /\/cdn-cgi\/scripts\/[a-z0-9./_-]+\/cloudflare-static\/rocket-loader\.min\.js/gi,
        '/fox-tpl/js/rocket-loader.min.js',
      )
      .replace(/cf_chl_[a-z0-9_-]+/gi, 'cf_chl___'),
    'utf8',
  );
}

export function decodeEntities(s) {
  return String(s)
    .replace(/&#x([0-9a-fA-F]+);/g, (_, h) => String.fromCodePoint(parseInt(h, 16)))
    .replace(/&#(\d+);/g, (_, d) => String.fromCodePoint(parseInt(d, 10)))
    .replace(/&nbsp;/gi, ' ')
    .replace(/&amp;/gi, '&')
    .replace(/&lt;/gi, '<')
    .replace(/&gt;/gi, '>')
    .replace(/&quot;/gi, '"')
    .replace(/&#39;|&apos;/gi, "'");
}

export function stripTags(s) {
  return String(s).replace(/<[^>]*>/g, ' ').replace(/\s+/g, ' ');
}

export function collapseHtml(html) {
  return String(html).replace(/\s+/g, ' ').trim();
}

export function extractBlock(html, tagName) {
  const re = new RegExp(`<${tagName}\\b[\\s\\S]*?<\\/${tagName}>`, 'i');
  const match = String(html).match(re);
  return match ? collapseHtml(match[0]) : null;
}

export function extractTitle(html) {
  const match = String(html).match(/<title[^>]*>([\s\S]*?)<\/title>/i);
  return match ? decodeEntities(match[1]).trim() : null;
}

export function extractDescription(html) {
  const text = String(html);
  return (
    text.match(/<meta\b[^>]*name=["']description["'][^>]*content=["']([^"']*)["'][^>]*>/i)?.[1] ??
    text.match(/<meta\b[^>]*content=["']([^"']*)["'][^>]*name=["']description["'][^>]*>/i)?.[1] ??
    null
  );
}

export function extractH1s(html) {
  const out = [];
  const re = /<h1\b[^>]*>([\s\S]*?)<\/h1>/gi;
  let match;
  while ((match = re.exec(String(html))) !== null) {
    const text = decodeEntities(stripTags(match[1])).trim();
    if (text) out.push(text);
  }
  return out;
}

export function extractCanonical(html) {
  const re = /<link\b[^>]*>/gi;
  let match;
  while ((match = re.exec(String(html))) !== null) {
    const tag = match[0];
    if (/rel\s*=\s*["']\s*canonical\s*["']/i.test(tag)) {
      const href = tag.match(/href\s*=\s*["']([^"']*)["']/i);
      if (href) return href[1].trim();
    }
  }
  return null;
}

export function countClassCombo(html, requiredClasses) {
  let count = 0;
  const re = /\bclass\s*=\s*(?:"([^"]*)"|'([^']*)')/gi;
  let match;
  while ((match = re.exec(String(html))) !== null) {
    const classes = (match[1] ?? match[2] ?? '').split(/\s+/);
    const set = new Set(classes);
    if (requiredClasses.every((className) => set.has(className))) count += 1;
  }
  return count;
}

export function countThumbClass(html, second) {
  return countClassCombo(html, ['thumb', second]);
}

export function extractCardNames(html) {
  const out = [];
  const re = /<div class="thumb-title"[^>]*>([\s\S]*?)<\/div>/gi;
  let match;
  while ((match = re.exec(String(html))) !== null) {
    const text = stripTags(match[1]).trim();
    if (text) out.push(text);
  }
  return out;
}
