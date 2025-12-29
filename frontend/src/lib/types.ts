export interface Codelab {
    id: string;
    title: string;
    description: string;
    author: string;
    is_public: boolean;
    quiz_enabled: boolean;
    require_quiz: boolean;
    require_feedback: boolean;
    created_at?: string;
}

export interface Quiz {
    id: string;
    codelab_id: string;
    question: string;
    options: string; // JSON string
    correct_answer: number;
    created_at?: string;
}

export interface Step {
    id: string;
    codelab_id: string;
    step_number: number;
    title: string;
    content_markdown: string;
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

export interface Feedback {
    id: string;
    codelab_id: string;
    difficulty: string;
    satisfaction: string;
    comment?: string;
    created_at?: string;
}

export interface Material {
    id: string;
    codelab_id: string;
    title: string;
    material_type: 'link' | 'file';
    link_url?: string;
    file_path?: string;
    created_at?: string;
}

export interface CertificateInfo {
    attendee_name: string;
    codelab_title: string;
    author: string;
    completed_at: string;
    verification_url: string;
}
