import { register, init, getLocaleFromNavigator } from 'svelte-i18n';

register('en', () => import('./en.json'));
register('ko', () => import('./ko.json'));
register('ja', () => import('./ja.json'));
register('zh', () => import('./zh.json'));

const navLocale = getLocaleFromNavigator();
const initialLocale = ['ko', 'ja', 'zh', 'en'].find(l => navLocale?.startsWith(l)) || 'en';

init({
    fallbackLocale: 'en',
    initialLocale,
});

// Force set locale immediately to prevent "locale not set" errors
import { locale } from 'svelte-i18n';
locale.set(initialLocale);
