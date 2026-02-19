import { getLocaleFromNavigator, init, locale, register } from "svelte-i18n";

export const localeLoaders = {
    en: () => import("./en.json"),
    ko: () => import("./ko.json"),
    ja: () => import("./ja.json"),
    zh: () => import("./zh.json"),
    de: () => import("./de.json"),
    es: () => import("./es.json"),
    fr: () => import("./fr.json"),
    id: () => import("./id.json"),
    it: () => import("./it.json"),
    pl: () => import("./pl.json"),
    pt: () => import("./pt.json"),
    vi: () => import("./vi.json"),
    tr: () => import("./tr.json"),
    ru: () => import("./ru.json"),
    he: () => import("./he.json"),
    ar: () => import("./ar.json"),
    fa: () => import("./fa.json"),
    hi: () => import("./hi.json"),
    bn: () => import("./bn.json"),
    th: () => import("./th.json"),
    "zh-TW": () => import("./zh-TW.json"),
} as const;

export const allLocales = [
    "ko",
    "ja",
    "zh-TW",
    "zh",
    "de",
    "es",
    "fr",
    "id",
    "it",
    "pl",
    "pt",
    "vi",
    "tr",
    "ru",
    "he",
    "ar",
    "fa",
    "hi",
    "bn",
    "th",
    "en",
] as const;

export function registerAllLocales(registerFn: typeof register = register): void {
    for (const [localeCode, loader] of Object.entries(localeLoaders)) {
        registerFn(localeCode, loader as () => Promise<unknown>);
    }
}

export function resolveInitialLocale(navLocale: string | null | undefined): string {
    return (
        allLocales.find((value) =>
            value.includes("-")
                ? navLocale?.toLowerCase().startsWith(value.toLowerCase())
                : navLocale?.startsWith(value),
        ) || "en"
    );
}

export function setupI18n(
    registerFn: typeof register = register,
    initFn: typeof init = init,
    localeStore: typeof locale = locale,
    navLocale: string | null | undefined = getLocaleFromNavigator(),
): string {
    registerAllLocales(registerFn);

    const initialLocale = resolveInitialLocale(navLocale);
    initFn({
        fallbackLocale: "en",
        initialLocale,
    });

    localeStore.set(initialLocale);
    return initialLocale;
}

setupI18n();
