
import { readFileSync, writeFileSync, existsSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const dir = join(__dirname, '..', 'src', 'lib', 'i18n');

const locales = ["ar", "de", "es", "fa", "fr", "he", "hi", "id", "it", "ja", "pl", "pt", "ru", "th", "tr", "vi", "zh"];

const zero_map = {
    "zh": {
        "certificate": { "no_search_results": "未找到搜索结果。", "completed_at": "完成时间", "download": "下载" }
    },
    "ja": {
        "certificate": { "no_search_results": "検索結果が見つかりませんでした。", "completed_at": "修了日", "download": "ダウンロード", "management_account": "管理アカウント" },
        "editor": { "require_feedback_desc": "参加者に各ステップのフィードバックを要求します。", "settings_tab": "設定", "settings_description": "コデラボの全般設定を構成します。" }
    },
    "ar": {
        "ai_generator": { "upload_files": "رفع الملفات", "diff_empty": "لا توجد فروق.", "diff_too_large": "الفارق كبير جداً." },
        "admin": { "consultant": { "help_desc": "ابحث عن ملفاتك أو اطرح أسئلة لتحسين مختبرك.", "placeholder": "اكتب سؤالك هنا...", "api_key_desc": "يُستخدم مفتاح Gemini لتشغيل ميزات الذكاء الاصطناعي.", "sources": "المصادر" } }
    },
    "fa": {
        "ai_generator": { "mode_advanced_star_message": "لطفاً با دادن ستاره در گیت‌هاب از ما حمایت کنید!" },
        "admin": { "consultant": { "help_desc": "فایل‌های خود را جستجو کنید یا برای بهبود کدلب خود سوال بپرسید.", "placeholder": "سوال خود را اینجا بنویسید...", "api_key_desc": "کلید Gemini برای اجرای ویژگی‌های هوش مصنوعی استفاده می‌شود.", "sources": "منابع" } }
    },
    "es": {
        "editor": { "guide_pro_result_revise": "Resultado de la revisión", "guide_pro_result_empty": "No hay resultados para mostrar.", "stuck_on_step": "¿Atascado en este paso?", "split_view": "Vista dividida", "certificate_tab": "Certificado" }
    },
    "he": {
        "ai_generator": { "mode_advanced_star_message": "נשמח אם תתמכו בנו בכוכב ב-GitHub!", "mode_advanced_star_button": "תנו לנו כוכב", "input_label": "הכנס את הקוד שלך כאן", "google_search": "חיפוש גוגל", "url_context": "הקשר קישור" }
    },
    "hi": {
        "ai_generator": { "mode_advanced_star_message": "कृपया गिटहब पर हमें स्टार देकर समर्थन करें!", "mode_advanced_star_button": "हमें स्टार दें", "input_label": "अपना कोड यहाँ डालें", "google_search": "गूगल सर्च", "url_context": "यूआरएल संदर्भ" }
    }
};

const deepMerge = (target, source) => {
    for (const key in source) {
        if (source[key] && typeof source[key] === 'object' && !Array.isArray(source[key])) {
            if (!target[key]) target[key] = {};
            deepMerge(target[key], source[key]);
        } else {
            target[key] = source[key];
        }
    }
};

locales.forEach(locale => {
    const data = zero_map[locale];
    if (data) {
        const filePath = join(dir, `${locale}.json`);
        if (existsSync(filePath)) {
            let content = JSON.parse(readFileSync(filePath, 'utf-8'));
            deepMerge(content, data);
            writeFileSync(filePath, JSON.stringify(content, null, 4) + '\n', 'utf-8');
            console.log(`Zero Update: ${locale}`);
        }
    }
});
