import { createClient, type SupabaseClient } from "@supabase/supabase-js";

const supabaseUrl = import.meta.env.VITE_SUPABASE_URL;
const supabaseAnonKey = import.meta.env.VITE_SUPABASE_ANON_KEY;

const isConfigValid =
    !!supabaseUrl &&
    supabaseUrl !== "undefined" &&
    !!supabaseAnonKey &&
    supabaseAnonKey !== "undefined";

let supabase: SupabaseClient | undefined;

if (isConfigValid) {
    supabase = createClient(supabaseUrl, supabaseAnonKey, {
        auth: {
            persistSession: true,
            autoRefreshToken: true,
            detectSessionInUrl: true,
        },
    });
}

export { supabase };
