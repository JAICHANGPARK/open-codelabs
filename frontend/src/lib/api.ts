import * as backend from "./api-backend";
import * as firebase from "./api-firebase";
import * as supabase from "./api-supabase";
import {
    alertExportBackupNotSupported,
    alertExportCodelabNotSupported,
    backendNoopUnsubscribe,
    noOpAsync,
    returnEmptyList,
    throwImportCodelabNotSupported,
    throwInspectBackupNotSupported,
    throwLinkSubmissionNotSupported,
    throwNotSupportedInBackendMode,
    throwNotSupportedInFirebaseMode,
    throwNotSupportedInServerlessMode,
    throwRestoreBackupNotSupported,
    throwUpdateCheckNotSupported,
} from "./api-fallbacks";
import type {
    AiConversation,
    Attendee,
    CertificateInfo,
    ChatMessage,
    Codelab,
    CreateInlineCommentPayload,
    Feedback,
    HelpRequest,
    InlineCommentMessage,
    InlineCommentTarget,
    InlineCommentThread,
    Material,
    Quiz,
    QuizSubmissionPayload,
    QuizSubmissionWithAttendee,
    SaveAiConversationPayload,
    Step,
    Submission,
    SubmissionWithAttendee,
} from "./types";

export type {
    AiConversation,
    Attendee,
    CertificateInfo,
    ChatMessage,
    Codelab,
    CreateInlineCommentPayload,
    Feedback,
    HelpRequest,
    InlineCommentMessage,
    InlineCommentTarget,
    InlineCommentThread,
    Material,
    Quiz,
    QuizSubmissionPayload,
    QuizSubmissionWithAttendee,
    SaveAiConversationPayload,
    Step,
    Submission,
    SubmissionWithAttendee,
};

type Mode = "backend" | "firebase" | "supabase";

const MODE: Mode =
    import.meta.env.VITE_USE_SUPABASE === "true"
        ? "supabase"
        : import.meta.env.VITE_USE_FIREBASE === "true"
            ? "firebase"
            : "backend";
const USE_SERVERLESS = MODE !== "backend";

const selectByMode = <T>(handlers: { backend: T; firebase: T; supabase: T }): T => handlers[MODE];
const noOpSaveAiConversation = noOpAsync as unknown as typeof backend.saveAiConversation;

export const ASSET_URL = USE_SERVERLESS ? "" : backend.ASSET_URL;

export const listCodelabs = selectByMode({
    backend: backend.listCodelabs,
    firebase: firebase.listCodelabs,
    supabase: supabase.listCodelabs,
});
export const getMyCodelabs = selectByMode({
    backend: returnEmptyList,
    firebase: firebase.getMyCodelabs,
    supabase: supabase.getMyCodelabs,
});
export const getJoinedCodelabs = selectByMode({
    backend: returnEmptyList,
    firebase: firebase.getJoinedCodelabs,
    supabase: supabase.getJoinedCodelabs,
});
export const getCodelab = selectByMode({
    backend: backend.getCodelab,
    firebase: firebase.getCodelab,
    supabase: supabase.getCodelab,
});
export const createCodelab = selectByMode({
    backend: backend.createCodelab,
    firebase: firebase.createCodelab,
    supabase: supabase.createCodelab,
});
export const updateCodelab = selectByMode({
    backend: backend.updateCodelab,
    firebase: firebase.updateCodelab,
    supabase: supabase.updateCodelab,
});
export const copyCodelab = selectByMode({
    backend: backend.copyCodelab,
    firebase: throwNotSupportedInServerlessMode,
    supabase: throwNotSupportedInServerlessMode,
});
export const saveSteps = selectByMode({
    backend: backend.saveSteps,
    firebase: firebase.saveSteps,
    supabase: supabase.saveSteps,
});
export const deleteCodelab = selectByMode({
    backend: backend.deleteCodelab,
    firebase: firebase.deleteCodelab,
    supabase: supabase.deleteCodelab,
});
export const login = selectByMode({
    backend: backend.login,
    firebase: firebase.login,
    supabase: supabase.login,
});
export const saveAdminSettings = selectByMode({
    backend: backend.saveAdminSettings,
    firebase: noOpAsync,
    supabase: noOpAsync,
});
export const loginWithGoogle = selectByMode({
    backend: throwNotSupportedInBackendMode,
    firebase: firebase.loginWithGoogle,
    supabase: supabase.loginWithGoogle,
});
export const logout = selectByMode({
    backend: backend.logout,
    firebase: firebase.logout,
    supabase: supabase.logout,
});
export const onAuthChange = selectByMode({
    backend: backendNoopUnsubscribe,
    firebase: firebase.onAuthChange,
    supabase: supabase.onAuthChange,
});
export const getSession = selectByMode({
    backend: backend.getSession,
    firebase: firebase.getSession,
    supabase: supabase.getSession,
});

export const registerAttendee = selectByMode({
    backend: backend.registerAttendee,
    firebase: firebase.registerAttendee,
    supabase: supabase.registerAttendee,
});
export const updateAttendeeProgress = selectByMode({
    backend: noOpAsync,
    firebase: firebase.updateAttendeeProgress,
    supabase: supabase.updateAttendeeProgress,
});
export const requestHelp = selectByMode({
    backend: backend.requestHelp,
    firebase: firebase.requestHelp,
    supabase: supabase.requestHelp,
});
export const getHelpRequests = selectByMode({
    backend: backend.getHelpRequests,
    firebase: firebase.getHelpRequests,
    supabase: supabase.getHelpRequests,
});
export const resolveHelpRequest = selectByMode({
    backend: backend.resolveHelpRequest,
    firebase: firebase.resolveHelpRequest,
    supabase: supabase.resolveHelpRequest,
});
export const getAttendees = selectByMode({
    backend: backend.getAttendees,
    firebase: firebase.getAttendees,
    supabase: supabase.getAttendees,
});
export const getChatHistory = selectByMode({
    backend: backend.getChatHistory,
    firebase: firebase.getChatHistory,
    supabase: supabase.getChatHistory,
});
export const getInlineComments = selectByMode({
    backend: backend.getInlineComments,
    firebase: firebase.getInlineComments,
    supabase: supabase.getInlineComments,
});
export const createInlineComment = selectByMode({
    backend: backend.createInlineComment,
    firebase: firebase.createInlineComment,
    supabase: supabase.createInlineComment,
});
export const replyInlineComment = selectByMode({
    backend: backend.replyInlineComment,
    firebase: firebase.replyInlineComment,
    supabase: supabase.replyInlineComment,
});
export const deleteInlineComment = selectByMode({
    backend: backend.deleteInlineComment,
    firebase: firebase.deleteInlineComment,
    supabase: supabase.deleteInlineComment,
});
export const sendChatMessage = selectByMode({
    backend: noOpAsync,
    firebase: firebase.sendChatMessage,
    supabase: supabase.sendChatMessage,
});
export const uploadImage = selectByMode({
    backend: backend.uploadImage,
    firebase: firebase.uploadImage,
    supabase: supabase.uploadImage,
});
export const submitFeedback = selectByMode({
    backend: backend.submitFeedback,
    firebase: firebase.submitFeedback,
    supabase: supabase.submitFeedback,
});
export const submitSubmissionLink = selectByMode({
    backend: backend.submitSubmissionLink,
    firebase: throwLinkSubmissionNotSupported,
    supabase: throwLinkSubmissionNotSupported,
});
export const getUpdateStatus = selectByMode({
    backend: backend.getUpdateStatus,
    firebase: throwUpdateCheckNotSupported,
    supabase: throwUpdateCheckNotSupported,
});
export const getFeedback = selectByMode({
    backend: backend.getFeedback,
    firebase: firebase.getFeedback,
    supabase: supabase.getFeedback,
});

export const completeCodelab = selectByMode({
    backend: backend.completeCodelab,
    firebase: noOpAsync,
    supabase: supabase.completeCodelab,
});

export const getCertificate = selectByMode({
    backend: backend.getCertificate,
    firebase: throwNotSupportedInFirebaseMode,
    supabase: supabase.getCertificate,
});

export const getMaterials = selectByMode({
    backend: backend.getMaterials,
    firebase: returnEmptyList,
    supabase: supabase.getMaterials,
});
export const addMaterial = selectByMode({
    backend: backend.addMaterial,
    firebase: throwNotSupportedInFirebaseMode,
    supabase: supabase.addMaterial,
});
export const deleteMaterial = selectByMode({
    backend: backend.deleteMaterial,
    firebase: throwNotSupportedInFirebaseMode,
    supabase: supabase.deleteMaterial,
});
export const uploadMaterial = selectByMode({
    backend: backend.uploadMaterial,
    firebase: throwNotSupportedInFirebaseMode,
    supabase: supabase.uploadMaterial,
});

export const submitFile = selectByMode({
    backend: backend.submitFile,
    firebase: throwNotSupportedInFirebaseMode,
    supabase: supabase.submitFile,
});
export const getSubmissions = selectByMode({
    backend: backend.getSubmissions,
    firebase: returnEmptyList,
    supabase: supabase.getSubmissions,
});
export const deleteSubmission = selectByMode({
    backend: backend.deleteSubmission,
    firebase: throwNotSupportedInFirebaseMode,
    supabase: supabase.deleteSubmission,
});

export const getQuizzes = selectByMode({
    backend: backend.getQuizzes,
    firebase: returnEmptyList,
    supabase: supabase.getQuizzes,
});
export const updateQuizzes = selectByMode({
    backend: backend.updateQuizzes,
    firebase: throwNotSupportedInFirebaseMode,
    supabase: supabase.updateQuizzes,
});
export const submitQuiz = selectByMode({
    backend: backend.submitQuiz,
    firebase: noOpAsync,
    supabase: supabase.submitQuiz,
});
export const getQuizSubmissions = selectByMode({
    backend: backend.getQuizSubmissions,
    firebase: returnEmptyList,
    supabase: supabase.getQuizSubmissions,
});

export const getWsUrl = backend.getWsUrl;
export const listenToWsReplacement = selectByMode({
    backend: firebase.listenToWsReplacement,
    firebase: firebase.listenToWsReplacement,
    supabase: supabase.listenToWsReplacement,
});

export const isFirebaseMode = () => MODE === "firebase";
export const isSupabaseMode = () => MODE === "supabase";
export const isServerlessMode = () => USE_SERVERLESS;

export const exportCodelab = selectByMode({
    backend: backend.exportCodelab,
    firebase: alertExportCodelabNotSupported,
    supabase: alertExportCodelabNotSupported,
});
export const importCodelab = selectByMode({
    backend: backend.importCodelab,
    firebase: throwImportCodelabNotSupported,
    supabase: throwImportCodelabNotSupported,
});
export const exportBackup = selectByMode({
    backend: backend.exportBackup,
    firebase: alertExportBackupNotSupported,
    supabase: alertExportBackupNotSupported,
});
export const restoreBackup = selectByMode({
    backend: backend.restoreBackup,
    firebase: throwRestoreBackupNotSupported,
    supabase: throwRestoreBackupNotSupported,
});
export const inspectBackup = selectByMode({
    backend: backend.inspectBackup,
    firebase: throwInspectBackupNotSupported,
    supabase: throwInspectBackupNotSupported,
});

export const createCodeServer = selectByMode({
    backend: backend.createCodeServer,
    firebase: throwNotSupportedInServerlessMode,
    supabase: throwNotSupportedInServerlessMode,
});
export const getCodeServerInfo = selectByMode({
    backend: backend.getCodeServerInfo,
    firebase: throwNotSupportedInServerlessMode,
    supabase: throwNotSupportedInServerlessMode,
});
export const createCodeServerBranch = selectByMode({
    backend: backend.createCodeServerBranch,
    firebase: throwNotSupportedInServerlessMode,
    supabase: throwNotSupportedInServerlessMode,
});
export const createCodeServerFolder = selectByMode({
    backend: backend.createCodeServerFolder,
    firebase: throwNotSupportedInServerlessMode,
    supabase: throwNotSupportedInServerlessMode,
});
export const downloadCodeServerWorkspace = selectByMode({
    backend: backend.downloadCodeServerWorkspace,
    firebase: throwNotSupportedInServerlessMode,
    supabase: throwNotSupportedInServerlessMode,
});
export const deleteCodeServer = selectByMode({
    backend: backend.deleteCodeServer,
    firebase: throwNotSupportedInServerlessMode,
    supabase: throwNotSupportedInServerlessMode,
});
export type { CodeServerInfo, WorkspaceFile } from "./api-backend";

export const saveAiConversation = selectByMode({
    backend: backend.saveAiConversation,
    firebase: noOpSaveAiConversation,
    supabase: noOpSaveAiConversation,
});
export const getAiConversations = selectByMode({
    backend: backend.getAiConversations,
    firebase: returnEmptyList,
    supabase: returnEmptyList,
});
