const API_URL = (import.meta.env.VITE_API_URL || 'http://localhost:8080') + '/api';

export interface Codelab {
    id: string;
    title: string;
    description: string;
    author: string;
    created_at?: string;
}

export interface Step {
    id: string;
    codelab_id: string;
    step_number: number;
    title: string;
    content_markdown: string;
}

export async function listCodelabs(): Promise<Codelab[]> {
    const res = await fetch(`${API_URL}/codelabs`);
    if (!res.ok) throw new Error('Failed to fetch codelabs');
    return res.json();
}

export async function getCodelab(id: string): Promise<[Codelab, Step[]]> {
    const res = await fetch(`${API_URL}/codelabs/${id}`);
    if (!res.ok) throw new Error('Failed to fetch codelab');
    return res.json();
}

export async function createCodelab(payload: { title: string; description: string; author: string }): Promise<Codelab> {
    const res = await fetch(`${API_URL}/codelabs`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
    });
    if (!res.ok) throw new Error('Failed to create codelab');
    return res.json();
}
