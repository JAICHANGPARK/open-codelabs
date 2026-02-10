
import { readFileSync, readdirSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const i18nDir = join(__dirname, '..', 'src', 'lib', 'i18n');

const en = JSON.parse(readFileSync(join(i18nDir, 'en.json'), 'utf-8'));

function flattenKeys(obj, prefix = '') {
    let keys = {};
    for (const [k, v] of Object.entries(obj)) {
        const newKey = prefix ? `${prefix}.${k}` : k;
        if (typeof v === 'object' && v !== null && !Array.isArray(v)) {
            Object.assign(keys, flattenKeys(v, newKey));
        } else {
            keys[newKey] = v;
        }
    }
    return keys;
}

const enKeys = flattenKeys(en);

const files = readdirSync(i18nDir).filter(f => f.endsWith('.json') && f !== 'en.json');

console.log('--- Translation Audit Report ---');
console.log(`Base: en.json (${Object.keys(enKeys).length} keys)\n`);

const summary = {};

files.forEach(file => {
    const locale = file.replace('.json', '');
    const content = JSON.parse(readFileSync(join(i18nDir, file), 'utf-8'));
    const locKeys = flattenKeys(content);

    const missing = [];
    const identical = [];

    for (const [key, val] of Object.entries(enKeys)) {
        if (!(key in locKeys)) {
            missing.push(key);
        } else if (locKeys[key] === val && val.length > 5) { // Ignore short strings like "OK", names, etc.
            identical.push(key);
        }
    }

    summary[locale] = { missing: missing.length, identical: identical.length };

    if (missing.length > 0 || identical.length > 0) {
        console.log(`[${locale}] Missing: ${missing.length}, Identical (potential untranslated): ${identical.length}`);
        if (missing.length > 0) console.log(`  First 5 missing: ${missing.slice(0, 5).join(', ')}`);
        if (identical.length > 0) console.log(`  First 5 identical: ${identical.slice(0, 5).join(', ')}`);
    } else {
        console.log(`[${locale}] OK`);
    }
});

console.log('\n--- Summary ---');
console.table(summary);
