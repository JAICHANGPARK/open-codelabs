import * as backend from './api-backend';
import * as firebase from './api-firebase';
import type { Codelab, Step, Attendee, HelpRequest, ChatMessage, Feedback, Material, CertificateInfo, Quiz, QuizSubmissionPayload, QuizSubmissionWithAttendee } from './types';

export type { Codelab, Step, Attendee, HelpRequest, ChatMessage, Feedback, Material, CertificateInfo, Quiz, QuizSubmissionPayload, QuizSubmissionWithAttendee };

const USE_FIREBASE = import.meta.env.VITE_USE_FIREBASE === 'true';

export const ASSET_URL = USE_FIREBASE ? '' : backend.ASSET_URL;

export const listCodelabs = USE_FIREBASE ? firebase.listCodelabs : backend.listCodelabs;
export const getMyCodelabs = USE_FIREBASE ? firebase.getMyCodelabs : async () => [];
export const getJoinedCodelabs = USE_FIREBASE ? firebase.getJoinedCodelabs : async () => [];
export const getCodelab = USE_FIREBASE ? firebase.getCodelab : backend.getCodelab;
export const createCodelab = USE_FIREBASE ? firebase.createCodelab : backend.createCodelab;
export const updateCodelab = USE_FIREBASE ? firebase.updateCodelab : backend.updateCodelab;
export const saveSteps = USE_FIREBASE ? firebase.saveSteps : backend.saveSteps;
export const deleteCodelab = USE_FIREBASE ? firebase.deleteCodelab : backend.deleteCodelab;
export const login = USE_FIREBASE ? firebase.login : backend.login;
export const loginWithGoogle = USE_FIREBASE ? firebase.loginWithGoogle : async () => { throw new Error('Not supported in backend mode'); };
export const logout = USE_FIREBASE ? firebase.logout : async () => { localStorage.removeItem('adminToken'); };
export const onAuthChange = USE_FIREBASE ? firebase.onAuthChange : (cb: any) => { /* no-op */ };

export const registerAttendee = USE_FIREBASE ? firebase.registerAttendee : backend.registerAttendee;
export const updateAttendeeProgress = USE_FIREBASE ? firebase.updateAttendeeProgress : async (codelabId: string, attendeeId: string, stepNumber: number) => { /* WebSocket handles this */ };
export const requestHelp = USE_FIREBASE ? firebase.requestHelp : backend.requestHelp;
export const getHelpRequests = USE_FIREBASE ? firebase.getHelpRequests : backend.getHelpRequests;
export const resolveHelpRequest = USE_FIREBASE ? firebase.resolveHelpRequest : backend.resolveHelpRequest;
export const getAttendees = USE_FIREBASE ? firebase.getAttendees : backend.getAttendees;
export const getChatHistory = USE_FIREBASE ? firebase.getChatHistory : backend.getChatHistory;
export const sendChatMessage = USE_FIREBASE ? firebase.sendChatMessage : async (codelabId: string, payload: any) => { /* WebSocket takes care of this in backend mode */ };
export const uploadImage = USE_FIREBASE ? firebase.uploadImage : backend.uploadImage;
export const submitFeedback = USE_FIREBASE ? firebase.submitFeedback : backend.submitFeedback;
export const getFeedback = USE_FIREBASE ? firebase.getFeedback : backend.getFeedback;

export const completeCodelab = USE_FIREBASE 
    ? async () => { /* Firebase logic needed */ } 
    : backend.completeCodelab;

export const getCertificate = USE_FIREBASE 
    ? async () => { throw new Error('Not supported in Firebase mode'); } 
    : backend.getCertificate;

export const getMaterials = USE_FIREBASE 
    ? async () => [] 
    : backend.getMaterials;
export const addMaterial = USE_FIREBASE 
    ? async () => { throw new Error('Not supported in Firebase mode'); } 
    : backend.addMaterial;
export const deleteMaterial = USE_FIREBASE 
    ? async () => { throw new Error('Not supported in Firebase mode'); } 
    : backend.deleteMaterial;
export const uploadMaterial = USE_FIREBASE 
    ? async () => { throw new Error('Not supported in Firebase mode'); } 
    : backend.uploadMaterial;

export const getQuizzes = USE_FIREBASE
    ? async () => []
    : backend.getQuizzes;
export const updateQuizzes = USE_FIREBASE
    ? async () => { throw new Error('Not supported in Firebase mode'); }
    : backend.updateQuizzes;
export const submitQuiz = USE_FIREBASE
    ? async () => { /* Not supported */ }
    : backend.submitQuiz;
export const getQuizSubmissions = USE_FIREBASE
    ? async () => []
    : backend.getQuizSubmissions;

// Export specialized functions
export const getWsUrl = backend.getWsUrl;
export const listenToWsReplacement = firebase.listenToWsReplacement;

// Export helper to check mode
export const isFirebaseMode = () => USE_FIREBASE;

// Unsupported in Firebase mode for now
export const exportCodelab = USE_FIREBASE 
    ? async () => { alert('Export is not supported in Firebase mode yet.'); } 
    : backend.exportCodelab;

export const importCodelab = USE_FIREBASE 
    ? async () => { throw new Error('Import is not supported in Firebase mode yet.'); } 
    : backend.importCodelab;
