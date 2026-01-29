import * as backend from './api-backend';
import * as firebase from './api-firebase';
import type { Codelab, Step, Attendee, HelpRequest, ChatMessage, Feedback, Material, CertificateInfo, Quiz, QuizSubmissionPayload, QuizSubmissionWithAttendee, Submission, SubmissionWithAttendee } from './types';

export type { Codelab, Step, Attendee, HelpRequest, ChatMessage, Feedback, Material, CertificateInfo, Quiz, QuizSubmissionPayload, QuizSubmissionWithAttendee, Submission, SubmissionWithAttendee };

const USE_FIREBASE = import.meta.env.VITE_USE_FIREBASE === 'true';

export const ASSET_URL = USE_FIREBASE ? '' : backend.ASSET_URL;

export const listCodelabs = USE_FIREBASE ? firebase.listCodelabs : backend.listCodelabs;
export const getMyCodelabs = USE_FIREBASE ? firebase.getMyCodelabs : async () => [];
export const getJoinedCodelabs = USE_FIREBASE ? firebase.getJoinedCodelabs : async () => [];
export const getCodelab = USE_FIREBASE ? firebase.getCodelab : backend.getCodelab;
export const createCodelab = USE_FIREBASE ? firebase.createCodelab : backend.createCodelab;
export const updateCodelab = USE_FIREBASE ? firebase.updateCodelab : backend.updateCodelab;
export const copyCodelab = USE_FIREBASE ? async (_id: string) => { throw new Error('Not supported in Firebase mode'); } : backend.copyCodelab;
export const saveSteps = USE_FIREBASE ? firebase.saveSteps : backend.saveSteps;
export const deleteCodelab = USE_FIREBASE ? firebase.deleteCodelab : backend.deleteCodelab;
export const login = USE_FIREBASE ? firebase.login : backend.login;
export const saveAdminSettings = USE_FIREBASE ? async (_payload: { gemini_api_key: string }) => { /* not needed for firebase */ } : backend.saveAdminSettings;
export const loginWithGoogle = USE_FIREBASE ? firebase.loginWithGoogle : async () => { throw new Error('Not supported in backend mode'); };
export const logout = USE_FIREBASE ? firebase.logout : backend.logout;
export const onAuthChange = USE_FIREBASE ? firebase.onAuthChange : (cb: any) => { /* no-op */ };
export const getSession = USE_FIREBASE ? firebase.getSession : backend.getSession;

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
    ? async (_codelabId: string) => { /* Firebase logic needed */ } 
    : backend.completeCodelab;

export const getCertificate = USE_FIREBASE 
    ? async (_attendeeId: string) => { throw new Error('Not supported in Firebase mode'); } 
    : backend.getCertificate;

export const getMaterials = USE_FIREBASE 
    ? async (_codelabId: string) => [] 
    : backend.getMaterials;
export const addMaterial = USE_FIREBASE 
    ? async (_codelabId: string, _payload: { title: string; material_type: 'link' | 'file'; link_url?: string; file_path?: string }) => { throw new Error('Not supported in Firebase mode'); } 
    : backend.addMaterial;
export const deleteMaterial = USE_FIREBASE 
    ? async (_codelabId: string, _materialId: string) => { throw new Error('Not supported in Firebase mode'); } 
    : backend.deleteMaterial;
export const uploadMaterial = USE_FIREBASE 
    ? async (_file: File) => { throw new Error('Not supported in Firebase mode'); } 
    : backend.uploadMaterial;

export const submitFile = USE_FIREBASE 
    ? async (_codelabId: string, _attendeeId: string, _file: File) => { throw new Error('Not supported in Firebase mode'); } 
    : backend.submitFile;
export const getSubmissions = USE_FIREBASE 
    ? async (_codelabId: string) => [] 
    : backend.getSubmissions;
export const deleteSubmission = USE_FIREBASE 
    ? async (_codelabId: string, _attendeeId: string, _submissionId: string) => { throw new Error('Not supported in Firebase mode'); } 
    : backend.deleteSubmission;

export const getQuizzes = USE_FIREBASE
    ? async (_codelabId: string) => []
    : backend.getQuizzes;
export const updateQuizzes = USE_FIREBASE
    ? async (_codelabId: string, _quizzes: { question: string, options: string[], correct_answer: number }[]) => { throw new Error('Not supported in Firebase mode'); }
    : backend.updateQuizzes;
export const submitQuiz = USE_FIREBASE
    ? async (_codelabId: string, _payload: QuizSubmissionPayload) => { /* Not supported */ }
    : backend.submitQuiz;
export const getQuizSubmissions = USE_FIREBASE
    ? async (_codelabId: string) => []
    : backend.getQuizSubmissions;

// Export specialized functions
export const getWsUrl = backend.getWsUrl;
export const listenToWsReplacement = firebase.listenToWsReplacement;

// Export helper to check mode
export const isFirebaseMode = () => USE_FIREBASE;

// Unsupported in Firebase mode for now
export const exportCodelab = USE_FIREBASE 
    ? async (_id: string) => { alert('Export is not supported in Firebase mode yet.'); } 
    : backend.exportCodelab;

export const importCodelab = USE_FIREBASE
    ? async (_file: File) => { throw new Error('Import is not supported in Firebase mode yet.'); }
    : backend.importCodelab;

// Code Server API (Backend only)
export const createCodeServer = USE_FIREBASE
    ? async (_codelabId: string, _workspaceFiles?: backend.WorkspaceFile[], _structureType?: 'branch' | 'folder') => { throw new Error('Not supported in Firebase mode'); }
    : backend.createCodeServer;
export const getCodeServerInfo = USE_FIREBASE
    ? async (_codelabId: string) => { throw new Error('Not supported in Firebase mode'); }
    : backend.getCodeServerInfo;
export const createCodeServerBranch = USE_FIREBASE
    ? async (_codelabId: string, _stepNumber: number, _branchType: 'start' | 'end') => { throw new Error('Not supported in Firebase mode'); }
    : backend.createCodeServerBranch;
export const createCodeServerFolder = USE_FIREBASE
    ? async (_codelabId: string, _stepNumber: number, _folderType: 'start' | 'end', _files: backend.WorkspaceFile[]) => { throw new Error('Not supported in Firebase mode'); }
    : backend.createCodeServerFolder;
export const downloadCodeServerWorkspace = USE_FIREBASE
    ? async (_codelabId: string) => { throw new Error('Not supported in Firebase mode'); }
    : backend.downloadCodeServerWorkspace;
export const deleteCodeServer = USE_FIREBASE
    ? async (_codelabId: string) => { throw new Error('Not supported in Firebase mode'); }
    : backend.deleteCodeServer;
export type { CodeServerInfo, WorkspaceFile } from './api-backend';
