import * as backend from './api-backend';
import * as firebase from './api-firebase';
import * as supabase from './api-supabase';
import type { Codelab, Step, Attendee, HelpRequest, ChatMessage, Feedback, Material, CertificateInfo, Quiz, QuizSubmissionPayload, QuizSubmissionWithAttendee, Submission, SubmissionWithAttendee, AiConversation, SaveAiConversationPayload } from './types';

export type { Codelab, Step, Attendee, HelpRequest, ChatMessage, Feedback, Material, CertificateInfo, Quiz, QuizSubmissionPayload, QuizSubmissionWithAttendee, Submission, SubmissionWithAttendee, AiConversation, SaveAiConversationPayload };

const USE_SUPABASE = import.meta.env.VITE_USE_SUPABASE === 'true';
const USE_FIREBASE = import.meta.env.VITE_USE_FIREBASE === 'true' && !USE_SUPABASE;
const USE_SERVERLESS = USE_FIREBASE || USE_SUPABASE;

export const ASSET_URL = USE_SERVERLESS ? '' : backend.ASSET_URL;

export const listCodelabs = USE_SUPABASE
    ? supabase.listCodelabs
    : USE_FIREBASE
        ? firebase.listCodelabs
        : backend.listCodelabs;
export const getMyCodelabs = USE_SUPABASE
    ? supabase.getMyCodelabs
    : USE_FIREBASE
        ? firebase.getMyCodelabs
        : async () => [];
export const getJoinedCodelabs = USE_SUPABASE
    ? supabase.getJoinedCodelabs
    : USE_FIREBASE
        ? firebase.getJoinedCodelabs
        : async () => [];
export const getCodelab = USE_SUPABASE
    ? supabase.getCodelab
    : USE_FIREBASE
        ? firebase.getCodelab
        : backend.getCodelab;
export const createCodelab = USE_SUPABASE
    ? supabase.createCodelab
    : USE_FIREBASE
        ? firebase.createCodelab
        : backend.createCodelab;
export const updateCodelab = USE_SUPABASE
    ? supabase.updateCodelab
    : USE_FIREBASE
        ? firebase.updateCodelab
        : backend.updateCodelab;
export const copyCodelab = USE_SERVERLESS
    ? async (_id: string) => { throw new Error('Not supported in serverless mode'); }
    : backend.copyCodelab;
export const saveSteps = USE_SUPABASE
    ? supabase.saveSteps
    : USE_FIREBASE
        ? firebase.saveSteps
        : backend.saveSteps;
export const deleteCodelab = USE_SUPABASE
    ? supabase.deleteCodelab
    : USE_FIREBASE
        ? firebase.deleteCodelab
        : backend.deleteCodelab;
export const login = USE_SUPABASE
    ? supabase.login
    : USE_FIREBASE
        ? firebase.login
        : backend.login;
export const saveAdminSettings = USE_SERVERLESS
    ? async (_payload: { gemini_api_key: string }) => { /* not needed for serverless */ }
    : backend.saveAdminSettings;
export const loginWithGoogle = USE_SUPABASE
    ? supabase.loginWithGoogle
    : USE_FIREBASE
        ? firebase.loginWithGoogle
        : async () => { throw new Error('Not supported in backend mode'); };
export const logout = USE_SUPABASE
    ? supabase.logout
    : USE_FIREBASE
        ? firebase.logout
        : backend.logout;
export const onAuthChange = USE_SUPABASE
    ? supabase.onAuthChange
    : USE_FIREBASE
        ? firebase.onAuthChange
        : (cb: any) => { /* no-op */ };
export const getSession = USE_SUPABASE
    ? supabase.getSession
    : USE_FIREBASE
        ? firebase.getSession
        : backend.getSession;

export const registerAttendee = USE_SUPABASE
    ? supabase.registerAttendee
    : USE_FIREBASE
        ? firebase.registerAttendee
        : backend.registerAttendee;
export const updateAttendeeProgress = USE_SUPABASE
    ? supabase.updateAttendeeProgress
    : USE_FIREBASE
        ? firebase.updateAttendeeProgress
        : async (_codelabId: string, _attendeeId: string, _stepNumber: number) => { /* WebSocket handles this */ };
export const requestHelp = USE_SUPABASE
    ? supabase.requestHelp
    : USE_FIREBASE
        ? firebase.requestHelp
        : backend.requestHelp;
export const getHelpRequests = USE_SUPABASE
    ? supabase.getHelpRequests
    : USE_FIREBASE
        ? firebase.getHelpRequests
        : backend.getHelpRequests;
export const resolveHelpRequest = USE_SUPABASE
    ? supabase.resolveHelpRequest
    : USE_FIREBASE
        ? firebase.resolveHelpRequest
        : backend.resolveHelpRequest;
export const getAttendees = USE_SUPABASE
    ? supabase.getAttendees
    : USE_FIREBASE
        ? firebase.getAttendees
        : backend.getAttendees;
export const getChatHistory = USE_SUPABASE
    ? supabase.getChatHistory
    : USE_FIREBASE
        ? firebase.getChatHistory
        : backend.getChatHistory;
export const sendChatMessage = USE_SUPABASE
    ? supabase.sendChatMessage
    : USE_FIREBASE
        ? firebase.sendChatMessage
        : async (_codelabId: string, _payload: any) => { /* WebSocket takes care of this in backend mode */ };
export const uploadImage = USE_SUPABASE
    ? supabase.uploadImage
    : USE_FIREBASE
        ? firebase.uploadImage
        : backend.uploadImage;
export const submitFeedback = USE_SUPABASE
    ? supabase.submitFeedback
    : USE_FIREBASE
        ? firebase.submitFeedback
        : backend.submitFeedback;
export const submitSubmissionLink = USE_SUPABASE
    ? async () => {
          throw new Error('Link submission not supported');
      }
    : USE_FIREBASE
        ? async () => {
              throw new Error('Link submission not supported');
          }
        : backend.submitSubmissionLink;

export const getUpdateStatus = USE_SUPABASE
    ? async () => {
          throw new Error('Update check not supported');
      }
    : USE_FIREBASE
        ? async () => {
              throw new Error('Update check not supported');
          }
        : backend.getUpdateStatus;
export const getFeedback = USE_SUPABASE
    ? supabase.getFeedback
    : USE_FIREBASE
        ? firebase.getFeedback
        : backend.getFeedback;

export const completeCodelab = USE_SUPABASE
    ? supabase.completeCodelab
    : USE_FIREBASE 
        ? async (_codelabId: string) => { /* Firebase logic needed */ } 
        : backend.completeCodelab;

export const getCertificate = USE_SUPABASE
    ? supabase.getCertificate
    : USE_FIREBASE 
        ? async (_attendeeId: string) => { throw new Error('Not supported in Firebase mode'); } 
        : backend.getCertificate;

export const getMaterials = USE_SUPABASE
    ? supabase.getMaterials
    : USE_FIREBASE 
        ? async (_codelabId: string) => [] 
        : backend.getMaterials;
export const addMaterial = USE_SUPABASE
    ? supabase.addMaterial
    : USE_FIREBASE 
        ? async (_codelabId: string, _payload: { title: string; material_type: 'link' | 'file'; link_url?: string; file_path?: string }) => { throw new Error('Not supported in Firebase mode'); } 
        : backend.addMaterial;
export const deleteMaterial = USE_SUPABASE
    ? supabase.deleteMaterial
    : USE_FIREBASE 
        ? async (_codelabId: string, _materialId: string) => { throw new Error('Not supported in Firebase mode'); } 
        : backend.deleteMaterial;
export const uploadMaterial = USE_SUPABASE
    ? supabase.uploadMaterial
    : USE_FIREBASE 
        ? async (_file: File) => { throw new Error('Not supported in Firebase mode'); } 
        : backend.uploadMaterial;

export const submitFile = USE_SUPABASE
    ? supabase.submitFile
    : USE_FIREBASE 
        ? async (_codelabId: string, _attendeeId: string, _file: File) => { throw new Error('Not supported in Firebase mode'); } 
        : backend.submitFile;
export const getSubmissions = USE_SUPABASE
    ? supabase.getSubmissions
    : USE_FIREBASE 
        ? async (_codelabId: string) => [] 
        : backend.getSubmissions;
export const deleteSubmission = USE_SUPABASE
    ? supabase.deleteSubmission
    : USE_FIREBASE 
        ? async (_codelabId: string, _attendeeId: string, _submissionId: string) => { throw new Error('Not supported in Firebase mode'); } 
        : backend.deleteSubmission;

export const getQuizzes = USE_SUPABASE
    ? supabase.getQuizzes
    : USE_FIREBASE
        ? async (_codelabId: string) => []
        : backend.getQuizzes;
export const updateQuizzes = USE_SUPABASE
    ? supabase.updateQuizzes
    : USE_FIREBASE
        ? async (_codelabId: string, _quizzes: { question: string, options: string[], correct_answer: number }[]) => { throw new Error('Not supported in Firebase mode'); }
        : backend.updateQuizzes;
export const submitQuiz = USE_SUPABASE
    ? supabase.submitQuiz
    : USE_FIREBASE
        ? async (_codelabId: string, _payload: QuizSubmissionPayload) => { /* Not supported */ }
        : backend.submitQuiz;
export const getQuizSubmissions = USE_SUPABASE
    ? supabase.getQuizSubmissions
    : USE_FIREBASE
        ? async (_codelabId: string) => []
        : backend.getQuizSubmissions;

// Export specialized functions
export const getWsUrl = backend.getWsUrl;
export const listenToWsReplacement = USE_SUPABASE
    ? supabase.listenToWsReplacement
    : firebase.listenToWsReplacement;

// Export helper to check mode
export const isFirebaseMode = () => USE_FIREBASE;
export const isSupabaseMode = () => USE_SUPABASE;
export const isServerlessMode = () => USE_SERVERLESS;

// Unsupported in Firebase mode for now
export const exportCodelab = USE_SERVERLESS 
    ? async (_id: string) => { alert('Export is not supported in serverless mode yet.'); } 
    : backend.exportCodelab;

export const importCodelab = USE_SERVERLESS
    ? async (_file: File) => { throw new Error('Import is not supported in serverless mode yet.'); }
    : backend.importCodelab;

export const exportBackup = USE_SERVERLESS
    ? async () => { alert('Backup export is not supported in serverless mode yet.'); }
    : backend.exportBackup;

export const restoreBackup = USE_SERVERLESS
    ? async (_file: File) => { throw new Error('Backup restore is not supported in serverless mode yet.'); }
    : backend.restoreBackup;

export const inspectBackup = USE_SERVERLESS
    ? async (_file: File) => { throw new Error('Backup inspect is not supported in serverless mode yet.'); }
    : backend.inspectBackup;

// Code Server API (Backend only)
export const createCodeServer = USE_SERVERLESS
    ? async (_codelabId: string, _workspaceFiles?: backend.WorkspaceFile[], _structureType?: 'branch' | 'folder') => { throw new Error('Not supported in serverless mode'); }
    : backend.createCodeServer;
export const getCodeServerInfo = USE_SERVERLESS
    ? async (_codelabId: string) => { throw new Error('Not supported in serverless mode'); }
    : backend.getCodeServerInfo;
export const createCodeServerBranch = USE_SERVERLESS
    ? async (_codelabId: string, _stepNumber: number, _branchType: 'start' | 'end') => { throw new Error('Not supported in serverless mode'); }
    : backend.createCodeServerBranch;
export const createCodeServerFolder = USE_SERVERLESS
    ? async (_codelabId: string, _stepNumber: number, _folderType: 'start' | 'end', _files: backend.WorkspaceFile[]) => { throw new Error('Not supported in serverless mode'); }
    : backend.createCodeServerFolder;
export const downloadCodeServerWorkspace = USE_SERVERLESS
    ? async (_codelabId: string) => { throw new Error('Not supported in serverless mode'); }
    : backend.downloadCodeServerWorkspace;
export const deleteCodeServer = USE_SERVERLESS
    ? async (_codelabId: string) => { throw new Error('Not supported in serverless mode'); }
    : backend.deleteCodeServer;
export type { CodeServerInfo, WorkspaceFile } from './api-backend';

// AI Conversation API (Backend only)
export const saveAiConversation = USE_SERVERLESS
    ? async (_payload: SaveAiConversationPayload) => { /* Not supported in serverless mode */ }
    : backend.saveAiConversation;
export const getAiConversations = USE_SERVERLESS
    ? async (_codelabId: string) => []
    : backend.getAiConversations;
