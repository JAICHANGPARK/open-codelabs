import { db, auth, storage, rtdb } from './firebase';
import {
    ref as rtdbRef,
    push,
    set,
    get as rtdbGet,
    onValue,
    onChildAdded,
    onChildChanged,
    onChildRemoved,
    update,
    remove,
    serverTimestamp as rtdbTimestamp
} from "firebase/database";
import {
    GoogleAuthProvider,
    signInWithPopup,
    signInAnonymously,
    signOut,
    onAuthStateChanged,
    type User
} from "firebase/auth";
import {
    collection,
    getDocs,
    getDoc,
    doc,
    addDoc,
    updateDoc,
    setDoc,
    deleteDoc,
    query,
    where,
    orderBy,
    serverTimestamp,
    type DocumentData
} from "firebase/firestore";
import {
    ref,
    uploadBytes,
    getDownloadURL
} from "firebase/storage";
import type { Codelab, Step, Attendee, HelpRequest, ChatMessage, Feedback, CreateInlineCommentPayload, InlineCommentMessage, InlineCommentThread } from './types';

const CODELABS_COLLECTION = "codelabs";

function isAdmin() {
    if (typeof localStorage === 'undefined') return false;
    return !!localStorage.getItem('adminToken');
}

async function ensureAuthUser(): Promise<User> {
    if (auth.currentUser) return auth.currentUser;
    const credential = await signInAnonymously(auth);
    return credential.user;
}

export async function listCodelabs(): Promise<Codelab[]> {
    const user = auth.currentUser;
    let q = query(collection(db, CODELABS_COLLECTION), orderBy("created_at", "desc"));

    // If not admin, only show public ones
    // Note: In Firebase mode, 'admin' can be anyone logged in or specific users.
    // For now, let's keep it simple: if logged in, you see your own + public.
    // If not logged in, only public.
    if (!user && !isAdmin()) {
        q = query(collection(db, CODELABS_COLLECTION), where("is_public", "==", true), orderBy("created_at", "desc"));
    }

    const querySnapshot = await getDocs(q);
    return querySnapshot.docs.map(d => ({ id: d.id, is_public: true, ...d.data() } as Codelab));
}

export async function getMyCodelabs(): Promise<Codelab[]> {
    const user = auth.currentUser;
    if (!user) return [];

    const q = query(
        collection(db, CODELABS_COLLECTION),
        where("owner_id", "==", user.uid),
        orderBy("created_at", "desc")
    );
    const querySnapshot = await getDocs(q);
    return querySnapshot.docs.map(d => ({ id: d.id, ...d.data() } as Codelab));
}

export async function getJoinedCodelabs(): Promise<Codelab[]> {
    const user = auth.currentUser;
    if (!user) return [];

    const q = query(collection(db, `users/${user.uid}/participations`), orderBy("joined_at", "desc"));
    const snapshot = await getDocs(q);

    const codelabs: Codelab[] = [];
    for (const d of snapshot.docs) {
        const codelabId = d.data().codelab_id;
        try {
            const cDoc = await getDoc(doc(db, CODELABS_COLLECTION, codelabId));
            if (cDoc.exists()) {
                codelabs.push({ id: cDoc.id, ...cDoc.data() } as Codelab);
            }
        } catch (e) {
            console.error(`Error fetching joined codelab ${codelabId}:`, e);
        }
    }
    return codelabs;
}

export async function getCodelab(id: string): Promise<[Codelab, Step[]]> {
    const codelabDoc = await getDoc(doc(db, CODELABS_COLLECTION, id));
    if (!codelabDoc.exists()) throw new Error('Codelab not found');

    const codelab = { id: codelabDoc.id, is_public: true, ...codelabDoc.data() } as Codelab;

    if (!codelab.is_public && !isAdmin()) {
        throw new Error('PRIVATE_CODELAB');
    }

    const stepsSnapshot = await getDocs(query(collection(db, `${CODELABS_COLLECTION}/${id}/steps`), orderBy("step_number", "asc")));
    const steps = stepsSnapshot.docs.map(d => ({ id: d.id, ...d.data() } as Step));

    return [codelab, steps];
}

export async function createCodelab(payload: { title: string; description: string; author: string; is_public?: boolean, quiz_enabled?: boolean, require_quiz?: boolean, require_feedback?: boolean }): Promise<Codelab> {
    const user = auth.currentUser;
    const data = {
        ...payload,
        owner_id: user?.uid || null,
        is_public: payload.is_public ?? true,
        quiz_enabled: payload.quiz_enabled ?? false,
        require_quiz: payload.require_quiz ?? false,
        require_feedback: payload.require_feedback ?? false,
        created_at: serverTimestamp()
    };
    const docRef = await addDoc(collection(db, CODELABS_COLLECTION), data);
    return {
        id: docRef.id,
        is_public: data.is_public,
        quiz_enabled: data.quiz_enabled,
        require_quiz: data.require_quiz,
        require_feedback: data.require_feedback,
        ...payload
    } as Codelab;
}

export async function updateCodelab(id: string, payload: { title: string; description: string; author: string; is_public?: boolean, quiz_enabled?: boolean, require_quiz?: boolean, require_feedback?: boolean }): Promise<Codelab> {
    const docRef = doc(db, CODELABS_COLLECTION, id);
    const data = {
        ...payload,
        is_public: payload.is_public ?? true,
        quiz_enabled: payload.quiz_enabled ?? false,
        require_quiz: payload.require_quiz ?? false,
        require_feedback: payload.require_feedback ?? false,
    };
    await updateDoc(docRef, data);
    return {
        id,
        is_public: data.is_public,
        quiz_enabled: data.quiz_enabled,
        require_quiz: data.require_quiz,
        require_feedback: data.require_feedback,
        ...payload
    } as Codelab;
}

export async function saveSteps(
    codelabId: string,
    steps: { id?: string; title: string; content_markdown: string }[],
): Promise<void> {
    const stepsCollection = collection(db, `${CODELABS_COLLECTION}/${codelabId}/steps`);

    // Delete existing steps first (simple approach)
    const existingSteps = await getDocs(stepsCollection);
    for (const d of existingSteps.docs) {
        await deleteDoc(d.ref);
    }

    // Add new steps (preserve existing IDs when provided)
    for (let i = 0; i < steps.length; i++) {
        const step = steps[i];
        const payload = {
            ...step,
            step_number: i + 1,
            codelab_id: codelabId,
        };
        const stepId = step.id?.trim();
        if (stepId) {
            await setDoc(doc(stepsCollection, stepId), payload);
        } else {
            await addDoc(stepsCollection, payload);
        }
    }
}

export async function deleteCodelab(codelabId: string): Promise<void> {
    await deleteDoc(doc(db, CODELABS_COLLECTION, codelabId));
    // Note: subcollections are not deleted automatically in Firestore, 
    // but for a simple workshop tool, it might be acceptable or we handle it if needed.
}

export async function login(admin_id: string, admin_pw: string): Promise<{ status: string; token?: string }> {
    // Simple mock login for Firebase mode if not using full Firebase Auth
    // In a real scenario, we might check against a 'config' collection or environment variables.
    if (admin_id === import.meta.env.VITE_ADMIN_ID && admin_pw === import.meta.env.VITE_ADMIN_PW) {
        return { status: "ok", token: "firebase-token-mock" };
    }
    throw new Error('Invalid credentials');
}

export async function loginWithGoogle(): Promise<{ token: string, user: User }> {
    const provider = new GoogleAuthProvider();
    try {
        const result = await signInWithPopup(auth, provider);
        const token = await result.user.getIdToken();
        return { token, user: result.user };
    } catch (error) {
        console.error("Google Login Error:", error);
        throw error;
    }
}

export async function logout(): Promise<void> {
    await signOut(auth);
    if (typeof localStorage !== 'undefined') {
        localStorage.removeItem('adminToken');
        localStorage.removeItem('user');
    }
}

export function onAuthChange(callback: (user: User | null) => void) {
    return onAuthStateChanged(auth, callback);
}

export async function getSession(): Promise<{ role: string; sub: string; codelab_id?: string | null } | null> {
    if (typeof localStorage === 'undefined') return null;
    const token = localStorage.getItem('adminToken');
    if (!token) return null;
    return { role: 'admin', sub: 'firebase' };
}

export async function registerAttendee(codelabId: string, name: string, code: string, email?: string): Promise<Attendee> {
    const user = auth.currentUser;
    const attendeesCollection = collection(db, `${CODELABS_COLLECTION}/${codelabId}/attendees`);

    // Check duplicate
    const q = query(attendeesCollection, where("name", "==", name));
    const snapshot = await getDocs(q);
    if (!snapshot.empty) throw new Error('DUPLICATE_NAME');

    const attendeeData = {
        name,
        code,
        email: email || null,
        uid: user?.uid || null,
        codelab_id: codelabId,
        current_step: 1,
        registered_at: serverTimestamp()
    };

    const docRef = await addDoc(attendeesCollection, attendeeData);

    // If user is logged in, also record in user's participations
    if (user) {
        try {
            await setDoc(doc(db, `users/${user.uid}/participations`, codelabId), {
                codelab_id: codelabId,
                attendee_id: docRef.id,
                joined_at: serverTimestamp()
            });
        } catch (e) {
            console.error("Failed to record participation:", e);
        }
    }

    const attendee = { id: docRef.id, name, code, codelab_id: codelabId, current_step: 1 };

    // Also register in RTDB for real-time tracking
    try {
        await set(rtdbRef(rtdb, `codelabs/${codelabId}/attendees/${docRef.id}`), {
            name,
            current_step: 1,
            last_active: rtdbTimestamp()
        });
    } catch (e) {
        console.error("RTDB registerAttendee error:", e);
    }

    return attendee;
}

export async function updateAttendeeProgress(codelabId: string, attendeeId: string, stepNumber: number): Promise<void> {
    // Update Firestore
    const docRef = doc(db, `${CODELABS_COLLECTION}/${codelabId}/attendees`, attendeeId);
    await updateDoc(docRef, { current_step: stepNumber });

    // Update RTDB
    try {
        await update(rtdbRef(rtdb, `codelabs/${codelabId}/attendees/${attendeeId}`), {
            current_step: stepNumber,
            last_active: rtdbTimestamp()
        });
    } catch (e) {
        console.error("RTDB updateAttendeeProgress error:", e);
    }
}

export async function requestHelp(codelabId: string, stepNumber: number): Promise<void> {
    const attendeeId = getStoredAttendeeId(codelabId);
    if (!attendeeId) throw new Error('ATTENDEE_REQUIRED');
    const attendeeDoc = await getDoc(doc(db, `${CODELABS_COLLECTION}/${codelabId}/attendees`, attendeeId));
    const attendeeName = attendeeDoc.exists() ? attendeeDoc.data().name : "Unknown";

    // Write to RTDB
    const helpRef = rtdbRef(rtdb, `codelabs/${codelabId}/help_requests`);
    const newHelpRef = push(helpRef);
    await set(newHelpRef, {
        id: newHelpRef.key,
        codelab_id: codelabId,
        attendee_id: attendeeId,
        attendee_name: attendeeName,
        step_number: stepNumber,
        status: "pending",
        created_at: rtdbTimestamp()
    });
}

export async function getHelpRequests(codelabId: string): Promise<HelpRequest[]> {
    // Return from RTDB for immediate status
    return new Promise((resolve) => {
        onValue(rtdbRef(rtdb, `codelabs/${codelabId}/help_requests`), (snapshot) => {
            const data = snapshot.val();
            if (!data) return resolve([]);
            const list = Object.values(data) as HelpRequest[];
            resolve(list.sort((a: any, b: any) => a.created_at - b.created_at));
        }, { onlyOnce: true });
    });
}

export async function resolveHelpRequest(codelabId: string, helpId: string): Promise<void> {
    const docRef = rtdbRef(rtdb, `codelabs/${codelabId}/help_requests/${helpId}`);
    await update(docRef, { status: "resolved" });
}

export async function getAttendees(codelabId: string): Promise<Attendee[]> {
    // Get from RTDB for real-time progress
    return new Promise((resolve) => {
        onValue(rtdbRef(rtdb, `codelabs/${codelabId}/attendees`), (snapshot) => {
            const data = snapshot.val();
            if (!data) return resolve([]);
            const list = Object.entries(data).map(([id, val]: [string, any]) => ({
                id,
                ...val
            } as Attendee));
            resolve(list);
        }, { onlyOnce: true });
    });
}

export async function getChatHistory(codelabId: string): Promise<ChatMessage[]> {
    return new Promise((resolve) => {
        onValue(rtdbRef(rtdb, `codelabs/${codelabId}/chat`), (snapshot) => {
            const data = snapshot.val();
            if (!data) return resolve([]);
            const list = Object.values(data) as ChatMessage[];
            resolve(list.sort((a: any, b: any) => a.created_at - b.created_at));
        }, { onlyOnce: true });
    });
}

export async function sendChatMessage(codelabId: string, payload: { sender: string, message: string, type: 'chat' | 'dm', target_id?: string }): Promise<void> {
    const chatRef = rtdbRef(rtdb, `codelabs/${codelabId}/chat`);
    const newChatRef = push(chatRef);
    await set(newChatRef, {
        id: newChatRef.key,
        codelab_id: codelabId,
        sender_name: payload.sender,
        message: payload.message,
        msg_type: payload.type,
        target_id: payload.target_id || null,
        created_at: rtdbTimestamp()
    });
}

function toIsoTimestamp(value: any): string | undefined {
    if (typeof value === "number") return new Date(value).toISOString();
    if (typeof value === "string") return value;
    return undefined;
}

function normalizeInlineThreads(
    threadData: Record<string, any> | null,
    messageData: Record<string, Record<string, any>> | null,
): InlineCommentThread[] {
    if (!threadData) return [];
    return Object.entries(threadData)
        .map(([anchorKey, thread]) => {
            const messageMap = messageData?.[anchorKey] || {};
            const messages = Object.entries(messageMap)
                .map(([messageId, message]) => ({
                    id: message.id || messageId,
                    thread_id: message.thread_id || thread.id || anchorKey,
                    codelab_id: message.codelab_id || thread.codelab_id,
                    author_role: message.author_role || "attendee",
                    author_id: message.author_id || "",
                    author_name: message.author_name || "Attendee",
                    message: message.message || "",
                    created_at: toIsoTimestamp(message.created_at),
                }) as InlineCommentMessage)
                .sort(
                    (a, b) =>
                        new Date(a.created_at || 0).getTime() -
                        new Date(b.created_at || 0).getTime(),
                );

            return {
                id: thread.id || anchorKey,
                codelab_id: thread.codelab_id,
                anchor_key: thread.anchor_key || anchorKey,
                target_type: thread.target_type,
                target_step_id: thread.target_step_id || null,
                start_offset: Number(thread.start_offset || 0),
                end_offset: Number(thread.end_offset || 0),
                selected_text: thread.selected_text || "",
                content_hash: thread.content_hash || "",
                created_by_attendee_id: thread.created_by_attendee_id || "",
                created_at: toIsoTimestamp(thread.created_at),
                messages,
            } as InlineCommentThread;
        })
        .sort(
            (a, b) =>
                new Date(a.created_at || 0).getTime() -
                new Date(b.created_at || 0).getTime(),
        );
}

export async function getInlineComments(
    codelabId: string,
    params?: { target_type?: "step" | "guide"; target_step_id?: string },
): Promise<InlineCommentThread[]> {
    const [threadsSnap, messagesSnap] = await Promise.all([
        rtdbGet(rtdbRef(rtdb, `codelabs/${codelabId}/inline_comment_threads`)),
        rtdbGet(rtdbRef(rtdb, `codelabs/${codelabId}/inline_comment_messages`)),
    ]);

    let threads = normalizeInlineThreads(
        (threadsSnap.val() as Record<string, any>) || null,
        (messagesSnap.val() as Record<string, Record<string, any>>) || null,
    );

    if (params?.target_type) {
        threads = threads.filter((thread) => thread.target_type === params.target_type);
    }
    if (params?.target_step_id) {
        threads = threads.filter(
            (thread) => thread.target_step_id === params.target_step_id,
        );
    }

    return threads;
}

export async function createInlineComment(
    codelabId: string,
    payload: CreateInlineCommentPayload,
): Promise<InlineCommentThread> {
    const attendee = getStoredAttendee(codelabId);
    if (!attendee) throw new Error("ATTENDEE_REQUIRED");
    const user = await ensureAuthUser();

    const threads = await getInlineComments(codelabId);
    let thread = threads.find((item) => item.anchor_key === payload.anchor_key);

    if (!thread) {
        const overlap = threads.find(
            (item) =>
                item.target_type === payload.target_type &&
                (item.target_step_id || null) ===
                    (payload.target_type === "step"
                        ? (payload.target_step_id || null)
                        : null) &&
                item.content_hash === payload.content_hash &&
                item.start_offset < payload.end_offset &&
                item.end_offset > payload.start_offset,
        );
        if (overlap) {
            throw new Error("OVERLAPPING_HIGHLIGHT");
        }

        const newThread = {
            id: payload.anchor_key,
            codelab_id: codelabId,
            anchor_key: payload.anchor_key,
            target_type: payload.target_type,
            target_step_id:
                payload.target_type === "step"
                    ? (payload.target_step_id || null)
                    : null,
            start_offset: payload.start_offset,
            end_offset: payload.end_offset,
            selected_text: payload.selected_text,
            content_hash: payload.content_hash,
            created_by_attendee_id: attendee.id,
            created_at: Date.now(),
        };
        await set(
            rtdbRef(
                rtdb,
                `codelabs/${codelabId}/inline_comment_threads/${payload.anchor_key}`,
            ),
            newThread,
        );
        thread = {
            ...newThread,
            created_at: toIsoTimestamp(newThread.created_at),
            messages: [],
        } as InlineCommentThread;
    }

    const messageRef = push(
        rtdbRef(rtdb, `codelabs/${codelabId}/inline_comment_messages/${thread.id}`),
    );
    await set(messageRef, {
        id: messageRef.key,
        thread_id: thread.id,
        codelab_id: codelabId,
        author_role: "attendee",
        author_id: attendee.id,
        author_uid: user.uid,
        author_name: attendee.name,
        message: payload.message.trim(),
        created_at: rtdbTimestamp(),
    });

    const refreshed = await getInlineComments(codelabId);
    const found = refreshed.find((item) => item.id === thread.id);
    if (!found) throw new Error("INLINE_COMMENT_NOT_FOUND");
    return found;
}

export async function replyInlineComment(
    codelabId: string,
    threadId: string,
    payload: { message: string; content_hash: string },
): Promise<InlineCommentThread> {
    const attendee = getStoredAttendee(codelabId);
    if (!attendee) throw new Error("ATTENDEE_REQUIRED");
    const user = await ensureAuthUser();

    const threadSnap = await rtdbGet(
        rtdbRef(rtdb, `codelabs/${codelabId}/inline_comment_threads/${threadId}`),
    );
    const thread = threadSnap.val();
    if (!thread) throw new Error("INLINE_COMMENT_THREAD_NOT_FOUND");
    if ((thread.content_hash || "") !== payload.content_hash.trim()) {
        throw new Error("STALE_THREAD");
    }

    const messageRef = push(
        rtdbRef(rtdb, `codelabs/${codelabId}/inline_comment_messages/${threadId}`),
    );
    await set(messageRef, {
        id: messageRef.key,
        thread_id: threadId,
        codelab_id: codelabId,
        author_role: "attendee",
        author_id: attendee.id,
        author_uid: user.uid,
        author_name: attendee.name,
        message: payload.message.trim(),
        created_at: rtdbTimestamp(),
    });

    const refreshed = await getInlineComments(codelabId);
    const found = refreshed.find((item) => item.id === threadId);
    if (!found) throw new Error("INLINE_COMMENT_THREAD_NOT_FOUND");
    return found;
}

export async function deleteInlineComment(
    codelabId: string,
    threadId: string,
    commentId: string,
): Promise<void> {
    const attendeeId = getStoredAttendeeId(codelabId);
    if (!attendeeId) throw new Error("ATTENDEE_REQUIRED");
    const user = await ensureAuthUser();

    const messagePath = `codelabs/${codelabId}/inline_comment_messages/${threadId}/${commentId}`;
    const messageSnap = await rtdbGet(rtdbRef(rtdb, messagePath));
    const message = messageSnap.val();
    if (!message) throw new Error("INLINE_COMMENT_NOT_FOUND");
    if (
        message.author_id !== attendeeId ||
        (message.author_uid && message.author_uid !== user.uid)
    ) {
        throw new Error("FORBIDDEN");
    }

    await remove(rtdbRef(rtdb, messagePath));

    const remainingSnap = await rtdbGet(
        rtdbRef(rtdb, `codelabs/${codelabId}/inline_comment_messages/${threadId}`),
    );
    const remaining = remainingSnap.val();
    if (!remaining || Object.keys(remaining).length === 0) {
        await remove(
            rtdbRef(rtdb, `codelabs/${codelabId}/inline_comment_threads/${threadId}`),
        );
        await remove(
            rtdbRef(rtdb, `codelabs/${codelabId}/inline_comment_messages/${threadId}`),
        );
    }
}

export async function uploadImage(file: File): Promise<{ url: string }> {
    const storageRef = ref(storage, `images/${Date.now()}_${file.name}`);
    await uploadBytes(storageRef, file);
    const url = await getDownloadURL(storageRef);
    return { url };
}

export async function submitFeedback(codelabId: string, payload: { difficulty: number; satisfaction: number; comments: string; attendee_id?: string }): Promise<void> {
    const feedbackCollection = collection(db, `${CODELABS_COLLECTION}/${codelabId}/feedback`);

    // Check if already submitted
    const attendeeId = payload.attendee_id || getStoredAttendeeId(codelabId);
    if (!attendeeId) throw new Error('ATTENDEE_REQUIRED');
    const q = query(feedbackCollection, where("attendee_id", "==", attendeeId));
    const snapshot = await getDocs(q);
    if (!snapshot.empty) throw new Error('ALREADY_SUBMITTED');

    await addDoc(feedbackCollection, {
        codelab_id: codelabId,
        attendee_id: attendeeId,
        difficulty: payload.difficulty.toString(),
        satisfaction: payload.satisfaction.toString(),
        comment: payload.comments,
        created_at: serverTimestamp()
    });
}

function getStoredAttendeeId(codelabId: string): string | null {
    if (typeof localStorage === 'undefined') return null;
    const raw = localStorage.getItem(`attendee_${codelabId}`);
    if (!raw) return null;
    try {
        const parsed = JSON.parse(raw) as { id?: string };
        return parsed.id || null;
    } catch {
        return null;
    }
}

function getStoredAttendee(codelabId: string): { id: string; name: string } | null {
    if (typeof localStorage === 'undefined') return null;
    const raw = localStorage.getItem(`attendee_${codelabId}`);
    if (!raw) return null;
    try {
        const parsed = JSON.parse(raw) as { id?: string; name?: string };
        if (!parsed.id || !parsed.name) return null;
        return { id: parsed.id, name: parsed.name };
    } catch {
        return null;
    }
}

export async function getFeedback(codelabId: string): Promise<Feedback[]> {
    const q = query(collection(db, `${CODELABS_COLLECTION}/${codelabId}/feedback`), orderBy("created_at", "desc"));
    const snapshot = await getDocs(q);
    return snapshot.docs.map(d => ({ id: d.id, ...d.data() } as Feedback));
}

// Special function for Firebase mode to replace WebSocket
export function listenToWsReplacement(codelabId: string, callback: (msg: any) => void) {
    // 1. Listen to chat
    const chatRef = rtdbRef(rtdb, `codelabs/${codelabId}/chat`);
    const chatUnsub = onChildAdded(chatRef, (snapshot) => {
        const data = snapshot.val();
        if (data && data.created_at) {
            // Only notify if it's recent (optional, but good for UX)
            callback({ type: "chat", ...data });
        }
    });

    // 2. Listen to help requests
    const helpRef = rtdbRef(rtdb, `codelabs/${codelabId}/help_requests`);
    const helpUnsub = onChildAdded(helpRef, (snapshot) => {
        const data = snapshot.val();
        callback({ type: "help_request", ...data });
    });

    // 3. Listen to attendee progress (for admin live view)
    const attendeesRef = rtdbRef(rtdb, `codelabs/${codelabId}/attendees`);
    const attendeesUnsub = onValue(attendeesRef, (snapshot) => {
        const data = snapshot.val();
        if (data) {
            callback({ type: "progress_update", attendees: data });
        }
    });

    // 4. Listen to inline comment changes (threads + messages)
    const inlineThreadsRef = rtdbRef(
        rtdb,
        `codelabs/${codelabId}/inline_comment_threads`,
    );
    const inlineMessagesRef = rtdbRef(
        rtdb,
        `codelabs/${codelabId}/inline_comment_messages`,
    );
    const inlineThreadAddedUnsub = onChildAdded(inlineThreadsRef, () => {
        callback({ type: "inline_comment_changed" });
    });
    const inlineThreadChangedUnsub = onChildChanged(inlineThreadsRef, () => {
        callback({ type: "inline_comment_changed" });
    });
    const inlineThreadRemovedUnsub = onChildRemoved(inlineThreadsRef, () => {
        callback({ type: "inline_comment_changed" });
    });
    const inlineMessageAddedUnsub = onChildAdded(inlineMessagesRef, () => {
        callback({ type: "inline_comment_changed" });
    });
    const inlineMessageChangedUnsub = onChildChanged(inlineMessagesRef, () => {
        callback({ type: "inline_comment_changed" });
    });
    const inlineMessageRemovedUnsub = onChildRemoved(inlineMessagesRef, () => {
        callback({ type: "inline_comment_changed" });
    });

    return () => {
        chatUnsub();
        helpUnsub();
        attendeesUnsub();
        inlineThreadAddedUnsub();
        inlineThreadChangedUnsub();
        inlineThreadRemovedUnsub();
        inlineMessageAddedUnsub();
        inlineMessageChangedUnsub();
        inlineMessageRemovedUnsub();
    };
}
