export function saveProgress(codelabId: string, stepIndex: number): void {
    if (typeof localStorage !== 'undefined') {
        localStorage.setItem(`progress_${codelabId}`, stepIndex.toString());
    }
}

export function loadProgress(codelabId: string): number {
    if (typeof localStorage !== 'undefined') {
        const saved = localStorage.getItem(`progress_${codelabId}`);
        return saved ? parseInt(saved, 10) : 0;
    }
    return 0;
}
