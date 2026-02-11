
import { readFileSync, writeFileSync, existsSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const dir = join(__dirname, '..', 'src', 'lib', 'i18n');

const updates = {
    "id": {
        "common": { "status": "Status", "optional": "Opsional" },
        "editor": {
            "tab_groups": { "authoring": "Penulisan", "operations": "Operasi", "assessment": "Penilaian" },
            "snippets": { "step_outline": "## Tujuan\n- \n\n## Prasyarat\n- \n\n## Langkah\n1. \n2. \n\n## Pos Pemeriksaan\n- ", "checklist": "- [ ] Tugas\n- [ ] Tugas", "callout": "> **Catatan**\n> ", "command_block": "```bash\n# Perintah\n```\n\nOutput:\n```\n\n```" }
        },
        "workspace": { "ai_phase_start": "Mulai", "ai_phase_end": "Selesai" },
        "ai_generator": { "title": "Generator AI", "generate_button": "Buat", "cancel_button": "Batal" }
    },
    "it": {
        "common": { "status": "Stato", "optional": "Opzionale" },
        "editor": {
            "tab_groups": { "authoring": "Scrittura", "operations": "Operazioni", "assessment": "Valutazione" },
            "snippets": { "step_outline": "## Obiettivi\n- \n\n## Prerequisiti\n- \n\n## Passaggi\n1. \n2. \n\n## Punto di controllo\n- ", "checklist": "- [ ] Attività\n- [ ] Attività", "callout": "> **Nota**\n> ", "command_block": "```bash\n# Comando\n```\n\nOutput:\n```\n\n```" }
        },
        "workspace": { "ai_phase_start": "Inizio", "ai_phase_end": "Fine" },
        "ai_generator": { "title": "Generatore AI", "generate_button": "Genera", "cancel_button": "Annulla" }
    },
    "es": {
        "common": { "status": "Estado", "optional": "Opcional" },
        "editor": {
            "tab_groups": { "authoring": "Autoría", "operations": "Operaciones", "assessment": "Evaluación" },
            "snippets": { "step_outline": "## Objetivos\n- \n\n## Requisitos previos\n- \n\n## Pasos\n1. \n2. \n\n## Punto de control\n- ", "checklist": "- [ ] Tarea\n- [ ] Tarea", "callout": "> **Nota**\n> ", "command_block": "```bash\n# Comando\n```\n\nResultado:\n```\n\n```" }
        },
        "workspace": { "ai_phase_start": "Inicio", "ai_phase_end": "Fin" },
        "ai_generator": { "title": "Generador AI", "generate_button": "Generar", "cancel_button": "Cancelar" }
    },
    "fr": {
        "common": { "status": "Statut", "optional": "Optionnel", "actions": "Actions", "documentation": "Documentation", "conversations": "Conversations", "question": "Question", "public": "Public" },
        "editor": {
            "tab_groups": { "authoring": "Rédaction", "operations": "Opérations", "assessment": "Évaluation" },
            "snippets": { "step_outline": "## Objectifs\n- \n\n## Prérequis\n- \n\n## Étapes\n1. \n2. \n\n## Point de contrôle\n- ", "checklist": "- [ ] Tâche\n- [ ] Tâche", "callout": "> **Note**\n> ", "command_block": "```bash\n# Commande\n```\n\nSortie :\n```\n\n```" }
        },
        "workspace": { "ai_phase_start": "Début", "ai_phase_end": "Fin" },
        "ai_generator": { "title": "Générateur AI", "generate_button": "Générer", "cancel_button": "Annuler" }
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
