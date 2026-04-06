import * as firebase from "./api-firebase";
import {
    alertExportBackupNotSupported,
    alertExportCodelabNotSupported,
    noOpAsync,
    returnEmptyList,
    throwImportCodelabNotSupported,
    throwInspectBackupNotSupported,
    throwLinkSubmissionNotSupported,
    throwNotSupportedInFirebaseMode,
    throwNotSupportedInServerlessMode,
    throwRestoreBackupNotSupported,
    throwUpdateCheckNotSupported,
} from "./api-fallbacks";

export const ASSET_URL = "";

export const listCodelabs = firebase.listCodelabs;
export const getMyCodelabs = firebase.getMyCodelabs;
export const getJoinedCodelabs = firebase.getJoinedCodelabs;
export const getCodelab = firebase.getCodelab;
export const createCodelab = firebase.createCodelab;
export const updateCodelab = firebase.updateCodelab;
export const copyCodelab = throwNotSupportedInServerlessMode;
export const saveSteps = firebase.saveSteps;
export const deleteCodelab = firebase.deleteCodelab;
export const login = firebase.login;
export const saveAdminSettings = noOpAsync;
export const loginWithGoogle = firebase.loginWithGoogle;
export const logout = firebase.logout;
export const onAuthChange = firebase.onAuthChange;
export const getSession = firebase.getSession;

export const registerAttendee = firebase.registerAttendee;
export const updateAttendeeProgress = firebase.updateAttendeeProgress;
export const requestHelp = firebase.requestHelp;
export const getHelpRequests = firebase.getHelpRequests;
export const resolveHelpRequest = firebase.resolveHelpRequest;
export const getAttendees = firebase.getAttendees;
export const getChatHistory = firebase.getChatHistory;
export const getInlineComments = firebase.getInlineComments;
export const createInlineComment = firebase.createInlineComment;
export const replyInlineComment = firebase.replyInlineComment;
export const deleteInlineComment = firebase.deleteInlineComment;
export const sendChatMessage = firebase.sendChatMessage;
export const uploadImage = firebase.uploadImage;
export const submitFeedback = firebase.submitFeedback;
export const submitSubmissionLink = throwLinkSubmissionNotSupported;
export const getUpdateStatus = throwUpdateCheckNotSupported;
export const getFeedback = firebase.getFeedback;

export const completeCodelab = noOpAsync;
export const getCertificate = throwNotSupportedInFirebaseMode;

export const getMaterials = returnEmptyList;
export const addMaterial = throwNotSupportedInFirebaseMode;
export const deleteMaterial = throwNotSupportedInFirebaseMode;
export const uploadMaterial = throwNotSupportedInFirebaseMode;

export const submitFile = throwNotSupportedInFirebaseMode;
export const getSubmissions = returnEmptyList;
export const deleteSubmission = throwNotSupportedInFirebaseMode;

export const getQuizzes = returnEmptyList;
export const updateQuizzes = throwNotSupportedInFirebaseMode;
export const submitQuiz = noOpAsync;
export const getQuizSubmissions = returnEmptyList;

export const getWsUrl = () => "";
export const listenToWsReplacement = firebase.listenToWsReplacement;

export const isFirebaseMode = () => true;
export const isSupabaseMode = () => false;
export const isServerlessMode = () => true;

export const exportCodelab = alertExportCodelabNotSupported;
export const importCodelab = throwImportCodelabNotSupported;
export const exportBackup = alertExportBackupNotSupported;
export const restoreBackup = throwRestoreBackupNotSupported;
export const inspectBackup = throwInspectBackupNotSupported;

export const createCodeServer = throwNotSupportedInServerlessMode;
export const getCodeServerInfo = throwNotSupportedInServerlessMode;
export const createCodeServerBranch = throwNotSupportedInServerlessMode;
export const createCodeServerFolder = throwNotSupportedInServerlessMode;
export const downloadCodeServerWorkspace = throwNotSupportedInServerlessMode;
export const deleteCodeServer = throwNotSupportedInServerlessMode;

export const saveAiConversation = noOpAsync;
export const getAiConversations = returnEmptyList;

