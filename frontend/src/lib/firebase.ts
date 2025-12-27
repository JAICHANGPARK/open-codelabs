import { initializeApp, getApps, type FirebaseApp } from "firebase/app";
import { getFirestore, type Firestore } from "firebase/firestore";
import { getAuth, type Auth } from "firebase/auth";
import { getStorage, type FirebaseStorage } from "firebase/storage";
import { getDatabase, type Database } from "firebase/database";

const firebaseConfig = {
    apiKey: import.meta.env.VITE_FIREBASE_API_KEY,
    authDomain: import.meta.env.VITE_FIREBASE_AUTH_DOMAIN,
    projectId: import.meta.env.VITE_FIREBASE_PROJECT_ID,
    storageBucket: import.meta.env.VITE_FIREBASE_STORAGE_BUCKET,
    messagingSenderId: import.meta.env.VITE_FIREBASE_MESSAGING_SENDER_ID,
    appId: import.meta.env.VITE_FIREBASE_APP_ID,
    databaseURL: import.meta.env.VITE_FIREBASE_DATABASE_URL,
};

// Check if Firebase config is actually provided
const isConfigValid = !!import.meta.env.VITE_FIREBASE_API_KEY && 
                     import.meta.env.VITE_FIREBASE_API_KEY !== "undefined" &&
                     import.meta.env.VITE_FIREBASE_API_KEY !== "";

let app: FirebaseApp | undefined;
let db: Firestore | any;
let auth: Auth | any;
let storage: FirebaseStorage | any;
let rtdb: Database | any;

if (isConfigValid) {
    try {
        if (!getApps().length) {
            app = initializeApp(firebaseConfig);
        } else {
            app = getApps()[0];
        }
        db = getFirestore(app);
        auth = getAuth(app);
        storage = getStorage(app);
        rtdb = getDatabase(app);
    } catch (e) {
        console.error("Firebase initialization failed:", e);
    }
}

export { app, db, auth, storage, rtdb };
export default app;
