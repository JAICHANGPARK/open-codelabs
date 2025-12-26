import { register, init, getLocaleFromNavigator } from 'svelte-i18n';

register('en', () => import('./en.json'));
register('ko', () => import('./ko.json'));
register('ja', () => import('./ja.json'));
register('zh', () => import('./zh.json'));

init({
    fallbackLocale: 'en',
    initialLocale: getLocaleFromNavigator(),
});
