import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { isAbsolutePath } from "@/utils/pathUtils";

export interface DeleteSessionResult {
  success: boolean;
  file_path: string;
  companion_dir_deleted: boolean;
}

export interface UseSessionDeleteReturn {
  isDeleting: boolean;
  error: string | null;
  deleteSession: (filePath: string) => Promise<DeleteSessionResult>;
}

/**
 * Hook for deleting Claude Code sessions.
 *
 * Calls the Rust backend to permanently remove a session JSONL file
 * and its companion directory (tool results, subagents).
 *
 * @example
 * ```tsx
 * const { deleteSession, isDeleting, error } = useSessionDelete();
 *
 * const handleDelete = async () => {
 *   try {
 *     await deleteSession(session.file_path);
 *     toast.success("Session deleted");
 *   } catch (err) {
 *     toast.error(`Failed: ${err}`);
 *   }
 * };
 * ```
 */
export const useSessionDelete = (): UseSessionDeleteReturn => {
  const [isDeleting, setIsDeleting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const deleteSession = useCallback(
    async (filePath: string): Promise<DeleteSessionResult> => {
      if (!filePath || !isAbsolutePath(filePath)) {
        const errorMessage = "Invalid file path: must be an absolute path";
        setError(errorMessage);
        throw new Error(errorMessage);
      }

      setIsDeleting(true);
      setError(null);

      try {
        const result = await invoke<DeleteSessionResult>("delete_session", {
          filePath,
        });
        return result;
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        setError(errorMessage);
        throw new Error(errorMessage);
      } finally {
        setIsDeleting(false);
      }
    },
    []
  );

  return {
    isDeleting,
    error,
    deleteSession,
  };
};
