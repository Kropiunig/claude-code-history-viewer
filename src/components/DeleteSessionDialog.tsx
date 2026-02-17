import React from "react";
import { useTranslation } from "react-i18next";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { AlertTriangle, Trash2 } from "lucide-react";
import { useSessionDelete } from "@/hooks/useSessionDelete";

interface DeleteSessionDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  filePath: string;
  sessionName: string;
  onSuccess?: () => void;
}

export const DeleteSessionDialog: React.FC<DeleteSessionDialogProps> = ({
  open,
  onOpenChange,
  filePath,
  sessionName,
  onSuccess,
}) => {
  const { t } = useTranslation();
  const { deleteSession, isDeleting, error } = useSessionDelete();

  const handleDelete = async () => {
    try {
      await deleteSession(filePath);
      onSuccess?.();
      onOpenChange(false);
    } catch {
      // Error is handled by the hook and displayed in the dialog
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Trash2 className="w-5 h-5 text-destructive" />
            {t("session.delete.title", "Delete session?")}
          </DialogTitle>
          <DialogDescription>
            {t("session.delete.description", "This action cannot be undone.")}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-3 py-2">
          <div className="space-y-1">
            <p className="text-sm text-muted-foreground">
              {t("session.delete.sessionLabel", "Session")}
            </p>
            <p className="text-sm bg-muted/50 rounded-md px-3 py-2 break-words line-clamp-3">
              {sessionName || t("session.summaryNotFound", "No summary")}
            </p>
          </div>

          <Alert variant="destructive">
            <AlertTriangle className="h-4 w-4" />
            <AlertDescription>
              {t(
                "session.delete.warning",
                "The session file and all associated data will be permanently deleted."
              )}
            </AlertDescription>
          </Alert>

          {error && (
            <Alert variant="destructive">
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}
        </div>

        <DialogFooter>
          <Button
            type="button"
            variant="outline"
            onClick={() => onOpenChange(false)}
            disabled={isDeleting}
          >
            {t("common.cancel")}
          </Button>
          <Button
            type="button"
            variant="destructive"
            onClick={handleDelete}
            disabled={isDeleting}
          >
            {isDeleting
              ? t("session.delete.deleting", "Deleting...")
              : t("session.delete.confirm", "Delete")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
