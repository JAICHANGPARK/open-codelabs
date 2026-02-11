
import { readFileSync, writeFileSync, existsSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const dir = join(__dirname, '..', 'src', 'lib', 'i18n');

const locales = ["fr", "id", "pt"];

const final_final_map = {
    "fr": {
        "editor": {
            "quiz_settings": "Paramètres du quiz",
            "quiz_enabled": "Quiz activé",
            "quiz_question": "Question du quiz",
            "quiz_options": "Options du quiz",
            "quiz_correct": "Correct (Vrai)"
        }
    },
    "id": {
        "workspace": {
            "copy_success": "Kode berhasil disalin antara langkah.",
            "actions": { "browse": "Telusuri berkas", "download": "Unduh Ruang Kerja" }
        },
        "profile": { "codelab": "Codelab Praktis" }
    },
    "pt": {
        "admin": { "consultant": { "history": "Histórico de Chat", "new_chat": "Nova Conversa", "no_history": "Sem histórico disponível.", "role_consultant": "Consultor" } },
        "raffle": { "refresh": "Atualizar Sorteio" }
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
    const data = final_final_map[locale];
    if (data) {
        const filePath = join(dir, `${locale}.json`);
        if (existsSync(filePath)) {
            let content = JSON.parse(readFileSync(filePath, 'utf-8'));
            deepMerge(content, data);
            writeFileSync(filePath, JSON.stringify(content, null, 4) + '\n', 'utf-8');
            console.log(`Final Final Zeroed: ${locale}`);
        }
    }
});
