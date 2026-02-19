import { createClient, type SupabaseClient } from "@supabase/supabase-js";

const supabaseUrl = import.meta.env.VITE_SUPABASE_URL;
const supabaseAnonKey = import.meta.env.VITE_SUPABASE_ANON_KEY;

type SupabaseDeps = {
    createClient: typeof createClient;
};

const defaultDeps: SupabaseDeps = {
    createClient,
};

const defaultAuthOptions = {
    persistSession: true,
    autoRefreshToken: true,
    detectSessionInUrl: true,
};

export function isSupabaseConfigValid(url: string | undefined, anonKey: string | undefined): boolean {
    return !!url && url !== "undefined" && !!anonKey && anonKey !== "undefined";
}

export function createSupabaseClient(
    url: string | undefined = supabaseUrl,
    anonKey: string | undefined = supabaseAnonKey,
    deps: SupabaseDeps = defaultDeps,
): SupabaseClient | undefined {
    if (!isSupabaseConfigValid(url, anonKey)) return undefined;

    return deps.createClient(url, anonKey, {
        auth: defaultAuthOptions,
    });
}

const supabase = createSupabaseClient();

export { supabase, defaultAuthOptions };
