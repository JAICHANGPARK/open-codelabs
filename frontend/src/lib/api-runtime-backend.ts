import * as backend from "./api-backend";
import {
    backendNoopUnsubscribe,
    noOpAsync,
    returnEmptyList,
    throwNotSupportedInBackendMode,
} from "./api-fallbacks";

export const ASSET_URL = backend.ASSET_URL;

export const listCodelabs = backend.listCodelabs;
export const getMyCodelabs = returnEmptyList;
export const getJoinedCodelabs = returnEmptyList;
export const getCodelab = backend.getCodelab;
export const createCodelab = backend.createCodelab;
export const updateCodelab = backend.updateCodelab;
export const copyCodelab = backend.copyCodelab;
export const saveSteps = backend.saveSteps;
export const deleteCodelab = backend.deleteCodelab;
export const login = backend.login;
export const saveAdminSettings = backend.saveAdminSettings;
export const loginWithGoogle = throwNotSupportedInBackendMode;
export const logout = backend.logout;
export const onAuthChange = backendNoopUnsubscribe;
export const getSession = backend.getSession;

export const registerAttendee = backend.registerAttendee;
export const updateAttendeeProgress = noOpAsync;
export const requestHelp = backend.requestHelp;
export const getHelpRequests = backend.getHelpRequests;
export const resolveHelpRequest = backend.resolveHelpRequest;
export const getAttendees = backend.getAttendees;
export const getChatHistory = backend.getChatHistory;
export const getInlineComments = backend.getInlineComments;
export const createInlineComment = backend.createInlineComment;
export const replyInlineComment = backend.replyInlineComment;
export const deleteInlineComment = backend.deleteInlineComment;
export const sendChatMessage = noOpAsync;
export const uploadImage = backend.uploadImage;
export const submitFeedback = backend.submitFeedback;
export const submitSubmissionLink = backend.submitSubmissionLink;
export const getUpdateStatus = backend.getUpdateStatus;
export const getFeedback = backend.getFeedback;

export const completeCodelab = backend.completeCodelab;
export const getCertificate = backend.getCertificate;

export const getMaterials = backend.getMaterials;
export const addMaterial = backend.addMaterial;
export const deleteMaterial = backend.deleteMaterial;
export const uploadMaterial = backend.uploadMaterial;

export const submitFile = backend.submitFile;
export const getSubmissions = backend.getSubmissions;
export const deleteSubmission = backend.deleteSubmission;

export const getQuizzes = backend.getQuizzes;
export const updateQuizzes = backend.updateQuizzes;
export const submitQuiz = backend.submitQuiz;
export const getQuizSubmissions = backend.getQuizSubmissions;

export const getWsUrl = backend.getWsUrl;
export function listenToWsReplacement(
    _codelabId: string,
    _callback: (msg: unknown) => void,
) {
    return () => {};
}

export const isFirebaseMode = () => false;
export const isSupabaseMode = () => false;
export const isServerlessMode = () => false;

export const exportCodelab = backend.exportCodelab;
export const importCodelab = backend.importCodelab;
export const exportBackup = backend.exportBackup;
export const restoreBackup = backend.restoreBackup;
export const inspectBackup = backend.inspectBackup;

export const createCodeServer = backend.createCodeServer;
export const getCodeServerInfo = backend.getCodeServerInfo;
export const createCodeServerBranch = backend.createCodeServerBranch;
export const createCodeServerFolder = backend.createCodeServerFolder;
export const downloadCodeServerWorkspace = backend.downloadCodeServerWorkspace;
export const deleteCodeServer = backend.deleteCodeServer;

export const saveAiConversation = backend.saveAiConversation;
export const getAiConversations = backend.getAiConversations;

