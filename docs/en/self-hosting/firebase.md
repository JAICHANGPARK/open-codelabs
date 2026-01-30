# Open Codelabs Firebase Deployment Guide

This document explains how to deploy Open Codelabs with Firebase (Hosting + Firestore + Realtime Database) for users who do not want to run the Rust/SQLite server.

## 1. Prepare a Firebase project

1. Create a new project in the [Firebase Console](https://console.firebase.google.com/).
2. Enable **Firestore Database** (production mode or test mode).
3. Enable **Realtime Database** (for chat and progress sync).
4. Enable **Firebase Storage** (for image uploads).
5. Enable **Google sign-in** in **Firebase Authentication**.
6. In **Project Settings**, add a Web App and note the Firebase SDK config.

## 2. Configure the frontend

Add the following to `frontend/.env`.

```bash
# Enable Firebase mode
VITE_USE_FIREBASE=true

# Admin credentials (Firebase mode only, fallback)
VITE_ADMIN_ID=admin
VITE_ADMIN_PW=admin123

# Firebase SDK config
VITE_FIREBASE_API_KEY=your_api_key
VITE_FIREBASE_AUTH_DOMAIN=your_project.firebaseapp.com
VITE_FIREBASE_PROJECT_ID=your_project_id
VITE_FIREBASE_STORAGE_BUCKET=your_project.appspot.com
VITE_FIREBASE_MESSAGING_SENDER_ID=your_sender_id
VITE_FIREBASE_APP_ID=your_app_id
VITE_FIREBASE_DATABASE_URL=https://your_project.firebaseio.com
```

## 3. Switch the SvelteKit adapter

To deploy as a static site on Firebase Hosting, use `adapter-static`.

1. Install the adapter in the `frontend` directory:
   ```bash
   cd frontend
   bun add -D @sveltejs/adapter-static
   ```
2. Update `svelte.config.js`:
   ```javascript
   import adapter from '@sveltejs/adapter-static';
   // ...
   ```
3. Configure CSR and prerendering in `src/routes/+layout.ts` if needed (it may already be set).

## 4. Security rules and indexes

These files in the project root are applied during deployment:
- `firestore.rules`: Firestore access rules
- `database.rules.json`: Realtime Database rules (chat, etc.)
- `storage.rules`: Storage upload rules
- `firestore.indexes.json`: indexes for efficient queries

## 5. Deploy

1. Install and log in to the Firebase CLI:
   ```bash
   npm install -g firebase-tools
   firebase login
   ```
2. Build the project:
   ```bash
   cd frontend
   bun run build
   ```
3. Deploy to Firebase:
   ```bash
   cd ..
   firebase deploy
   ```

## Key features and notes

- **Google sign-in**: Firebase Auth supports Google login, and signed-in users can manage or review their codelabs.
- **Realtime features**: Realtime Database is used for chat, help requests, and progress sharing.
- **Data storage**: codelab metadata and steps are stored in Firestore.
- **Limitations**: In Firebase mode, ZIP import/export is not supported.

## Related links

- [Firebase Console](https://console.firebase.google.com/)
- [Firebase Hosting docs](https://firebase.google.com/docs/hosting)
