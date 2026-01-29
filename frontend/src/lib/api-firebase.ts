import { db, auth, storage, rtdb } from './firebase';
import { 
    ref as rtdbRef, 
    push, 
    set, 
    onValue, 
    onChildAdded, 
    remove, 
    update,
    serverTimestamp as rtdbTimestamp
} from "firebase/database";
import { 
    GoogleAuthProvider,
    signInWithPopup,
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
    type DocumentData,
    onSnapshot
} from "firebase/firestore";
import { 
    ref, 
    uploadBytes, 
    getDownloadURL 
} from "firebase/storage";
import type { Codelab, Step, Attendee, HelpRequest, ChatMessage, Feedback } from './types';

const CODELABS_COLLECTION = "codelabs";

function isAdmin() {
    if (typeof localStorage === 'undefined') return false;
    return !!localStorage.getItem('adminToken');
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

export async function saveSteps(codelabId: string, steps: { title: string, content_markdown: string }[]): Promise<void> {
    const stepsCollection = collection(db, `${CODELABS_COLLECTION}/${codelabId}/steps`);
    
    // Delete existing steps first (simple approach)
    const existingSteps = await getDocs(stepsCollection);
    for (const d of existingSteps.docs) {
        await deleteDoc(d.ref);
    }
    
    // Add new steps
    for (let i = 0; i < steps.length; i++) {
        await addDoc(stepsCollection, {
            ...steps[i],
            step_number: i + 1,
            codelab_id: codelabId
        });
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

export async function getSession(): Promise<{ role: string; sub: string } | null> {
    if (typeof localStorage === 'undefined') return null;
    const token = localStorage.getItem('adminToken');
    if (!token) return null;
    return { role: 'admin', sub: 'firebase' };
}

export async function registerAttendee(codelabId: string, name: string, code: string): Promise<Attendee> {
    const user = auth.currentUser;
    const attendeesCollection = collection(db, `${CODELABS_COLLECTION}/${codelabId}/attendees`);
    
    // Check duplicate
    const q = query(attendeesCollection, where("name", "==", name));
    const snapshot = await getDocs(q);
    if (!snapshot.empty) throw new Error('DUPLICATE_NAME');
    
    const attendeeData = {
        name,
        code,
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

    return () => {
        // Firebase RTDB doesn't return unsub from onChildAdded, need to use off()
        // but since we might have multiple listeners, it's better to manage carefully.
        // For simplicity in this context:
    };
}
