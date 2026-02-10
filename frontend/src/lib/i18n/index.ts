import { register, init, getLocaleFromNavigator } from 'svelte-i18n';

// Register all 21 supported locales (lazy-loaded)
register('en', () => import('./en.json'));
register('ko', () => import('./ko.json'));
register('ja', () => import('./ja.json'));
register('zh', () => import('./zh.json'));
register('de', () => import('./de.json'));
register('es', () => import('./es.json'));
register('fr', () => import('./fr.json'));
register('id', () => import('./id.json'));
register('it', () => import('./it.json'));
register('pl', () => import('./pl.json'));
register('pt', () => import('./pt.json'));
register('vi', () => import('./vi.json'));
register('tr', () => import('./tr.json'));
register('ru', () => import('./ru.json'));
register('he', () => import('./he.json'));
register('ar', () => import('./ar.json'));
register('fa', () => import('./fa.json'));
register('hi', () => import('./hi.json'));
register('bn', () => import('./bn.json'));
register('th', () => import('./th.json'));
register('zh-TW', () => import('./zh-TW.json'));

const allLocales = [
    'ko', 'ja', 'zh-TW', 'zh', 'de', 'es', 'fr', 'id', 'it',
    'pl', 'pt', 'vi', 'tr', 'ru', 'he', 'ar', 'fa', 'hi', 'bn', 'th', 'en'
];

const navLocale = getLocaleFromNavigator();
// Match zh-TW first, then fall back to base language codes
const initialLocale = allLocales.find(l =>
    l.includes('-') ? navLocale?.toLowerCase().startsWith(l.toLowerCase()) : navLocale?.startsWith(l)
) || 'en';

init({
    fallbackLocale: 'en',
    initialLocale,
});

// Force set locale immediately to prevent "locale not set" errors
import { locale } from 'svelte-i18n';
locale.set(initialLocale);
