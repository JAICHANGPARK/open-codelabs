import { browser } from '$app/environment';
import { encryptForBackend, getEncryptionPassword } from './crypto';
import type { Codelab, Step, Attendee, HelpRequest, ChatMessage, Feedback, Material, CertificateInfo, Quiz, QuizSubmissionPayload, QuizSubmissionWithAttendee, Submission, SubmissionWithAttendee, AiConversation, SaveAiConversationPayload, InlineCommentThread, CreateInlineCommentPayload } from './types';
export type { Codelab, Step, Attendee, HelpRequest, ChatMessage, Feedback, CertificateInfo, Quiz, QuizSubmissionPayload, QuizSubmissionWithAttendee, Submission, SubmissionWithAttendee, AiConversation, SaveAiConversationPayload, InlineCommentThread, CreateInlineCommentPayload };

const envApiUrl = import.meta.env.VITE_API_URL;
let BASE_URL = envApiUrl || 'http://localhost:8080';

if (browser && (envApiUrl === 'http://backend:8080' || !envApiUrl || envApiUrl.includes('localhost'))) {
    // If we are in the browser and the URL is set to the Docker internal name or not set,
    // or pointing to localhost (which won't work from remote), we use the current hostname.
    const hostname = window.location.hostname;
    const isTunnelHost = hostname.includes('ngrok') || hostname.includes('bore') || hostname.includes('trycloudflare.com');
    const isDefaultPort = window.location.port === '' || window.location.port === '443' || window.location.port === '80';
    const isLocalhost = hostname === 'localhost' || hostname === '127.0.0.1' || hostname === '::1';

    if (isTunnelHost || (!isLocalhost && isDefaultPort)) {
        // For tunnel services or default web ports, use the current origin without port 8080
        // The SvelteKit proxy (hooks.server.ts) will handle forwarding to the backend.
        BASE_URL = window.location.origin;
    } else {
        // Standard local development, keep using port 8080
        BASE_URL = `${window.location.protocol}//${window.location.hostname}:8080`;
    }
}

const API_URL = BASE_URL + '/api';
export const ASSET_URL = BASE_URL;

function getCookie(name: string): string | null {
    if (!browser) return null;
    const match = document.cookie.match(new RegExp(`(?:^|; )${name}=([^;]*)`));
    return match ? decodeURIComponent(match[1]) : null;
}

function getCsrfToken(): string | null {
    return getCookie("__Host-oc_csrf") || getCookie("oc_csrf");
}

function withCsrf(headers?: HeadersInit, method?: string): Headers {
    const merged = new Headers(headers || {});
    const verb = (method || "GET").toUpperCase();
    if (!["GET", "HEAD", "OPTIONS"].includes(verb)) {
        const token = getCsrfToken();
        if (token) merged.set("X-CSRF-Token", token);
    }
    return merged;
}

async function apiFetch(path: string, init: RequestInit = {}): Promise<Response> {
    const method = init.method || "GET";
    const headers = withCsrf(init.headers, method);
    return fetch(`${API_URL}${path}`, {
        ...init,
        credentials: "include",
        headers
    });
}

export async function listCodelabs(): Promise<Codelab[]> {
    const res = await apiFetch(`/codelabs`);
    if (!res.ok) throw new Error('Failed to fetch codelabs');
    return res.json();
}

export async function getCodelab(id: string): Promise<[Codelab, Step[]]> {
    const res = await apiFetch(`/codelabs/${id}`);
    if (res.status === 403) throw new Error('PRIVATE_CODELAB');
    if (!res.ok) throw new Error('Failed to fetch codelab');
    return res.json();
}

export async function createCodelab(payload: { title: string; description: string; author: string; is_public?: boolean, quiz_enabled?: boolean, require_quiz?: boolean, require_feedback?: boolean, guide_markdown?: string }): Promise<Codelab> {
    const res = await apiFetch(`/codelabs`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(payload),
    });
    if (!res.ok) throw new Error('Failed to create codelab');
    return res.json();
}

export async function updateCodelab(id: string, payload: { title: string; description: string; author: string; is_public?: boolean, quiz_enabled?: boolean, require_quiz?: boolean, require_feedback?: boolean, guide_markdown?: string }): Promise<Codelab> {
    const res = await apiFetch(`/codelabs/${id}`, {
        method: 'PUT',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(payload),
    });
    if (!res.ok) throw new Error('Failed to update codelab');
    return res.json();
}

export async function saveSteps(
    codelabId: string,
    steps: { id?: string; title: string; content_markdown: string }[],
): Promise<void> {
    const res = await apiFetch(`/codelabs/${codelabId}/steps`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ steps })
    });
    if (!res.ok) throw new Error('Failed to update steps');
}

export async function deleteCodelab(codelabId: string): Promise<void> {
    const res = await apiFetch(`/codelabs/${codelabId}`, {
        method: 'DELETE'
    });
    if (!res.ok) throw new Error('Failed to delete codelab');
}

export async function copyCodelab(id: string): Promise<Codelab> {
    const res = await apiFetch(`/codelabs/${id}/copy`, { method: 'POST' });
    if (!res.ok) throw new Error('Failed to copy codelab');
    return res.json();
}

export async function login(admin_id: string, admin_pw: string): Promise<{ status: string; token?: string }> {
    const res = await apiFetch(`/login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ admin_id, admin_pw }),
    });
    if (!res.ok) throw new Error('Invalid credentials');
    return res.json();
}

export async function logout(): Promise<void> {
    const res = await apiFetch(`/logout`, { method: 'POST' });
    if (!res.ok) throw new Error('Logout failed');
}

export async function getSession(): Promise<{ role: string; sub: string; exp: number; codelab_id?: string | null } | null> {
    const res = await apiFetch(`/session`);
    if (res.status === 401) return null;
    if (!res.ok) throw new Error('Failed to fetch session');
    return res.json();
}

export interface AuditLog {
    id: string;
    action: string;
    actor_type: string;
    actor_id?: string;
    target_id?: string;
    codelab_id?: string;
    ip?: string;
    user_agent?: string;
    metadata?: string;
    created_at: string;
}

export async function getAuditLogs(params?: {
    limit?: number;
    offset?: number;
    codelab_id?: string;
    action?: string;
}): Promise<AuditLog[]> {
    const searchParams = new URLSearchParams();
    if (params?.limit) searchParams.append('limit', params.limit.toString());
    if (params?.offset) searchParams.append('offset', params.offset.toString());
    if (params?.codelab_id) searchParams.append('codelab_id', params.codelab_id);
    if (params?.action) searchParams.append('action', params.action);

    const res = await apiFetch(`/admin/audit-logs?${searchParams.toString()}`);
    if (!res.ok) throw new Error('Failed to fetch audit logs');
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

    const res = await apiFetch(`/admin/settings`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(finalPayload),
    });
    if (!res.ok) throw new Error('Failed to save settings to server');
}

export async function exportCodelab(id: string): Promise<void> {
    const res = await apiFetch(`/codelabs/${id}/export`);
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
    const res = await apiFetch(`/codelabs/import`, {
        method: 'POST',
        body: formData,
    });
    if (!res.ok) throw new Error('Import failed');
    return res.json();
}

export async function exportBackup(): Promise<void> {
    const res = await apiFetch(`/admin/backup/export`);
    if (!res.ok) throw new Error('Backup export failed');
    const blob = await res.blob();
    const url = window.URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `backup_full.zip`;
    document.body.appendChild(a);
    a.click();
    window.URL.revokeObjectURL(url);
}

export async function restoreBackup(file: File): Promise<void> {
    const formData = new FormData();
    formData.append('file', file);
    const res = await apiFetch(`/admin/backup/restore`, {
        method: 'POST',
        body: formData,
    });
    if (!res.ok) {
        const text = await res.text().catch(() => '');
        throw new Error(text ? `Backup restore failed: ${text}` : 'Backup restore failed');
    }
}

export async function inspectBackup(file: File): Promise<{
    version: number;
    created_at: string;
    codelabs: number;
    steps: number;
    attendees: number;
    help_requests: number;
    chat_messages: number;
    feedback: number;
    materials: number;
    quizzes: number;
    quiz_submissions: number;
    submissions: number;
    audit_logs: number;
    codeserver_workspaces: number;
    ai_conversations: number;
    ai_threads: number;
    ai_messages: number;
    uploads_files: number;
    workspaces_files: number;
}> {
    const formData = new FormData();
    formData.append('file', file);
    const res = await apiFetch(`/admin/backup/inspect`, {
        method: 'POST',
        body: formData,
    });
    if (!res.ok) {
        const text = await res.text().catch(() => '');
        throw new Error(text ? `Backup inspect failed: ${text}` : 'Backup inspect failed');
    }
    return res.json();
}

export async function registerAttendee(codelabId: string, name: string, code: string, email?: string): Promise<Attendee> {
    const res = await apiFetch(`/codelabs/${codelabId}/register`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name, code, email }),
    });
    if (res.status === 409) throw new Error('DUPLICATE_NAME');
    if (!res.ok) {
        const text = await res.text().catch(() => '');
        throw new Error(text || 'Registration failed');
    }
    return res.json();
}

export async function requestHelp(codelabId: string, stepNumber: number): Promise<void> {
    const res = await apiFetch(`/codelabs/${codelabId}/help`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({ step_number: stepNumber }),
    });
    if (!res.ok) {
        const errorText = await res.text().catch(() => 'Unknown error');
        throw new Error(`Help request failed (${res.status}): ${errorText}`);
    }
}

export async function getHelpRequests(codelabId: string): Promise<HelpRequest[]> {
    const res = await apiFetch(`/codelabs/${codelabId}/help`);
    if (!res.ok) throw new Error('Failed to fetch help requests');
    return res.json();
}

export async function resolveHelpRequest(codelabId: string, helpId: string): Promise<void> {
    const res = await apiFetch(`/codelabs/${codelabId}/help/${helpId}/resolve`, { method: 'POST' });
    if (!res.ok) throw new Error('Failed to resolve help request');
}

export async function getAttendees(codelabId: string): Promise<Attendee[]> {
    const res = await apiFetch(`/codelabs/${codelabId}/attendees`);
    if (!res.ok) throw new Error('Failed to fetch attendees');
    return res.json();
}

export async function getChatHistory(codelabId: string): Promise<ChatMessage[]> {
    const res = await apiFetch(`/codelabs/${codelabId}/chat`);
    if (!res.ok) throw new Error('Failed to fetch chat history');
    return res.json();
}

export async function getInlineComments(
    codelabId: string,
    params?: { target_type?: "step" | "guide"; target_step_id?: string },
): Promise<InlineCommentThread[]> {
    const searchParams = new URLSearchParams();
    if (params?.target_type) searchParams.set("target_type", params.target_type);
    if (params?.target_step_id) searchParams.set("target_step_id", params.target_step_id);
    const query = searchParams.toString();
    const res = await apiFetch(
        `/codelabs/${codelabId}/inline-comments${query ? `?${query}` : ""}`,
    );
    if (!res.ok) throw new Error("Failed to fetch inline comments");
    return res.json();
}

export async function createInlineComment(
    codelabId: string,
    payload: CreateInlineCommentPayload,
): Promise<InlineCommentThread> {
    const res = await apiFetch(`/codelabs/${codelabId}/inline-comments`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
    });
    if (!res.ok) {
        const text = await res.text().catch(() => "");
        throw new Error(text || "Failed to create inline comment");
    }
    return res.json();
}

export async function replyInlineComment(
    codelabId: string,
    threadId: string,
    payload: { message: string; content_hash: string },
): Promise<InlineCommentThread> {
    const res = await apiFetch(
        `/codelabs/${codelabId}/inline-comments/${threadId}/comments`,
        {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(payload),
        },
    );
    if (!res.ok) {
        const text = await res.text().catch(() => "");
        throw new Error(text || "Failed to reply to inline comment");
    }
    return res.json();
}

export async function deleteInlineComment(
    codelabId: string,
    threadId: string,
    commentId: string,
): Promise<void> {
    const res = await apiFetch(
        `/codelabs/${codelabId}/inline-comments/${threadId}/comments/${commentId}`,
        { method: "DELETE" },
    );
    if (!res.ok) {
        const text = await res.text().catch(() => "");
        throw new Error(text || "Failed to delete inline comment");
    }
}

export async function uploadImage(file: File): Promise<{ url: string }> {
    const formData = new FormData();
    formData.append('file', file);
    const res = await apiFetch(`/upload/image`, {
        method: 'POST',
        body: formData,
    });
    if (!res.ok) {
        const text = await res.text().catch(() => '');
        throw new Error(text ? `Upload failed: ${text}` : 'Upload failed');
    }
    return res.json();
}

export async function submitFeedback(codelabId: string, payload: { difficulty: number; satisfaction: number; comments: string }): Promise<void> {
    const body = {
        difficulty: payload.difficulty.toString(),
        satisfaction: payload.satisfaction.toString(),
        comment: payload.comments
    };

    const res = await apiFetch(`/codelabs/${codelabId}/feedback`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
    });
    if (res.status === 409) throw new Error('ALREADY_SUBMITTED');
    if (!res.ok) throw new Error('Feedback submission failed');
}

export async function getFeedback(codelabId: string): Promise<Feedback[]> {
    const res = await apiFetch(`/codelabs/${codelabId}/feedback`);
    if (!res.ok) throw new Error('Failed to fetch feedback');
    return res.json();
}

export async function submitSubmissionLink(
    codelabId: string,
    attendeeId: string,
    url: string,
    title?: string,
): Promise<Submission> {
    const res = await apiFetch(
        `/codelabs/${codelabId}/attendees/${attendeeId}/submissions/link`,
        {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ url, title }),
        },
    );
    if (!res.ok) {
        const text = await res.text().catch(() => '');
        throw new Error(text || 'Link submission failed');
    }
    return res.json();
}

export async function getUpdateStatus(): Promise<{
    frontend: { current?: string | null; latest?: string | null; update_available: boolean; error?: string | null };
    backend: { current?: string | null; latest?: string | null; update_available: boolean; error?: string | null };
}> {
    const res = await apiFetch('/admin/updates');
    if (!res.ok) {
        const text = await res.text().catch(() => '');
        throw new Error(text || 'Failed to check updates');
    }
    return res.json();
}

export async function completeCodelab(codelabId: string): Promise<void> {
    const res = await apiFetch(`/codelabs/${codelabId}/complete`, { method: 'POST' });
    if (!res.ok) throw new Error('Failed to complete codelab');
}

export async function getCertificate(attendeeId: string): Promise<CertificateInfo> {
    const res = await apiFetch(`/certificates/${attendeeId}`);
    if (!res.ok) throw new Error('Failed to fetch certificate');
    return res.json();
}

export function getWsUrl(codelabId: string, roleHint?: 'admin' | 'attendee', token?: string): string {
    const url = new URL(API_URL.replace('http', 'ws'));
    const base = `${url.protocol}//${url.host}/api/ws/${codelabId}`;
    const params = new URLSearchParams();
    if (roleHint) params.append('as', roleHint);
    if (token) params.append('token', token);
    const queryString = params.toString();
    return queryString ? `${base}?${queryString}` : base;
}

export async function getMaterials(codelabId: string): Promise<Material[]> {
    const res = await apiFetch(`/codelabs/${codelabId}/materials`);
    if (!res.ok) throw new Error('Failed to fetch materials');
    return res.json();
}

export async function getQuizzes(codelabId: string): Promise<Quiz[]> {
    const res = await apiFetch(`/codelabs/${codelabId}/quizzes`);
    if (!res.ok) throw new Error('Failed to fetch quizzes');
    return res.json();
}

export async function submitQuiz(codelabId: string, payload: QuizSubmissionPayload): Promise<void> {
    const res = await apiFetch(`/codelabs/${codelabId}/quizzes/submit`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
    });
    if (!res.ok) throw new Error('Quiz submission failed');
}

export async function getQuizSubmissions(codelabId: string): Promise<QuizSubmissionWithAttendee[]> {
    const res = await apiFetch(`/codelabs/${codelabId}/quizzes/submissions`);
    if (!res.ok) throw new Error('Failed to fetch quiz submissions');
    return res.json();
}

export async function updateQuizzes(codelabId: string, quizzes: { question: string, options: string[], correct_answer: number, correct_answers?: number[], quiz_type?: string }[]): Promise<void> {
    const res = await apiFetch(`/codelabs/${codelabId}/quizzes`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(quizzes)
    });
    if (!res.ok) throw new Error('Failed to update quizzes');
}

export async function addMaterial(codelabId: string, payload: { title: string; material_type: 'link' | 'file'; link_url?: string; file_path?: string }): Promise<Material> {
    const res = await apiFetch(`/codelabs/${codelabId}/materials`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify(payload),
    });
    if (!res.ok) throw new Error('Failed to add material');
    return res.json();
}

export async function deleteMaterial(codelabId: string, materialId: string): Promise<void> {
    const res = await apiFetch(`/codelabs/${codelabId}/materials/${materialId}`, { method: 'DELETE' });
    if (!res.ok) throw new Error('Failed to delete material');
}

export async function uploadMaterial(file: File): Promise<{ url: string; original_name: string }> {
    const formData = new FormData();
    formData.append('file', file);
    const res = await apiFetch(`/upload/material`, {
        method: 'POST',
        body: formData,
    });
    if (!res.ok) throw new Error('Upload failed');
    return res.json();
}

export async function submitFile(codelabId: string, attendeeId: string, file: File): Promise<Submission> {
    const formData = new FormData();
    formData.append('file', file);
    const res = await apiFetch(`/codelabs/${codelabId}/attendees/${attendeeId}/submissions`, {
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
    const res = await apiFetch(`/codelabs/${codelabId}/submissions`);
    if (!res.ok) throw new Error('Failed to fetch submissions');
    return res.json();
}

export async function deleteSubmission(codelabId: string, attendeeId: string, submissionId: string): Promise<void> {
    const res = await apiFetch(`/codelabs/${codelabId}/attendees/${attendeeId}/submissions/${submissionId}`, { method: 'DELETE' });
    if (!res.ok) throw new Error('Failed to delete submission');
}

// Code Server API
export interface CodeServerInfo {
    container_name?: string;
    port?: number;
    password?: string;
    path?: string;
    structure_type: string;
}

export interface WorkspaceFile {
    path: string;
    content: string;
}

export async function createCodeServer(
    codelabId: string,
    workspaceFiles?: WorkspaceFile[],
    structureType?: 'branch' | 'folder'
): Promise<CodeServerInfo> {
    const res = await apiFetch(`/codeserver`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            codelab_id: codelabId,
            workspace_files: workspaceFiles,
            ...(structureType ? { structure_type: structureType } : {})
        })
    });
    if (!res.ok) throw new Error('Failed to create code server');
    return res.json();
}

export async function getCodeServerInfo(codelabId: string): Promise<CodeServerInfo> {
    const res = await apiFetch(`/codeserver/${codelabId}`);
    if (!res.ok) throw new Error('Failed to get code server info');
    return res.json();
}

export async function createCodeServerBranch(codelabId: string, stepNumber: number, branchType: 'start' | 'end'): Promise<void> {
    const res = await apiFetch(`/codeserver/${codelabId}/branch`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            step_number: stepNumber,
            branch_type: branchType
        })
    });
    if (!res.ok) throw new Error('Failed to create branch');
}

export async function createCodeServerFolder(codelabId: string, stepNumber: number, folderType: 'start' | 'end', files: WorkspaceFile[]): Promise<void> {
    const res = await apiFetch(`/codeserver/${codelabId}/folder`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            step_number: stepNumber,
            folder_type: folderType,
            files
        })
    });
    if (!res.ok) throw new Error('Failed to create folder');
}

export async function downloadCodeServerWorkspace(codelabId: string): Promise<void> {
    const res = await apiFetch(`/codeserver/${codelabId}/download`);
    if (!res.ok) throw new Error('Failed to download workspace');

    const blob = await res.blob();
    const url = window.URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `codelab-${codelabId}-workspace.tar`;
    document.body.appendChild(a);
    a.click();
    window.URL.revokeObjectURL(url);
    document.body.removeChild(a);
}

export async function getWorkspaceBranches(codelabId: string): Promise<string[]> {
    const res = await apiFetch(`/codeserver/${codelabId}/branches`);
    if (!res.ok) throw new Error('Failed to get branches');
    return res.json();
}

export async function getWorkspaceFiles(codelabId: string, branch: string): Promise<string[]> {
    const res = await apiFetch(`/codeserver/${codelabId}/branches/${encodeURIComponent(branch)}/files`);
    if (!res.ok) throw new Error('Failed to get files');
    return res.json();
}

export async function getWorkspaceFileContent(codelabId: string, branch: string, file: string): Promise<string> {
    const res = await apiFetch(`/codeserver/${codelabId}/branches/${encodeURIComponent(branch)}/file?file=${encodeURIComponent(file)}`);
    if (!res.ok) throw new Error('Failed to get file content');
    return res.text();
}

export async function updateWorkspaceBranchFiles(
    codelabId: string,
    branch: string,
    files: WorkspaceFile[],
    deleteFiles: string[] = [],
    commitMessage?: string
): Promise<void> {
    const res = await apiFetch(`/codeserver/${codelabId}/branches/${encodeURIComponent(branch)}/files`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            files,
            delete_files: deleteFiles.length ? deleteFiles : undefined,
            commit_message: commitMessage
        })
    });
    if (!res.ok) throw new Error('Failed to update workspace branch files');
}

export async function getWorkspaceFolders(codelabId: string): Promise<string[]> {
    const res = await apiFetch(`/codeserver/${codelabId}/folders`);
    if (!res.ok) throw new Error('Failed to get folders');
    return res.json();
}

export async function getWorkspaceFolderFiles(codelabId: string, folder: string): Promise<string[]> {
    const res = await apiFetch(`/codeserver/${codelabId}/folders/${encodeURIComponent(folder)}/files`);
    if (!res.ok) throw new Error('Failed to get folder files');
    return res.json();
}

export async function getWorkspaceFolderFileContent(codelabId: string, folder: string, file: string): Promise<string> {
    const res = await apiFetch(`/codeserver/${codelabId}/folders/${encodeURIComponent(folder)}/file?file=${encodeURIComponent(file)}`);
    if (!res.ok) throw new Error('Failed to get folder file content');
    return res.text();
}

export async function updateWorkspaceFolderFiles(
    codelabId: string,
    folder: string,
    files: WorkspaceFile[],
    deleteFiles: string[] = []
): Promise<void> {
    const res = await apiFetch(`/codeserver/${codelabId}/folders/${encodeURIComponent(folder)}/files`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            files,
            delete_files: deleteFiles.length ? deleteFiles : undefined
        })
    });
    if (!res.ok) throw new Error('Failed to update workspace folder files');
}

export async function deleteCodeServer(codelabId: string): Promise<void> {
    const res = await apiFetch(`/codeserver/${codelabId}`, { method: 'DELETE' });
    if (!res.ok) throw new Error('Failed to delete code server');
}

export async function saveAiConversation(payload: SaveAiConversationPayload): Promise<{ id: string }> {
    const res = await apiFetch('/ai/conversations', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
    });
    if (!res.ok) throw new Error('Failed to save AI conversation');
    return res.json();
}

export async function getAiConversations(codelabId: string): Promise<AiConversation[]> {
    const res = await apiFetch(`/codelabs/${codelabId}/ai/conversations`);
    if (!res.ok) throw new Error('Failed to get AI conversations');
    return res.json();
}
