import { db, auth, storage } from './firebase';
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

export async function listCodelabs(): Promise<Codelab[]> {
    const q = query(collection(db, CODELABS_COLLECTION), orderBy("created_at", "desc"));
    const querySnapshot = await getDocs(q);
    return querySnapshot.docs.map(d => ({ id: d.id, ...d.data() } as Codelab));
}

export async function getCodelab(id: string): Promise<[Codelab, Step[]]> {
    const codelabDoc = await getDoc(doc(db, CODELABS_COLLECTION, id));
    if (!codelabDoc.exists()) throw new Error('Codelab not found');
    
    const codelab = { id: codelabDoc.id, ...codelabDoc.data() } as Codelab;
    
    const stepsSnapshot = await getDocs(query(collection(db, `${CODELABS_COLLECTION}/${id}/steps`), orderBy("step_number", "asc")));
    const steps = stepsSnapshot.docs.map(d => ({ id: d.id, ...d.data() } as Step));
    
    return [codelab, steps];
}

export async function createCodelab(payload: { title: string; description: string; author: string }): Promise<Codelab> {
    const docRef = await addDoc(collection(db, CODELABS_COLLECTION), {
        ...payload,
        created_at: serverTimestamp()
    });
    return { id: docRef.id, ...payload };
}

export async function updateCodelab(id: string, payload: { title: string; description: string; author: string }): Promise<Codelab> {
    const docRef = doc(db, CODELABS_COLLECTION, id);
    await updateDoc(docRef, payload);
    return { id, ...payload };
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

export async function login(admin_id: string, admin_pw: string): Promise<{ token: string }> {
    // Simple mock login for Firebase mode if not using full Firebase Auth
    // In a real scenario, we might check against a 'config' collection or environment variables.
    if (admin_id === import.meta.env.VITE_ADMIN_ID && admin_pw === import.meta.env.VITE_ADMIN_PW) {
        return { token: "firebase-token-mock" };
    }
    throw new Error('Invalid credentials');
}

export async function registerAttendee(codelabId: string, name: string, code: string): Promise<Attendee> {
    const attendeesCollection = collection(db, `${CODELABS_COLLECTION}/${codelabId}/attendees`);
    
    // Check duplicate
    const q = query(attendeesCollection, where("name", "==", name));
    const snapshot = await getDocs(q);
    if (!snapshot.empty) throw new Error('DUPLICATE_NAME');
    
    const docRef = await addDoc(attendeesCollection, {
        name,
        code,
        codelab_id: codelabId,
        current_step: 1
    });
    
    return { id: docRef.id, name, code, codelab_id: codelabId, current_step: 1 };
}

export async function updateAttendeeProgress(codelabId: string, attendeeId: string, stepNumber: number): Promise<void> {
    const docRef = doc(db, `${CODELABS_COLLECTION}/${codelabId}/attendees`, attendeeId);
    await updateDoc(docRef, { current_step: stepNumber });
}

export async function requestHelp(codelabId: string, attendeeId: string, stepNumber: number): Promise<void> {
    const helpCollection = collection(db, `${CODELABS_COLLECTION}/${codelabId}/help_requests`);
    const attendeeDoc = await getDoc(doc(db, `${CODELABS_COLLECTION}/${codelabId}/attendees`, attendeeId));
    const attendeeName = attendeeDoc.exists() ? attendeeDoc.data().name : "Unknown";

    await addDoc(helpCollection, {
        codelab_id: codelabId,
        attendee_id: attendeeId,
        attendee_name: attendeeName,
        step_number: stepNumber,
        status: "pending",
        created_at: serverTimestamp()
    });
}

export async function getHelpRequests(codelabId: string): Promise<HelpRequest[]> {
    const q = query(collection(db, `${CODELABS_COLLECTION}/${codelabId}/help_requests`), orderBy("created_at", "asc"));
    const snapshot = await getDocs(q);
    return snapshot.docs.map(d => ({ id: d.id, ...d.data() } as HelpRequest));
}

export async function resolveHelpRequest(codelabId: string, helpId: string): Promise<void> {
    const docRef = doc(db, `${CODELABS_COLLECTION}/${codelabId}/help_requests`, helpId);
    await updateDoc(docRef, { status: "resolved" });
}

export async function getAttendees(codelabId: string): Promise<Attendee[]> {
    const snapshot = await getDocs(collection(db, `${CODELABS_COLLECTION}/${codelabId}/attendees`));
    return snapshot.docs.map(d => ({ id: d.id, ...d.data() } as Attendee));
}

export async function getChatHistory(codelabId: string): Promise<ChatMessage[]> {
    const q = query(collection(db, `${CODELABS_COLLECTION}/${codelabId}/chat`), orderBy("created_at", "asc"));
    const snapshot = await getDocs(q);
    return snapshot.docs.map(d => ({ id: d.id, ...d.data() } as ChatMessage));
}

export async function sendChatMessage(codelabId: string, payload: { sender: string, message: string, type: 'chat' | 'dm', target_id?: string }): Promise<void> {
    const chatCollection = collection(db, `${CODELABS_COLLECTION}/${codelabId}/chat`);
    await addDoc(chatCollection, {
        codelab_id: codelabId,
        sender_name: payload.sender,
        message: payload.message,
        msg_type: payload.type,
        target_id: payload.target_id || null,
        created_at: serverTimestamp()
    });
}

export async function uploadImage(file: File): Promise<{ url: string }> {
    const storageRef = ref(storage, `images/${Date.now()}_${file.name}`);
    await uploadBytes(storageRef, file);
    const url = await getDownloadURL(storageRef);
    return { url };
}

export async function submitFeedback(codelabId: string, payload: { difficulty: number; satisfaction: number; comments: string; attendee_id: string }): Promise<void> {
    const feedbackCollection = collection(db, `${CODELABS_COLLECTION}/${codelabId}/feedback`);
    
    // Check if already submitted
    const q = query(feedbackCollection, where("attendee_id", "==", payload.attendee_id));
    const snapshot = await getDocs(q);
    if (!snapshot.empty) throw new Error('ALREADY_SUBMITTED');

    await addDoc(feedbackCollection, {
        codelab_id: codelabId,
        attendee_id: payload.attendee_id,
        difficulty: payload.difficulty.toString(),
        satisfaction: payload.satisfaction.toString(),
        comment: payload.comments,
        created_at: serverTimestamp()
    });
}

export async function getFeedback(codelabId: string): Promise<Feedback[]> {
    const q = query(collection(db, `${CODELABS_COLLECTION}/${codelabId}/feedback`), orderBy("created_at", "desc"));
    const snapshot = await getDocs(q);
    return snapshot.docs.map(d => ({ id: d.id, ...d.data() } as Feedback));
}

// Special function for Firebase mode to replace WebSocket
export function listenToWsReplacement(codelabId: string, callback: (msg: any) => void) {
    // 1. Listen to chat
    const chatUnsub = onSnapshot(query(collection(db, `${CODELABS_COLLECTION}/${codelabId}/chat`), orderBy("created_at", "desc"), where("created_at", ">", new Date())), (snapshot) => {
        snapshot.docChanges().forEach((change) => {
            if (change.type === "added") {
                callback({ type: "chat", ...change.doc.data() });
            }
        });
    });

    // 2. Listen to help requests
    const helpUnsub = onSnapshot(collection(db, `${CODELABS_COLLECTION}/${codelabId}/help_requests`), (snapshot) => {
        snapshot.docChanges().forEach((change) => {
            if (change.type === "added" || change.type === "modified") {
                callback({ type: "help_request", ...change.doc.data() });
            }
        });
    });

    return () => {
        chatUnsub();
        helpUnsub();
    };
}
