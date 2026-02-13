export interface Codelab {
    id: string;
    title: string;
    description: string;
    author: string;
    is_public: boolean;
    quiz_enabled: boolean;
    require_quiz: boolean;
    require_feedback: boolean;
    require_submission?: boolean;
    guide_markdown?: string;
    created_at?: string;
}

export interface Quiz {
    id: string;
    codelab_id: string;
    question: string;
    quiz_type: 'multiple_choice' | 'descriptive';
    options: string[];
    correct_answer: number;
    correct_answers?: number[];
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
    email?: string;
    code?: string;
    current_step?: number;
    is_completed?: boolean;
    completed_at?: string;
    created_at?: string;
    token?: string;
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
    sender_id?: string;
    created_at?: string;
}

export type InlineCommentTarget = "step" | "guide";

export interface InlineCommentMessage {
    id: string;
    thread_id: string;
    codelab_id: string;
    author_role: "admin" | "attendee";
    author_id: string;
    author_name: string;
    message: string;
    created_at?: string;
}

export interface InlineCommentThread {
    id: string;
    codelab_id: string;
    anchor_key: string;
    target_type: InlineCommentTarget;
    target_step_id?: string | null;
    start_offset: number;
    end_offset: number;
    selected_text: string;
    content_hash: string;
    created_by_attendee_id: string;
    created_at?: string;
    messages: InlineCommentMessage[];
}

export interface CreateInlineCommentPayload {
    anchor_key: string;
    target_type: InlineCommentTarget;
    target_step_id?: string | null;
    start_offset: number;
    end_offset: number;
    selected_text: string;
    content_hash: string;
    message: string;
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
    codelab_id: string;
    author: string;
    completed_at: string;
    verification_url: string;
}

export interface QuizSubmission {
    id: string;
    codelab_id: string;
    attendee_id: string;
    quiz_id: string;
    answer: string;
    is_correct: boolean;
    created_at?: string;
}

export interface QuizSubmissionWithAttendee extends QuizSubmission {
    attendee_name: string;
}

export interface QuizSubmissionPayload {
    submissions: {
        quiz_id: string;
        answer: string;
        is_correct: boolean;
    }[];
}

export interface Submission {
    id: string;
    codelab_id: string;
    attendee_id: string;
    file_path: string;
    file_name: string;
    file_size: number;
    submission_type?: 'file' | 'link';
    link_url?: string | null;
    created_at?: string;
}

export interface SubmissionWithAttendee extends Submission {
    attendee_name: string;
}

export interface AiConversation {
    id: string;
    codelab_id: string;
    user_id: string;
    user_type: 'admin' | 'attendee';
    user_name: string;
    step_number?: number;
    question: string;
    answer: string;
    model?: string;
    created_at?: string;
}

export interface SaveAiConversationPayload {
    codelab_id: string;
    step_number?: number;
    question: string;
    answer: string;
    model?: string;
}
