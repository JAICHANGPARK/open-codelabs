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

export async function updateCodelab(id: string, payload: { title: string; description: string; author: string }): Promise<Codelab> {
    const res = await fetch(`${API_URL}/codelabs/${id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
    });
    if (!res.ok) throw new Error('Failed to update codelab');
    return res.json();
}

export async function saveSteps(id: string, steps: { title: string; content_markdown: string }[]): Promise<void> {
    const res = await fetch(`${API_URL}/codelabs/${id}/steps`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ steps }),
    });
    if (!res.ok) throw new Error('Failed to save steps');
}

export async function login(admin_id: string, admin_pw: string): Promise<{ token: string }> {
    const res = await fetch(`${API_URL}/login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ admin_id, admin_pw }),
    });
    if (!res.ok) throw new Error('Invalid credentials');
    return res.json();
}

export async function exportCodelab(id: string): Promise<void> {
    const res = await fetch(`${API_URL}/codelabs/${id}/export`);
    if (!res.ok) throw new Error('Export failed');
    const blob = await res.blob();
    const url = window.URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `codelab_${id}.zip`;
    document.body.appendChild(a);
    a.click();
    window.URL.revokeObjectURL(url);
}

export async function importCodelab(file: File): Promise<Codelab> {
    const formData = new FormData();
    formData.append('file', file);
    const res = await fetch(`${API_URL}/codelabs/import`, {
        method: 'POST',
        body: formData,
    });
    if (!res.ok) throw new Error('Import failed');
    return res.json();
}
