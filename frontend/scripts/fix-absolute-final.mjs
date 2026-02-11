
import { readFileSync, writeFileSync, existsSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const dir = join(__dirname, '..', 'src', 'lib', 'i18n');

const absolute_sweep_map = {
    "ar": {
        "workspace": {
            "copy_title": "نسخ رمز المرحلة",
            "copy_desc": "انسخ الرمز النهائي للمرحلة السابقة كرمز بداية للمرحلة التالية.",
            "copy_button": "نسخ نهاية المرحلة {source} ← بداية المرحلة {target}",
            "copy_success": "تم نسخ رمز نهاية المرحلة {source} إلى رمز بداية المرحلة {target}.",
            "actions": { "browse": "تصفح الملفات", "download": "تحميل مساحة العمل" }
        },
        "certificate": { "issued_to": "صادر إلى", "completion_date": "تاريخ الإكمال", "verify_desc": "تحقق من صحة هذه الشهادة" }
    },
    "fa": {
        "workspace": {
            "copy_title": "کپی کد مرحله",
            "copy_desc": "کد پایانی مرحله قبلی را به عنوان کد شروع مرحله بعدی کپی کنید.",
            "copy_button": "کپی پایان مرحله {source} ← شروع مرحله {target}",
            "copy_success": "کد پایان مرحله {source} به کد شروع مرحله {target} کپی شد.",
            "actions": { "browse": "مرور فایل‌ها", "download": "دانلود فضای کاری" }
        }
    },
    "he": {
        "workspace": {
            "copy_title": "העתק קוד שלב",
            "copy_desc": "העתק את הקוד הסופי של השלב הקודם כקוד ההתחלה של השלב הבא.",
            "copy_button": "העתק סוף שלב {source} ← התחלת שלב {target}",
            "copy_success": "קוד סוף שלב {source} הועתק לקוד התחלת שלב {target}.",
            "actions": { "browse": "עיון בקבצים", "download": "הורד סביבת עבודה" }
        }
    },
    "hi": {
        "workspace": {
            "copy_title": "चरण कोड कॉपी करें",
            "copy_desc": "पिछले चरण के अंतिम कोड को अगले चरण के शुरुआती कोड के रूप में कॉपी करें।",
            "copy_button": "चरण {source} का अंत कॉपी करें → चरण {target} का प्रारंभ",
            "copy_success": "चरण {source} का अंत कोड चरण {target} के प्रारंभ कोड में कॉपी किया गया है।",
            "actions": { "browse": "फ़ाइलें ब्राउज़ करें", "download": "वर्कस्पेस डाउनलोड करें" }
        }
    },
    "th": {
        "workspace": {
            "copy_title": "คัดลอกโค้ดขั้นตอน",
            "copy_desc": "คัดลอกโค้ดสุดท้ายของขั้นตอนก่อนหน้าเพื่อเป็นโค้ดเริ่มต้นของขั้นตอนถัดไป",
            "copy_button": "คัดลอกจบขั้นตอน {source} ← เริ่มขั้นตอน {target}",
            "copy_success": "คัดลอกโค้ดจบขั้นตอน {source} ไปยังขั้นตอนเริ่ม {target} เรียบร้อยแล้ว",
            "actions": { "browse": "เรียกดูไฟล์", "download": "ดาวน์โหลดเวิร์กสเปซ" }
        }
    },
    "tr": {
        "workspace": {
            "copy_title": "Adım Kodunu Kopyala",
            "copy_desc": "Önceki adımın son kodunu sonraki adımın başlangıç kodu olarak kopyalayın.",
            "copy_button": "Adım {source} sonunu kopyala → Adım {target} başlangıcı",
            "copy_success": "Adım {source} son kodu Adım {target} başlangıç koduna kopyalandı.",
            "actions": { "browse": "Dosyalara Göz At", "download": "Çalışma Alanını İndir" }
        }
    },
    "vi": {
        "workspace": {
            "copy_title": "Sao chép mã bước",
            "copy_desc": "Sao chép mã cuối cùng của bước trước làm mã bắt đầu cho bước tiếp theo.",
            "copy_button": "Sao chép kết thúc bước {source} ← bắt đầu bước {target}",
            "copy_success": "Mã cuối bước {source} đã được sao chép vào mã đầu bước {target}.",
            "actions": { "browse": "Duyệt tệp tin", "download": "Tải xuống không gian làm việc" }
        }
    },
    "es": {
        "editor": { "guide_description": "Descripción de la guía", "markdown_editor": "Editor Markdown", "live_preview": "Vista previa en vivo" },
        "workspace": { "copy_title": "Copiar código de paso", "copy_button": "Copiar fin paso {source} → inicio paso {target}" }
    },
    "fr": {
        "editor": { "guide_description": "Description du guide", "markdown_editor": "Éditeur Markdown", "live_preview": "Aperçu en direct" },
        "workspace": { "copy_title": "Copier le code de l'étape", "copy_button": "Copier fin étape {source} → début étape {target}" }
    },
    "id": {
        "editor": { "guide_description": "Deskripsi panduan", "markdown_editor": "Editor Markdown", "live_preview": "Pratinjau langsung" },
        "workspace": { "copy_title": "Salin kode langkah", "copy_button": "Salin akhir langkah {source} → awal langkah {target}" }
    },
    "it": {
        "editor": { "guide_description": "Descrizione della guida", "markdown_editor": "Editor Markdown", "live_preview": "Anteprima dal vivo" },
        "workspace": { "copy_title": "Copia codice passaggio", "copy_button": "Copia fine passaggio {source} → inizio passaggio {target}" }
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

for (const [locale, data] of Object.entries(absolute_sweep_map)) {
    const filePath = join(dir, `${locale}.json`);
    if (existsSync(filePath)) {
        let content = JSON.parse(readFileSync(filePath, 'utf-8'));
        deepMerge(content, data);
        writeFileSync(filePath, JSON.stringify(content, null, 4) + '\n', 'utf-8');
        console.log(`Final sweep updated ${locale}`);
    }
}
