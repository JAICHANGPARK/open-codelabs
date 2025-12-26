const BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080';
const API_URL = BASE_URL + '/api';
export const ASSET_URL = BASE_URL;

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

export async function saveSteps(codelabId: string, steps: { title: string, content_markdown: string }[]): Promise<void> {
    const res = await fetch(`${API_URL}/codelabs/${codelabId}/steps`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ steps })
    });
    if (!res.ok) throw new Error('Failed to update steps');
}

export async function deleteCodelab(codelabId: string): Promise<void> {
    const res = await fetch(`${API_URL}/codelabs/${codelabId}`, {
        method: 'DELETE'
    });
    if (!res.ok) throw new Error('Failed to delete codelab');
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

export interface Attendee {
    id: string;
    codelab_id: string;
    name: string;
    code: string;
    current_step?: number;
}

export interface HelpRequest {
    id: string;
    codelab_id: string;
    attendee_id: string;
    attendee_name: string;
    step_number: number;
    status: string;
}

export interface ChatMessage {
    id: string;
    codelab_id: string;
    sender_name: string;
    message: string;
    msg_type: 'chat' | 'dm';
    target_id?: string;
    created_at?: string;
}

export async function registerAttendee(codelabId: string, name: string, code: string): Promise<Attendee> {
    const res = await fetch(`${API_URL}/codelabs/${codelabId}/register`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name, code }),
    });
    if (res.status === 409) throw new Error('DUPLICATE_NAME');
    if (!res.ok) throw new Error('Registration failed');
    return res.json();
}

export async function requestHelp(codelabId: string, attendeeId: string, stepNumber: number): Promise<void> {
    const res = await fetch(`${API_URL}/codelabs/${codelabId}/help`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'X-Attendee-ID': attendeeId
        },
        body: JSON.stringify({ step_number: stepNumber }),
    });
    if (!res.ok) throw new Error('Help request failed');
}

export async function getHelpRequests(codelabId: string): Promise<HelpRequest[]> {
    const res = await fetch(`${API_URL}/codelabs/${codelabId}/help`);
    if (!res.ok) throw new Error('Failed to fetch help requests');
    return res.json();
}

export async function resolveHelpRequest(codelabId: string, helpId: string): Promise<void> {
    const res = await fetch(`${API_URL}/codelabs/${codelabId}/help/${helpId}/resolve`, {
        method: 'POST',
    });
    if (!res.ok) throw new Error('Failed to resolve help request');
}

export async function getAttendees(codelabId: string): Promise<Attendee[]> {
    const res = await fetch(`${API_URL}/codelabs/${codelabId}/attendees`);
    if (!res.ok) throw new Error('Failed to fetch attendees');
    return res.json();
}

export async function getChatHistory(codelabId: string): Promise<ChatMessage[]> {
    const res = await fetch(`${API_URL}/codelabs/${codelabId}/chat`);
    if (!res.ok) throw new Error('Failed to fetch chat history');
    return res.json();
}

export async function uploadImage(file: File): Promise<{ url: string }> {
    const formData = new FormData();
    formData.append('file', file);
    const res = await fetch(`${API_URL}/upload/image`, {
        method: 'POST',
        body: formData,
    });
    if (!res.ok) throw new Error('Upload failed');
    return res.json();
}

export interface Feedback {
    id: string;
    codelab_id: string;
    difficulty: number;
    satisfaction: number;
    comments?: string;
    created_at?: string;
}

export async function submitFeedback(codelabId: string, payload: { difficulty: number; satisfaction: number; comments: string }): Promise<void> {
    const res = await fetch(`${API_URL}/codelabs/${codelabId}/feedback`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
    });
    if (!res.ok) throw new Error('Feedback submission failed');
}

export async function getFeedback(codelabId: string): Promise<Feedback[]> {
    const res = await fetch(`${API_URL}/codelabs/${codelabId}/feedback`);
    if (!res.ok) throw new Error('Failed to fetch feedback');
    return res.json();
}

export function getWsUrl(codelabId: string): string {
    const url = new URL(API_URL.replace('http', 'ws'));
    return `${url.protocol}//${url.host}/api/ws/${codelabId}`;
}
