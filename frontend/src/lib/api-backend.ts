import { browser } from '$app/environment';
import { encryptForBackend, getEncryptionPassword } from './crypto';
import type { Codelab, Step, Attendee, HelpRequest, ChatMessage, Feedback, Material, CertificateInfo, Quiz, QuizSubmissionPayload, QuizSubmissionWithAttendee, Submission, SubmissionWithAttendee } from './types';
export type { Codelab, Step, Attendee, HelpRequest, ChatMessage, Feedback, CertificateInfo, Quiz, QuizSubmissionPayload, QuizSubmissionWithAttendee, Submission, SubmissionWithAttendee };

const envApiUrl = import.meta.env.VITE_API_URL;
let BASE_URL = envApiUrl || 'http://localhost:8080';

if (browser && (envApiUrl === 'http://backend:8080' || !envApiUrl || envApiUrl.includes('localhost'))) {
    // If we are in the browser and the URL is set to the Docker internal name or not set,
    // or pointing to localhost (which won't work from remote), we use the current hostname.
    if (window.location.hostname.includes('ngrok') || window.location.hostname.includes('bore') || window.location.port === '443' || window.location.port === '80') {
        // For tunnel services or standard web ports, use the current origin without port 8080
        // The SvelteKit proxy (hooks.server.ts) will handle forwarding to the backend.
        BASE_URL = window.location.origin;
    } else {
        // Standard local development, keep using port 8080
        BASE_URL = `${window.location.protocol}//${window.location.hostname}:8080`;
    }
}

const API_URL = BASE_URL + '/api';
export const ASSET_URL = BASE_URL;

function getAuthHeader(): Record<string, string> {
    if (!browser) return {};
    const token = localStorage.getItem('adminToken');
    return token ? { 'Authorization': `Bearer ${token}` } : {};
}

export async function listCodelabs(): Promise<Codelab[]> {
    const res = await fetch(`${API_URL}/codelabs`, {
        headers: getAuthHeader()
    });
    if (!res.ok) throw new Error('Failed to fetch codelabs');
    return res.json();
}

export async function getCodelab(id: string): Promise<[Codelab, Step[]]> {
    const res = await fetch(`${API_URL}/codelabs/${id}`, {
        headers: getAuthHeader()
    });
    if (res.status === 403) throw new Error('PRIVATE_CODELAB');
    if (!res.ok) throw new Error('Failed to fetch codelab');
    return res.json();
}

export async function createCodelab(payload: { title: string; description: string; author: string; is_public?: boolean, quiz_enabled?: boolean, require_quiz?: boolean, require_feedback?: boolean, guide_markdown?: string }): Promise<Codelab> {
    const res = await fetch(`${API_URL}/codelabs`, {
        method: 'POST',
        headers: { 
            'Content-Type': 'application/json',
            ...getAuthHeader()
        },
        body: JSON.stringify(payload),
    });
    if (!res.ok) throw new Error('Failed to create codelab');
    return res.json();
}

export async function updateCodelab(id: string, payload: { title: string; description: string; author: string; is_public?: boolean, quiz_enabled?: boolean, require_quiz?: boolean, require_feedback?: boolean, guide_markdown?: string }): Promise<Codelab> {
    const res = await fetch(`${API_URL}/codelabs/${id}`, {
        method: 'PUT',
        headers: { 
            'Content-Type': 'application/json',
            ...getAuthHeader()
        },
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

export async function copyCodelab(id: string): Promise<Codelab> {
    const res = await fetch(`${API_URL}/codelabs/${id}/copy`, {
        method: 'POST',
        headers: getAuthHeader()
    });
    if (!res.ok) throw new Error('Failed to copy codelab');
    return res.json();
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

export async function saveAdminSettings(payload: { gemini_api_key: string }): Promise<void> {
    let finalPayload = { ...payload };
    
    // Encrypt the API key using admin password if available
    if (browser) {
        const adminPw = getEncryptionPassword();
        if (payload.gemini_api_key) {
            if (!adminPw) {
                throw new Error('ENCRYPTION_PASSWORD_MISSING');
            }
            finalPayload.gemini_api_key = encryptForBackend(payload.gemini_api_key, adminPw);
        }
    }

    const res = await fetch(`${API_URL}/admin/settings`, {
        method: 'POST',
        headers: { 
            'Content-Type': 'application/json',
            ...getAuthHeader()
        },
        body: JSON.stringify(finalPayload),
    });
    if (!res.ok) throw new Error('Failed to save settings to server');
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

export async function submitFeedback(codelabId: string, payload: { difficulty: number; satisfaction: number; comments: string; attendee_id: string }): Promise<void> {
    const body = {
        difficulty: payload.difficulty.toString(),
        satisfaction: payload.satisfaction.toString(),
        comment: payload.comments,
        attendee_id: payload.attendee_id
    };

    const res = await fetch(`${API_URL}/codelabs/${codelabId}/feedback`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
    });
    if (res.status === 409) throw new Error('ALREADY_SUBMITTED');
    if (!res.ok) throw new Error('Feedback submission failed');
}

export async function getFeedback(codelabId: string): Promise<Feedback[]> {
    const res = await fetch(`${API_URL}/codelabs/${codelabId}/feedback`);
    if (!res.ok) throw new Error('Failed to fetch feedback');
    return res.json();
}

export async function completeCodelab(codelabId: string, attendeeId: string): Promise<void> {
    const res = await fetch(`${API_URL}/codelabs/${codelabId}/complete`, {
        method: 'POST',
        headers: { 
            'X-Attendee-ID': attendeeId
        },
    });
    if (!res.ok) throw new Error('Failed to complete codelab');
}

export async function getCertificate(attendeeId: string): Promise<CertificateInfo> {
    const res = await fetch(`${API_URL}/certificates/${attendeeId}`);
    if (!res.ok) throw new Error('Failed to fetch certificate');
    return res.json();
}

export function getWsUrl(codelabId: string): string {
    const url = new URL(API_URL.replace('http', 'ws'));
    return `${url.protocol}//${url.host}/api/ws/${codelabId}`;
}

export async function getMaterials(codelabId: string): Promise<Material[]> {
    const res = await fetch(`${API_URL}/codelabs/${codelabId}/materials`);
    if (!res.ok) throw new Error('Failed to fetch materials');
    return res.json();
}

export async function getQuizzes(codelabId: string): Promise<Quiz[]> {
    const res = await fetch(`${API_URL}/codelabs/${codelabId}/quizzes`);
    if (!res.ok) throw new Error('Failed to fetch quizzes');
    return res.json();
}

export async function submitQuiz(codelabId: string, payload: QuizSubmissionPayload): Promise<void> {
    const res = await fetch(`${API_URL}/codelabs/${codelabId}/quizzes/submit`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
    });
    if (!res.ok) throw new Error('Quiz submission failed');
}

export async function getQuizSubmissions(codelabId: string): Promise<QuizSubmissionWithAttendee[]> {
    const res = await fetch(`${API_URL}/codelabs/${codelabId}/quizzes/submissions`, {
        headers: getAuthHeader()
    });
    if (!res.ok) throw new Error('Failed to fetch quiz submissions');
    return res.json();
}

export async function updateQuizzes(codelabId: string, quizzes: { question: string, options: string[], correct_answer: number }[]): Promise<void> {
    const res = await fetch(`${API_URL}/codelabs/${codelabId}/quizzes`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(quizzes)
    });
    if (!res.ok) throw new Error('Failed to update quizzes');
}

export async function addMaterial(codelabId: string, payload: { title: string; material_type: 'link' | 'file'; link_url?: string; file_path?: string }): Promise<Material> {
    const res = await fetch(`${API_URL}/codelabs/${codelabId}/materials`, {
        method: 'POST',
        headers: { 
            'Content-Type': 'application/json',
            ...getAuthHeader()
        },
        body: JSON.stringify(payload),
    });
    if (!res.ok) throw new Error('Failed to add material');
    return res.json();
}

export async function deleteMaterial(codelabId: string, materialId: string): Promise<void> {
    const res = await fetch(`${API_URL}/codelabs/${codelabId}/materials/${materialId}`, {
        method: 'DELETE',
        headers: getAuthHeader()
    });
    if (!res.ok) throw new Error('Failed to delete material');
}

export async function uploadMaterial(file: File): Promise<{ url: string; original_name: string }> {
    const formData = new FormData();
    formData.append('file', file);
    const res = await fetch(`${API_URL}/upload/material`, {
        method: 'POST',
        headers: getAuthHeader(),
        body: formData,
    });
    if (!res.ok) throw new Error('Upload failed');
    return res.json();
}

export async function submitFile(codelabId: string, attendeeId: string, file: File): Promise<Submission> {
    const formData = new FormData();
    formData.append('file', file);
    const res = await fetch(`${API_URL}/codelabs/${codelabId}/attendees/${attendeeId}/submissions`, {
        method: 'POST',
        body: formData,
    });
    if (!res.ok) {
        const error = await res.text();
        throw new Error(error || 'Submission failed');
    }
    return res.json();
}

export async function getSubmissions(codelabId: string): Promise<SubmissionWithAttendee[]> {
    const res = await fetch(`${API_URL}/codelabs/${codelabId}/submissions`, {
        headers: getAuthHeader()
    });
    if (!res.ok) throw new Error('Failed to fetch submissions');
    return res.json();
}

export async function deleteSubmission(codelabId: string, attendeeId: string, submissionId: string): Promise<void> {
    const res = await fetch(`${API_URL}/codelabs/${codelabId}/attendees/${attendeeId}/submissions/${submissionId}`, {
        method: 'DELETE',
        headers: getAuthHeader()
    });
    if (!res.ok) throw new Error('Failed to delete submission');
}
