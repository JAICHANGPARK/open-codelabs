export async function returnEmptyList<T = never>(..._args: unknown[]): Promise<T[]> {
    return [];
}

export async function noOpAsync(..._args: unknown[]): Promise<void> {}

export function backendNoopUnsubscribe(_cb: unknown): () => void {
    return () => {};
}

export async function throwNotSupportedInServerlessMode(..._args: unknown[]): Promise<never> {
    throw new Error("Not supported in serverless mode");
}

export async function throwNotSupportedInBackendMode(..._args: unknown[]): Promise<never> {
    throw new Error("Not supported in backend mode");
}

export async function throwNotSupportedInFirebaseMode(..._args: unknown[]): Promise<never> {
    throw new Error("Not supported in Firebase mode");
}

export async function throwLinkSubmissionNotSupported(..._args: unknown[]): Promise<never> {
    throw new Error("Link submission not supported");
}

export async function throwUpdateCheckNotSupported(..._args: unknown[]): Promise<never> {
    throw new Error("Update check not supported");
}

export async function alertExportCodelabNotSupported(..._args: unknown[]): Promise<void> {
    alert("Export is not supported in serverless mode yet.");
}

export async function throwImportCodelabNotSupported(..._args: unknown[]): Promise<never> {
    throw new Error("Import is not supported in serverless mode yet.");
}

export async function alertExportBackupNotSupported(..._args: unknown[]): Promise<void> {
    alert("Backup export is not supported in serverless mode yet.");
}

export async function throwRestoreBackupNotSupported(..._args: unknown[]): Promise<never> {
    throw new Error("Backup restore is not supported in serverless mode yet.");
}

export async function throwInspectBackupNotSupported(..._args: unknown[]): Promise<never> {
    throw new Error("Backup inspect is not supported in serverless mode yet.");
}
