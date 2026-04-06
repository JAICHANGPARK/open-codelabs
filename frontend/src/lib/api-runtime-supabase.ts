import * as supabase from "./api-supabase";
import {
    alertExportBackupNotSupported,
    alertExportCodelabNotSupported,
    noOpAsync,
    returnEmptyList,
    throwImportCodelabNotSupported,
    throwInspectBackupNotSupported,
    throwLinkSubmissionNotSupported,
    throwNotSupportedInServerlessMode,
    throwRestoreBackupNotSupported,
    throwUpdateCheckNotSupported,
} from "./api-fallbacks";

export const ASSET_URL = "";

export const listCodelabs = supabase.listCodelabs;
export const getMyCodelabs = supabase.getMyCodelabs;
export const getJoinedCodelabs = supabase.getJoinedCodelabs;
export const getCodelab = supabase.getCodelab;
export const createCodelab = supabase.createCodelab;
export const updateCodelab = supabase.updateCodelab;
export const copyCodelab = throwNotSupportedInServerlessMode;
export const saveSteps = supabase.saveSteps;
export const deleteCodelab = supabase.deleteCodelab;
export const login = supabase.login;
export const saveAdminSettings = noOpAsync;
export const loginWithGoogle = supabase.loginWithGoogle;
export const logout = supabase.logout;
export const onAuthChange = supabase.onAuthChange;
export const getSession = supabase.getSession;

export const registerAttendee = supabase.registerAttendee;
export const updateAttendeeProgress = supabase.updateAttendeeProgress;
export const requestHelp = supabase.requestHelp;
export const getHelpRequests = supabase.getHelpRequests;
export const resolveHelpRequest = supabase.resolveHelpRequest;
export const getAttendees = supabase.getAttendees;
export const getChatHistory = supabase.getChatHistory;
export const getInlineComments = supabase.getInlineComments;
export const createInlineComment = supabase.createInlineComment;
export const replyInlineComment = supabase.replyInlineComment;
export const deleteInlineComment = supabase.deleteInlineComment;
export const sendChatMessage = supabase.sendChatMessage;
export const uploadImage = supabase.uploadImage;
export const submitFeedback = supabase.submitFeedback;
export const submitSubmissionLink = throwLinkSubmissionNotSupported;
export const getUpdateStatus = throwUpdateCheckNotSupported;
export const getFeedback = supabase.getFeedback;

export const completeCodelab = supabase.completeCodelab;
export const getCertificate = supabase.getCertificate;

export const getMaterials = supabase.getMaterials;
export const addMaterial = supabase.addMaterial;
export const deleteMaterial = supabase.deleteMaterial;
export const uploadMaterial = supabase.uploadMaterial;

export const submitFile = supabase.submitFile;
export const getSubmissions = supabase.getSubmissions;
export const deleteSubmission = supabase.deleteSubmission;

export const getQuizzes = supabase.getQuizzes;
export const updateQuizzes = supabase.updateQuizzes;
export const submitQuiz = supabase.submitQuiz;
export const getQuizSubmissions = supabase.getQuizSubmissions;

export const getWsUrl = () => "";
export const listenToWsReplacement = supabase.listenToWsReplacement;

export const isFirebaseMode = () => false;
export const isSupabaseMode = () => true;
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

