
import { readFileSync, writeFileSync, existsSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const dir = join(__dirname, '..', 'src', 'lib', 'i18n');

const updates = {
    "ko": {
        "common": { "title": "오픈 코드랩 (Open Codelabs)" },
        "editor": { "material_placeholder_link": "재료 URL (예: 보충 자료)" },
        "attendee": { "email_placeholder": "이메일 주소 (선택 사항)" },
        "submission_panel": { "size_label": "크기" },
        "certificate": { "subtitle": "실습 프로젝트를 성공적으로 이수하였음을 증명합니다" }
    },
    "zh-TW": {
        "editor": { "material_placeholder_link": "物料 URL (例如：補充教材)" },
        "workspace": { "notebook": { "markdown_label": "Markdown" } },
        "attendee": { "email_placeholder": "電子郵件地址 (選填)" },
        "profile": { "codelab": "程式碼實驗室" },
        "submission_panel": { "size_label": "大小" }
    },
    "zh": {
        "editor": {
            "guide_pro_view_markdown": "查看 Markdown",
            "material_placeholder_link": "物料 URL (例如：補充教材)"
        },
        "workspace": {
            "notebook": { "markdown_label": "Markdown" },
            "ai_response_title": "AI 響應",
            "ai_response_step": "步驟 {step}: {title}"
        },
        "attendee": { "email_placeholder": "電子郵件地址 (選填)" },
        "submission_panel": { "size_label": "大小" },
        "ai_generator": {
            "view_markdown": "查看 Markdown",
            "upload_files": "上傳檔案",
            "duration_label": "時長",
            "duration_mins": "分鐘",
            "duration_custom": "自定義"
        },
        "audit": {
            "filter_codelab_id": "實驗室 ID",
            "col_target_codelab": "目標實驗室"
        }
    },
    "ja": {
        "editor": {
            "quiz_type": "クイズ形式",
            "multiple_choice": "選択肢形式",
            "descriptive": "記述式",
            "descriptive_hint": "答えのヒントを入力してください",
            "add_option": "選択肢を追加",
            "guide_pro_view_markdown": "Markdownを表示",
            "material_placeholder_link": "教材URL (例: 補足資料)"
        },
        "workspace": {
            "notebook": { "markdown_label": "Markdown" },
            "ai_response_title": "AIの回答",
            "ai_response_step": "ステップ {step}: {title}"
        },
        "ai_generator": {
            "view_markdown": "Markdownを表示",
            "upload_files": "ファイルをアップロード",
            "duration_label": "所要時間",
            "duration_mins": "分",
            "duration_custom": "カスタム"
        },
        "audit": {
            "filter_codelab_id": "コードラボID",
            "col_target_codelab": "ターゲットラボ"
        },
        "attendee": { "email_placeholder": "メールアドレス (任意)" },
        "submission_panel": { "size_label": "サイズ" }
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
