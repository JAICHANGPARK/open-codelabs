import { initializeApp, getApps, type FirebaseApp } from "firebase/app";
import { getFirestore, type Firestore } from "firebase/firestore";
import { getAuth, type Auth } from "firebase/auth";
import { getStorage, type FirebaseStorage } from "firebase/storage";
import { getDatabase, type Database } from "firebase/database";

export type FirebaseConfig = {
    apiKey?: string;
    authDomain?: string;
    projectId?: string;
    storageBucket?: string;
    messagingSenderId?: string;
    appId?: string;
    databaseURL?: string;
};

export type FirebaseServices = {
    app?: FirebaseApp;
    db?: Firestore;
    auth?: Auth;
    storage?: FirebaseStorage;
    rtdb?: Database;
};

type FirebaseDeps = {
    initializeApp: typeof initializeApp;
    getApps: typeof getApps;
    getFirestore: typeof getFirestore;
    getAuth: typeof getAuth;
    getStorage: typeof getStorage;
    getDatabase: typeof getDatabase;
};

const firebaseConfig: FirebaseConfig = {
    apiKey: import.meta.env.VITE_FIREBASE_API_KEY,
    authDomain: import.meta.env.VITE_FIREBASE_AUTH_DOMAIN,
    projectId: import.meta.env.VITE_FIREBASE_PROJECT_ID,
    storageBucket: import.meta.env.VITE_FIREBASE_STORAGE_BUCKET,
    messagingSenderId: import.meta.env.VITE_FIREBASE_MESSAGING_SENDER_ID,
    appId: import.meta.env.VITE_FIREBASE_APP_ID,
    databaseURL: import.meta.env.VITE_FIREBASE_DATABASE_URL,
};

const defaultDeps: FirebaseDeps = {
    initializeApp,
    getApps,
    getFirestore,
    getAuth,
    getStorage,
    getDatabase,
};

export function isFirebaseConfigValid(apiKey: string | undefined): boolean {
    return !!apiKey && apiKey !== "undefined" && apiKey !== "";
}

export function createFirebaseServices(config: FirebaseConfig = firebaseConfig, deps: FirebaseDeps = defaultDeps): FirebaseServices {
    if (!isFirebaseConfigValid(config.apiKey)) return {};

    let app: FirebaseApp | undefined;
    let db: Firestore | undefined;
    let auth: Auth | undefined;
    let storage: FirebaseStorage | undefined;
    let rtdb: Database | undefined;

    try {
        const apps = deps.getApps();
        app = apps.length ? apps[0] : deps.initializeApp(config);
        db = deps.getFirestore(app);
        auth = deps.getAuth(app);
        storage = deps.getStorage(app);
        rtdb = deps.getDatabase(app);
    } catch (e) {
        console.error("Firebase initialization failed:", e);
    }

    return { app, db, auth, storage, rtdb };
}

const services = createFirebaseServices();

// Preserve loose runtime exports for existing Firebase API wrappers.
const app: FirebaseApp | any = services.app;
const db: Firestore | any = services.db;
const auth: Auth | any = services.auth;
const storage: FirebaseStorage | any = services.storage;
const rtdb: Database | any = services.rtdb;

export { app, db, auth, storage, rtdb };
export default app;
