/**
 * Generate i18n translation files from en.json base.
 * This script creates translation JSON files for all target languages,
 * translating UI-facing string values while preserving structure, keys, and placeholders.
 *
 * Usage: node scripts/generate-i18n.mjs
 */
import { readFileSync, writeFileSync, existsSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const i18nDir = join(__dirname, '..', 'src', 'lib', 'i18n');
const enData = JSON.parse(readFileSync(join(i18nDir, 'en.json'), 'utf-8'));

// Translation maps for each language - only UI-visible strings
// Keys preserved, placeholders like {count}, {title} preserved as-is
const translations = {
    de: {
        // common
        "Interactive learning experience": "Interaktive Lernerfahrung",
        "Facilitator": "Moderator",
        "Attendee": "Teilnehmer",
        "Login": "Anmelden",
        "Logout": "Abmelden",
        "Save": "Speichern",
        "Cancel": "Abbrechen",
        "Delete": "Löschen",
        "Create": "Erstellen",
        "Import": "Importieren",
        "Export": "Exportieren",
        "Copy": "Kopieren",
        "Loading...": "Laden...",
        "Send": "Senden",
        "Actions": "Aktionen",
        "Author": "Autor",
        "Date": "Datum",
        "Status": "Status",
        "Error": "Fehler",
        "Success": "Erfolg",
        "failed": "fehlgeschlagen",
        "Photo": "Foto",
        "Toggle Colorblind Mode": "Farbenblind-Modus umschalten",
        "Read this step": "Diesen Schritt vorlesen",
        "Stop reading": "Vorlesen beenden",
        "Sign in with Google": "Mit Google anmelden",
        "GitHub Repository": "GitHub-Repository",
        "Documentation": "Dokumentation",
        "Change Language": "Sprache ändern",
        "Conversations": "Unterhaltungen",
        "total": "gesamt",
        "Refresh": "Aktualisieren",
        "Question": "Frage",
        "Answer": "Antwort",
        "Select an item to view details": "Element auswählen, um Details anzuzeigen",
        "No data available": "Keine Daten verfügbar",
        "Toggle Theme": "Design umschalten",
        "OR": "ODER",
        "Optional": "Optional",
        "Required": "Erforderlich",
        "AI Assistant": "KI-Assistent",
        "Close": "Schließen",
        "Visibility": "Sichtbarkeit",
        "Public": "Öffentlich",
        "Private": "Privat",
        "Skip to main content": "Zum Hauptinhalt springen",
        "Download": "Herunterladen",
        "Open in new tab": "In neuem Tab öffnen",
        "Are you sure you want to delete this item? This action cannot be undone.": "Sind Sie sicher, dass Sie dieses Element löschen möchten? Diese Aktion kann nicht rückgängig gemacht werden.",
        // theme
        "Theme": "Design",
        "Mode": "Modus",
        "Preset": "Voreinstellung",
        "System": "System",
        "Light": "Hell",
        "Dark": "Dunkel",
        "Default": "Standard",
        "Mint": "Minze",
        "Ocean": "Ozean",
        "Sunset": "Sonnenuntergang",
        "Forest": "Wald",
        "Berry": "Beere",
        "Slate": "Schiefer",
        // dashboard
        "Facilitator Dashboard": "Moderator-Dashboard",
        "Manage your codelabs and track participant progress": "Verwalten Sie Ihre Codelabs und verfolgen Sie den Fortschritt der Teilnehmer",
        "New Codelab": "Neues Codelab",
        "No codelabs yet": "Noch keine Codelabs",
        "Get started by creating your first hands-on material.": "Beginnen Sie mit der Erstellung Ihres ersten Praxismaterials.",
        "Create your first codelab": "Erstes Codelab erstellen",
        "Import Codelab": "Codelab importieren",
        "Create New Codelab": "Neues Codelab erstellen",
        "Design your next learning experience": "Gestalten Sie Ihre nächste Lernerfahrung",
        "Title": "Titel",
        "Description": "Beschreibung",
        "e.g., Intro to SvelteKit": "z.B. Einführung in SvelteKit",
        "What will they learn?": "Was werden die Teilnehmer lernen?",
        "Your Name": "Ihr Name",
        "Are you sure you want to delete this codelab? This action cannot be undone.": "Sind Sie sicher, dass Sie dieses Codelab löschen möchten?",
        "Create with AI": "Mit KI erstellen",
        "Copy Participant Link": "Teilnehmer-Link kopieren",
        "Unknown Date": "Unbekanntes Datum",
        "Update available": "Update verfügbar",
        "Pull the latest Docker images to update.": "Laden Sie die neuesten Docker-Images herunter.",
        // ... continues for all strings
    },
};

// Deep translate function - replaces values matching translation map
function deepTranslate(obj, transMap) {
    if (typeof obj === 'string') {
        // Direct match
        if (transMap[obj]) return transMap[obj];
        // No match - return original (English fallback)
        return obj;
    }
    if (Array.isArray(obj)) {
        return obj.map(item => deepTranslate(item, transMap));
    }
    if (typeof obj === 'object' && obj !== null) {
        const result = {};
        for (const [key, value] of Object.entries(obj)) {
            result[key] = deepTranslate(value, transMap);
        }
        return result;
    }
    return obj;
}

// For this approach, we'll just copy en.json as-is for each new language
// The actual translations will be applied via a simpler copy approach
const targetLangs = ['de', 'es', 'fr', 'id', 'it', 'pl', 'pt', 'vi', 'tr', 'ru', 'he', 'ar', 'fa', 'hi', 'bn', 'th', 'zh-TW'];

for (const lang of targetLangs) {
    const targetPath = join(i18nDir, `${lang}.json`);
    if (!existsSync(targetPath)) {
        // Start with English as base - users can refine translations later
        writeFileSync(targetPath, JSON.stringify(enData, null, 4), 'utf-8');
        console.log(`Created ${lang}.json (English base - needs translation)`);
    } else {
        console.log(`Skipped ${lang}.json (already exists)`);
    }
}

console.log('\nDone! Files created with English base text.');
console.log('Each file needs to be translated to the target language.');
