# Open Codelabs Supabase Mode

This guide shows how to run Open Codelabs without the Rust backend by using Supabase for data, auth, storage, and realtime.

## 1. Create a Supabase project

- Create a new project in the Supabase dashboard.
- Copy the **Project URL** and **Anon Key** from Settings > API.

## 2. Create a storage bucket

- Create a bucket (public) for uploads, e.g. `open-codelabs`.
- Use the bucket name in `VITE_SUPABASE_STORAGE_BUCKET`.

## 3. Create tables and realtime

Run the schema from:

```
docs/self-hosting/supabase-schema.sql
```

If `alter publication` fails because tables are already added, it is safe to ignore.

## 4. Enable Google OAuth (optional)

If you want Google login, enable the Google provider in Supabase Auth settings.

## 5. Frontend environment variables

Set these in `frontend/.env` (or root `.env` for docker):

```bash
VITE_USE_SUPABASE=true
VITE_SUPABASE_URL=https://your-project.supabase.co
VITE_SUPABASE_ANON_KEY=your_anon_key
VITE_SUPABASE_STORAGE_BUCKET=open-codelabs
VITE_ADMIN_ID=admin
VITE_ADMIN_PW=admin
```

## Notes

- Supabase mode runs without the Rust backend. Code server, import/export ZIP, and backend-only AI features remain unavailable.
- For production, review RLS policies before exposing the anon key.
