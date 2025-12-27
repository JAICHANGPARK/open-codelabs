import { browser } from "$app/environment";

export function createTtsPlayer() {
    let synth: SpeechSynthesis | null = null;
    let currentUtterance: SpeechSynthesisUtterance | null = null;

    if (browser) {
        synth = window.speechSynthesis;
    }

    return {
        speak(text: string, lang: string = "en") {
            if (!synth) return;

            this.stop();

            // Strip HTML if present (though the input should ideally be clean text)
            const cleanText = text.replace(/<[^>]*>/g, "");

            currentUtterance = new SpeechSynthesisUtterance(cleanText);
            
            // Map Svelte-i18n locale to BCP 47 language tags
            const langMap: Record<string, string> = {
                "ko": "ko-KR",
                "en": "en-US",
                "ja": "ja-JP",
                "zh": "zh-CN"
            };
            
            currentUtterance.lang = langMap[lang] || lang;
            currentUtterance.rate = 1.0;
            currentUtterance.pitch = 1.0;

            synth.speak(currentUtterance);
        },

        stop() {
            if (synth) {
                synth.cancel();
            }
        },

        pause() {
            if (synth) {
                synth.pause();
            }
        },

        resume() {
            if (synth) {
                synth.resume();
            }
        }
    };
}
