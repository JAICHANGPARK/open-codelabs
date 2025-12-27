export interface Codelab {
    id: string;
    title: string;
    description: string;
    author: string;
    is_public: boolean;
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
