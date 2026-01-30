import { supabase } from "./supabase";
import type {
    Codelab,
    Step,
    Attendee,
    HelpRequest,
    ChatMessage,
    Feedback,
    Material,
    CertificateInfo,
    Quiz,
    QuizSubmissionPayload,
    QuizSubmissionWithAttendee,
    Submission,
    SubmissionWithAttendee,
} from "./types";

const CODELABS_TABLE = "codelabs";
const STORAGE_BUCKET =
    import.meta.env.VITE_SUPABASE_STORAGE_BUCKET || "open-codelabs";

function requireClient() {
    if (!supabase) throw new Error("Supabase is not configured");
    return supabase;
}

function getAdminToken(): string | null {
    if (typeof localStorage === "undefined") return null;
    return localStorage.getItem("adminToken");
}

function isAdmin(): boolean {
    return !!getAdminToken();
}

function randomId(): string {
    if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
        return crypto.randomUUID();
    }
    return `${Date.now()}-${Math.random().toString(16).slice(2)}`;
}

function normalizeUser(user: any, accessToken?: string | null) {
    if (!user) return null;
    const email = user.email || "";
    const displayName =
        user.user_metadata?.full_name ||
        user.user_metadata?.name ||
        user.user_metadata?.display_name ||
        email.split("@")[0] ||
        "User";
    const photoURL =
        user.user_metadata?.avatar_url || user.user_metadata?.picture || null;
    return {
        uid: user.id,
        email,
        displayName,
        photoURL,
        accessToken: accessToken || null,
    };
}

async function getAuthUser() {
    const client = requireClient();
    const { data, error } = await client.auth.getUser();
    if (error) return null;
    return data.user;
}

function toFirestoreTimestamp(value?: string | null) {
    if (!value) return undefined;
    return { toDate: () => new Date(value) };
}

function getStoredAttendeeId(codelabId: string): string | null {
    if (typeof localStorage === "undefined") return null;
    const raw = localStorage.getItem(`attendee_${codelabId}`);
    if (!raw) return null;
    try {
        const parsed = JSON.parse(raw) as { id?: string };
        return parsed.id || null;
    } catch {
        return null;
    }
}

function getPublicUrl(path: string): string {
    const client = requireClient();
    const { data } = client.storage.from(STORAGE_BUCKET).getPublicUrl(path);
    return data.publicUrl;
}

function toPublicFilePath(path?: string | null): string | null {
    if (!path) return null;
    if (path.startsWith("http://") || path.startsWith("https://")) return path;
    return getPublicUrl(path);
}

function extractStoragePath(url: string): string | null {
    const marker = `/storage/v1/object/public/${STORAGE_BUCKET}/`;
    const index = url.indexOf(marker);
    if (index === -1) return null;
    return url.slice(index + marker.length);
}

async function uploadToBucket(path: string, file: File): Promise<string> {
    const client = requireClient();
    const { error } = await client.storage
        .from(STORAGE_BUCKET)
        .upload(path, file, {
            upsert: false,
            contentType: file.type || "application/octet-stream",
        });
    if (error) throw error;
    return getPublicUrl(path);
}

export async function listCodelabs(): Promise<Codelab[]> {
    const client = requireClient();
    const user = await getAuthUser();
    let query = client
        .from(CODELABS_TABLE)
        .select("*")
        .order("created_at", { ascending: false });

    if (!user && !isAdmin()) {
        query = query.eq("is_public", true);
    } else if (user && !isAdmin()) {
        query = query.or(`is_public.eq.true,owner_id.eq.${user.id}`);
    }

    const { data, error } = await query;
    if (error) throw error;
    return (data || []) as Codelab[];
}

export async function getMyCodelabs(): Promise<Codelab[]> {
    const client = requireClient();
    const user = await getAuthUser();
    if (!user) return [];

    const { data, error } = await client
        .from(CODELABS_TABLE)
        .select("*")
        .eq("owner_id", user.id)
        .order("created_at", { ascending: false });
    if (error) throw error;
    return (data || []) as Codelab[];
}

export async function getJoinedCodelabs(): Promise<Codelab[]> {
    const client = requireClient();
    const user = await getAuthUser();
    if (!user) return [];

    const { data, error } = await client
        .from("participations")
        .select("codelab_id, codelabs(*)")
        .eq("user_id", user.id)
        .order("joined_at", { ascending: false });
    if (error) throw error;

    return (data || [])
        .map((row: any) => {
            if (!row.codelabs) return null;
            return { id: row.codelab_id, ...row.codelabs } as Codelab;
        })
        .filter(Boolean) as Codelab[];
}

export async function getCodelab(id: string): Promise<[Codelab, Step[]]> {
    const client = requireClient();
    const { data: codelab, error } = await client
        .from(CODELABS_TABLE)
        .select("*")
        .eq("id", id)
        .single();
    if (error || !codelab) throw new Error("Codelab not found");

    const user = await getAuthUser();
    if (!codelab.is_public && !isAdmin()) {
        if (!user || codelab.owner_id !== user.id) {
            throw new Error("PRIVATE_CODELAB");
        }
    }

    const { data: steps, error: stepsError } = await client
        .from("steps")
        .select("*")
        .eq("codelab_id", id)
        .order("step_number", { ascending: true });
    if (stepsError) throw stepsError;

    return [codelab as Codelab, (steps || []) as Step[]];
}

export async function createCodelab(payload: {
    title: string;
    description: string;
    author: string;
    is_public?: boolean;
    quiz_enabled?: boolean;
    require_quiz?: boolean;
    require_feedback?: boolean;
}): Promise<Codelab> {
    const client = requireClient();
    const user = await getAuthUser();
    const data = {
        ...payload,
        owner_id: user?.id || null,
        is_public: payload.is_public ?? true,
        quiz_enabled: payload.quiz_enabled ?? false,
        require_quiz: payload.require_quiz ?? false,
        require_feedback: payload.require_feedback ?? false,
    };

    const { data: created, error } = await client
        .from(CODELABS_TABLE)
        .insert(data)
        .select("*")
        .single();
    if (error || !created) throw error || new Error("Failed to create codelab");

    return created as Codelab;
}

export async function updateCodelab(
    id: string,
    payload: {
        title: string;
        description: string;
        author: string;
        is_public?: boolean;
        quiz_enabled?: boolean;
        require_quiz?: boolean;
        require_feedback?: boolean;
        guide_markdown?: string;
    },
): Promise<Codelab> {
    const client = requireClient();
    const data = {
        ...payload,
        is_public: payload.is_public ?? true,
        quiz_enabled: payload.quiz_enabled ?? false,
        require_quiz: payload.require_quiz ?? false,
        require_feedback: payload.require_feedback ?? false,
    };
    const { data: updated, error } = await client
        .from(CODELABS_TABLE)
        .update(data)
        .eq("id", id)
        .select("*")
        .single();
    if (error || !updated) throw error || new Error("Failed to update codelab");
    return updated as Codelab;
}

export async function saveSteps(
    codelabId: string,
    steps: { title: string; content_markdown: string }[],
): Promise<void> {
    const client = requireClient();
    const { error: deleteError } = await client
        .from("steps")
        .delete()
        .eq("codelab_id", codelabId);
    if (deleteError) throw deleteError;

    const payload = steps.map((step, index) => ({
        codelab_id: codelabId,
        step_number: index + 1,
        title: step.title,
        content_markdown: step.content_markdown,
    }));

    if (payload.length === 0) return;
    const { error } = await client.from("steps").insert(payload);
    if (error) throw error;
}

export async function deleteCodelab(codelabId: string): Promise<void> {
    const client = requireClient();
    const { error } = await client
        .from(CODELABS_TABLE)
        .delete()
        .eq("id", codelabId);
    if (error) throw error;
}

export async function login(
    admin_id: string,
    admin_pw: string,
): Promise<{ status: string; token?: string }> {
    if (
        admin_id === import.meta.env.VITE_ADMIN_ID &&
        admin_pw === import.meta.env.VITE_ADMIN_PW
    ) {
        return { status: "ok", token: "supabase-admin-token" };
    }
    throw new Error("Invalid credentials");
}

export async function loginWithGoogle(): Promise<{ token: string; user: any }> {
    const client = requireClient();
    const redirectTo =
        typeof window !== "undefined" ? window.location.href : undefined;
    const { data, error } = await client.auth.signInWithOAuth({
        provider: "google",
        options: {
            redirectTo,
            skipBrowserRedirect: true,
        },
    });
    if (error) throw error;
    if (data?.url && typeof window !== "undefined") {
        window.location.assign(data.url);
    }
    const user = await getAuthUser();
    return { token: "", user: normalizeUser(user) };
}

export async function logout(): Promise<void> {
    const client = requireClient();
    await client.auth.signOut();
    if (typeof localStorage !== "undefined") {
        localStorage.removeItem("adminToken");
        localStorage.removeItem("user");
    }
}

export function onAuthChange(callback: (user: any | null) => void) {
    const client = requireClient();
    const { data } = client.auth.onAuthStateChange((_event, session) => {
        callback(normalizeUser(session?.user, session?.access_token));
    });
    return () => data.subscription.unsubscribe();
}

export async function getSession(): Promise<{ role: string; sub: string } | null> {
    const token = getAdminToken();
    if (token) return { role: "admin", sub: "supabase-admin" };

    const client = requireClient();
    const { data, error } = await client.auth.getSession();
    if (error || !data.session) return null;
    return { role: "admin", sub: data.session.user.id };
}

export async function registerAttendee(
    codelabId: string,
    name: string,
    code: string,
    email?: string,
): Promise<Attendee> {
    const client = requireClient();

    const { data: existing, error: existingError } = await client
        .from("attendees")
        .select("id")
        .eq("codelab_id", codelabId)
        .eq("name", name)
        .limit(1);
    if (existingError) throw existingError;
    if (existing && existing.length > 0) throw new Error("DUPLICATE_NAME");

    const { data: attendee, error } = await client
        .from("attendees")
        .insert({
            codelab_id: codelabId,
            name,
            code,
            email: email || null,
            current_step: 1,
        })
        .select("*")
        .single();
    if (error || !attendee) throw error || new Error("Registration failed");

    const user = await getAuthUser();
    if (user) {
        await client.from("participations").upsert(
            {
                user_id: user.id,
                codelab_id: codelabId,
                attendee_id: attendee.id,
                joined_at: new Date().toISOString(),
            },
            { onConflict: "user_id,codelab_id" },
        );
    }

    return {
        id: attendee.id,
        name: attendee.name,
        code: attendee.code,
        codelab_id: attendee.codelab_id,
        current_step: attendee.current_step,
        is_completed: attendee.is_completed,
        completed_at: attendee.completed_at,
        created_at: attendee.created_at,
    } as Attendee;
}

export async function updateAttendeeProgress(
    codelabId: string,
    attendeeId: string,
    stepNumber: number,
): Promise<void> {
    const client = requireClient();
    const { error } = await client
        .from("attendees")
        .update({ current_step: stepNumber })
        .eq("id", attendeeId)
        .eq("codelab_id", codelabId);
    if (error) throw error;
}

export async function requestHelp(
    codelabId: string,
    stepNumber: number,
): Promise<void> {
    const client = requireClient();
    const attendeeId = getStoredAttendeeId(codelabId);
    if (!attendeeId) throw new Error("ATTENDEE_REQUIRED");

    const { error } = await client.from("help_requests").insert({
        codelab_id: codelabId,
        attendee_id: attendeeId,
        step_number: stepNumber,
        status: "pending",
    });
    if (error) throw error;
}

export async function getHelpRequests(
    codelabId: string,
): Promise<HelpRequest[]> {
    const client = requireClient();
    const { data, error } = await client
        .from("help_requests")
        .select("id, codelab_id, attendee_id, step_number, status, created_at, attendees(name)")
        .eq("codelab_id", codelabId)
        .order("created_at", { ascending: true });
    if (error) throw error;

    return (data || []).map((row: any) => ({
        id: row.id,
        codelab_id: row.codelab_id,
        attendee_id: row.attendee_id,
        attendee_name: row.attendees?.name || "Unknown",
        step_number: row.step_number,
        status: row.status,
    })) as HelpRequest[];
}

export async function resolveHelpRequest(
    codelabId: string,
    helpId: string,
): Promise<void> {
    const client = requireClient();
    const { error } = await client
        .from("help_requests")
        .update({ status: "resolved" })
        .eq("id", helpId)
        .eq("codelab_id", codelabId);
    if (error) throw error;
}

export async function getAttendees(codelabId: string): Promise<Attendee[]> {
    const client = requireClient();
    const { data, error } = await client
        .from("attendees")
        .select("*")
        .eq("codelab_id", codelabId)
        .order("created_at", { ascending: false });
    if (error) throw error;
    return (data || []) as Attendee[];
}

export async function getChatHistory(
    codelabId: string,
): Promise<ChatMessage[]> {
    const client = requireClient();
    const { data, error } = await client
        .from("chat_messages")
        .select("*")
        .eq("codelab_id", codelabId)
        .order("created_at", { ascending: true });
    if (error) throw error;
    return (data || []) as ChatMessage[];
}

export async function sendChatMessage(
    codelabId: string,
    payload: { sender: string; message: string; type: "chat" | "dm"; target_id?: string },
): Promise<void> {
    const client = requireClient();
    const { error } = await client.from("chat_messages").insert({
        codelab_id: codelabId,
        sender_name: payload.sender,
        message: payload.message,
        msg_type: payload.type,
        target_id: payload.target_id || null,
    });
    if (error) throw error;
}

export async function uploadImage(file: File): Promise<{ url: string }> {
    const ext = file.name.split(".").pop() || "bin";
    const path = `images/${randomId()}.${ext}`;
    const url = await uploadToBucket(path, file);
    return { url };
}

export async function submitFeedback(
    codelabId: string,
    payload: { difficulty: number; satisfaction: number; comments: string; attendee_id?: string },
): Promise<void> {
    const client = requireClient();
    const attendeeId = payload.attendee_id || getStoredAttendeeId(codelabId);
    if (!attendeeId) throw new Error("ATTENDEE_REQUIRED");

    const { data: existing, error: existingError } = await client
        .from("feedback")
        .select("id")
        .eq("codelab_id", codelabId)
        .eq("attendee_id", attendeeId)
        .limit(1);
    if (existingError) throw existingError;
    if (existing && existing.length > 0) throw new Error("ALREADY_SUBMITTED");

    const { error } = await client.from("feedback").insert({
        codelab_id: codelabId,
        attendee_id: attendeeId,
        difficulty: payload.difficulty.toString(),
        satisfaction: payload.satisfaction.toString(),
        comment: payload.comments,
    });
    if (error) throw error;
}

export async function getFeedback(codelabId: string): Promise<Feedback[]> {
    const client = requireClient();
    const { data, error } = await client
        .from("feedback")
        .select("*")
        .eq("codelab_id", codelabId)
        .order("created_at", { ascending: false });
    if (error) throw error;
    return (data || []) as Feedback[];
}

export async function completeCodelab(codelabId: string): Promise<void> {
    const client = requireClient();
    const attendeeId = getStoredAttendeeId(codelabId);
    if (!attendeeId) throw new Error("ATTENDEE_REQUIRED");

    const { error } = await client
        .from("attendees")
        .update({ is_completed: true, completed_at: new Date().toISOString() })
        .eq("id", attendeeId)
        .eq("codelab_id", codelabId);
    if (error) throw error;
}

export async function getCertificate(attendeeId: string): Promise<CertificateInfo> {
    const client = requireClient();
    const { data, error } = await client
        .from("attendees")
        .select("name, completed_at, is_completed, codelabs(id, title, author)")
        .eq("id", attendeeId)
        .single();
    if (error || !data) throw error || new Error("Certificate not found");

    if (!data.is_completed) {
        throw new Error("REQUIREMENTS_NOT_MET");
    }

    return {
        attendee_name: data.name,
        codelab_title: data.codelabs?.title || "",
        codelab_id: data.codelabs?.id || "",
        author: data.codelabs?.author || "",
        completed_at: data.completed_at || "",
        verification_url: `/verify/${attendeeId}`,
    };
}

export async function getMaterials(codelabId: string): Promise<Material[]> {
    const client = requireClient();
    const { data, error } = await client
        .from("materials")
        .select("*")
        .eq("codelab_id", codelabId)
        .order("created_at", { ascending: true });
    if (error) throw error;

    return (data || []).map((row: any) => ({
        ...row,
        file_path: toPublicFilePath(row.file_path),
    })) as Material[];
}

export async function addMaterial(
    codelabId: string,
    payload: { title: string; material_type: "link" | "file"; link_url?: string; file_path?: string },
): Promise<Material> {
    const client = requireClient();
    const storagePath = payload.file_path
        ? extractStoragePath(payload.file_path)
        : null;
    const { data, error } = await client
        .from("materials")
        .insert({
            codelab_id: codelabId,
            title: payload.title,
            material_type: payload.material_type,
            link_url: payload.link_url || null,
            file_path: storagePath || payload.file_path || null,
        })
        .select("*")
        .single();
    if (error || !data) throw error || new Error("Failed to add material");

    return {
        ...data,
        file_path: toPublicFilePath(data.file_path),
    } as Material;
}

export async function deleteMaterial(
    codelabId: string,
    materialId: string,
): Promise<void> {
    const client = requireClient();
    const { data: material, error: fetchError } = await client
        .from("materials")
        .select("file_path")
        .eq("id", materialId)
        .eq("codelab_id", codelabId)
        .single();
    if (fetchError) throw fetchError;

    const storagePath = material?.file_path
        ? extractStoragePath(material.file_path) || material.file_path
        : null;
    if (storagePath) {
        const { error: removeError } = await client.storage
            .from(STORAGE_BUCKET)
            .remove([storagePath]);
        if (removeError) throw removeError;
    }

    const { error } = await client
        .from("materials")
        .delete()
        .eq("id", materialId)
        .eq("codelab_id", codelabId);
    if (error) throw error;
}

export async function uploadMaterial(
    file: File,
): Promise<{ url: string; original_name: string }> {
    const ext = file.name.split(".").pop() || "bin";
    const path = `materials/${randomId()}.${ext}`;
    const url = await uploadToBucket(path, file);
    return { url, original_name: file.name };
}

export async function submitFile(
    codelabId: string,
    attendeeId: string,
    file: File,
): Promise<Submission> {
    const client = requireClient();
    const ext = file.name.split(".").pop() || "bin";
    const path = `submissions/${codelabId}/${attendeeId}/${randomId()}.${ext}`;
    const url = await uploadToBucket(path, file);

    const { data, error } = await client
        .from("submissions")
        .insert({
            codelab_id: codelabId,
            attendee_id: attendeeId,
            file_path: path,
            file_name: file.name,
            file_size: file.size,
        })
        .select("*")
        .single();
    if (error || !data) throw error || new Error("Submission failed");

    return {
        ...data,
        file_path: url,
    } as Submission;
}

export async function getSubmissions(
    codelabId: string,
): Promise<SubmissionWithAttendee[]> {
    const client = requireClient();
    const { data, error } = await client
        .from("submissions")
        .select("*, attendees(name)")
        .eq("codelab_id", codelabId)
        .order("created_at", { ascending: false });
    if (error) throw error;

    return (data || []).map((row: any) => ({
        ...row,
        attendee_name: row.attendees?.name || "Unknown",
        file_path: toPublicFilePath(row.file_path) || row.file_path,
    })) as SubmissionWithAttendee[];
}

export async function deleteSubmission(
    codelabId: string,
    attendeeId: string,
    submissionId: string,
): Promise<void> {
    const client = requireClient();
    const { data: submission, error: fetchError } = await client
        .from("submissions")
        .select("file_path")
        .eq("id", submissionId)
        .eq("codelab_id", codelabId)
        .eq("attendee_id", attendeeId)
        .single();
    if (fetchError) throw fetchError;

    const submissionPath = submission?.file_path
        ? extractStoragePath(submission.file_path) || submission.file_path
        : null;
    if (submissionPath) {
        const { error: removeError } = await client.storage
            .from(STORAGE_BUCKET)
            .remove([submissionPath]);
        if (removeError) throw removeError;
    }

    const { error } = await client
        .from("submissions")
        .delete()
        .eq("id", submissionId)
        .eq("codelab_id", codelabId)
        .eq("attendee_id", attendeeId);
    if (error) throw error;
}

export async function getQuizzes(codelabId: string): Promise<Quiz[]> {
    const client = requireClient();
    const { data, error } = await client
        .from("quizzes")
        .select("*")
        .eq("codelab_id", codelabId)
        .order("created_at", { ascending: true });
    if (error) throw error;
    return (data || []) as Quiz[];
}

export async function updateQuizzes(
    codelabId: string,
    quizzes: { question: string; options: string[]; correct_answer: number; quiz_type?: string }[],
): Promise<void> {
    const client = requireClient();
    const { error: deleteError } = await client
        .from("quizzes")
        .delete()
        .eq("codelab_id", codelabId);
    if (deleteError) throw deleteError;

    if (quizzes.length === 0) return;
    const payload = quizzes.map((q) => ({
        codelab_id: codelabId,
        question: q.question,
        options: JSON.stringify(q.options),
        correct_answer: q.correct_answer,
        quiz_type: q.quiz_type || "multiple_choice",
    }));
    const { error } = await client.from("quizzes").insert(payload);
    if (error) throw error;
}

export async function submitQuiz(
    codelabId: string,
    payload: QuizSubmissionPayload,
): Promise<void> {
    const client = requireClient();
    const attendeeId = getStoredAttendeeId(codelabId);
    if (!attendeeId) throw new Error("ATTENDEE_REQUIRED");

    const { error: deleteError } = await client
        .from("quiz_submissions")
        .delete()
        .eq("codelab_id", codelabId)
        .eq("attendee_id", attendeeId);
    if (deleteError) throw deleteError;

    const rows = payload.submissions.map((submission) => ({
        codelab_id: codelabId,
        attendee_id: attendeeId,
        quiz_id: submission.quiz_id,
        answer: submission.answer,
        is_correct: submission.is_correct,
    }));
    if (rows.length === 0) return;

    const { error } = await client.from("quiz_submissions").insert(rows);
    if (error) throw error;
}

export async function getQuizSubmissions(
    codelabId: string,
): Promise<QuizSubmissionWithAttendee[]> {
    const client = requireClient();
    const { data, error } = await client
        .from("quiz_submissions")
        .select("*, attendees(name)")
        .eq("codelab_id", codelabId)
        .order("created_at", { ascending: false });
    if (error) throw error;

    return (data || []).map((row: any) => ({
        ...row,
        attendee_name: row.attendees?.name || "Unknown",
    })) as QuizSubmissionWithAttendee[];
}

export function listenToWsReplacement(
    codelabId: string,
    callback: (msg: any) => void,
) {
    const client = requireClient();
    const channel = client.channel(`codelab-${codelabId}`);

    channel.on(
        "postgres_changes",
        {
            event: "INSERT",
            schema: "public",
            table: "chat_messages",
            filter: `codelab_id=eq.${codelabId}`,
        },
        (payload) => {
            const msg: any = payload.new || {};
            callback({
                type: msg.msg_type || "chat",
                ...msg,
                created_at: toFirestoreTimestamp(msg.created_at),
            });
        },
    );

    channel.on(
        "postgres_changes",
        {
            event: "*",
            schema: "public",
            table: "help_requests",
            filter: `codelab_id=eq.${codelabId}`,
        },
        (payload) => {
            const req: any = payload.new || payload.old || {};
            callback({ type: "help_request", ...req });
        },
    );

    channel.on(
        "postgres_changes",
        {
            event: "*",
            schema: "public",
            table: "attendees",
            filter: `codelab_id=eq.${codelabId}`,
        },
        (payload) => {
            const attendee: any = payload.new || payload.old || {};
            callback({ type: "progress_update", attendee });
        },
    );

    channel.subscribe();

    return () => {
        void client.removeChannel(channel);
    };
}
