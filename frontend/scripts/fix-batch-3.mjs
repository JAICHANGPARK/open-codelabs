
import { readFileSync, writeFileSync, existsSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const dir = join(__dirname, '..', 'src', 'lib', 'i18n');

const updates = {
    "ar": {
        "common": { "title": "أوبن كودلاب (Open Codelabs)", "status": "الحالة", "optional": "اختياري" },
        "editor": {
            "tab_groups": { "authoring": "التأليف", "operations": "العمليات", "assessment": "التقييم" },
            "snippets": { "step_outline": "## الأهداف\n- \n\n## المتطلبات الأساسية\n- \n\n## الخطوات\n1. \n2. \n\n## نقطة تحقق\n- ", "checklist": "- [ ] مهمة\n- [ ] مهمة", "callout": "> **ملاحظة**\n> ", "command_block": "```bash\n# أمر\n```\n\nالمخرجات:\n```\n\n```" }
        },
        "workspace": { "ai_phase_start": "بدء", "ai_phase_end": "نهاية" },
        "ai_generator": { "title": "مولد الذكاء الاصطناعي", "generate_button": "توليد", "cancel_button": "إلغاء" }
    },
    "fa": {
        "common": { "title": "اوپن کدلب (Open Codelabs)", "status": "وضعیت", "optional": "اختیاری" },
        "editor": {
            "tab_groups": { "authoring": "تألیف", "operations": "عملیات", "assessment": "ارزیابی" },
            "snippets": { "step_outline": "## اهداف\n- \n\n## پیش‌نیازها\n- \n\n## مراحل\n1. \n2. \n\n## نقطه بازرسی\n- ", "checklist": "- [ ] وظیفه\n- [ ] وظیفه", "callout": "> **نکته**\n> ", "command_block": "```bash\n# دستور\n```\n\nخروجی:\n```\n\n```" }
        },
        "workspace": { "ai_phase_start": "شروع", "ai_phase_end": "پایان" },
        "ai_generator": { "title": "مولد هوش مصنوعی", "generate_button": "تولید", "cancel_button": "لغو" }
    },
    "he": {
        "common": { "title": "Open Codelabs (אופן קודלבס)", "status": "סטטוס", "optional": "אופציונלי" },
        "editor": {
            "tab_groups": { "authoring": "כתיבה", "operations": "פעולות", "assessment": "הערכה" },
            "snippets": { "step_outline": "## יעדים\n- \n\n## דרישות קדם\n- \n\n## שלבים\n1. \n2. \n\n## נקודת ביקורת\n- ", "checklist": "- [ ] משימה\n- [ ] משימה", "callout": "> **הערה**\n> ", "command_block": "```bash\n# פקודה\n```\n\nפלט:\n```\n\n```" }
        },
        "workspace": { "ai_phase_start": "התחלה", "ai_phase_end": "סיום" },
        "ai_generator": { "title": "מחולל AI", "generate_button": "צור", "cancel_button": "ביטול" }
    },
    "hi": {
        "common": { "title": "ओपन-कोडलैब्स", "status": "स्थिति", "optional": "वैकल्पिक" },
        "editor": {
            "tab_groups": { "authoring": "लेखन", "operations": "संचालन", "assessment": "मूल्यांकन" },
            "snippets": { "step_outline": "## उद्देश्य\n- \n\n## पूर्वापेक्षाएँ\n- \n\n## चरण\n1. \n2. \n\n## चेकपॉइंट\n- ", "checklist": "- [ ] कार्य\n- [ ] कार्य", "callout": "> **नोट**\n> ", "command_block": "```bash\n# कमांड\n```\n\nआउटपुट:\n```\n\n```" }
        },
        "workspace": { "ai_phase_start": "प्रारंभ", "ai_phase_end": "अंत" },
        "ai_generator": { "title": "AI जनरेटर", "generate_button": "जेनरेट करें", "cancel_button": "रद्द करें" }
    },
    "th": {
        "common": { "title": "Open Codelabs", "status": "สถานะ", "optional": "ไม่บังคับ" },
        "editor": {
            "tab_groups": { "authoring": "การเขียน", "operations": "การดำเนินงาน", "assessment": "การประเมิน" },
            "snippets": { "step_outline": "## วัตถุประสงค์\n- \n\n## ข้อกำหนดเบื้องต้น\n- \n\n## ขั้นตอน\n1. \n2. \n\n## จุดตรวจสอบ\n- ", "checklist": "- [ ] งาน\n- [ ] งาน", "callout": "> **หมายเหตุ**\n> ", "command_block": "```bash\n# คำสั่ง\n```\n\nผลลัพธ์:\n```\n\n```" }
        },
        "workspace": { "ai_phase_start": "เริ่ม", "ai_phase_end": "จบ" },
        "ai_generator": { "title": "ตัวสร้าง AI", "generate_button": "สร้าง", "cancel_button": "ยกเลิก" }
    },
    "tr": {
        "common": { "title": "Open Codelabs", "status": "Durum", "optional": "İsteğe bağlı" },
        "editor": {
            "tab_groups": { "authoring": "Yazma", "operations": "İşlemler", "assessment": "Değerlendirme" },
            "snippets": { "step_outline": "## Hedefler\n- \n\n## Önkoşullar\n- \n\n## Adımlar\n1. \n2. \n\n## Kontrol Noktası\n- ", "checklist": "- [ ] Görev\n- [ ] Görev", "callout": "> **Not**\n> ", "command_block": "```bash\n# Komut\n```\n\nÇıktı:\n```\n\n```" }
        },
        "workspace": { "ai_phase_start": "Başla", "ai_phase_end": "Bitir" },
        "ai_generator": { "title": "AI Oluşturucu", "generate_button": "Oluştur", "cancel_button": "İptal" }
    },
    "vi": {
        "common": { "title": "Open Codelabs", "status": "Trạng thái", "optional": "Tùy chọn" },
        "editor": {
            "tab_groups": { "authoring": "Soạn thảo", "operations": "Vận hành", "assessment": "Đánh giá" },
            "snippets": { "step_outline": "## Mục tiêu\n- \n\n## Tiền đề\n- \n\n## Các bước\n1. \n2. \n\n## Điểm kiểm tra\n- ", "checklist": "- [ ] Nhiệm vụ\n- [ ] Nhiệm vụ", "callout": "> **Ghi chú**\n> ", "command_block": "```bash\n# Lệnh\n```\n\nKết quả:\n```\n\n```" }
        },
        "workspace": { "ai_phase_start": "Bắt đầu", "ai_phase_end": "Kết thúc" },
        "ai_generator": { "title": "Trình tạo AI", "generate_button": "Tạo", "cancel_button": "Hủy" }
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

for (const [locale, data] of Object.entries(updates)) {
    const filePath = join(dir, `${locale}.json`);
    if (existsSync(filePath)) {
        let content = JSON.parse(readFileSync(filePath, 'utf-8'));
        deepMerge(content, data);
        writeFileSync(filePath, JSON.stringify(content, null, 4) + '\n', 'utf-8');
        console.log(`Updated ${locale}`);
    }
}
