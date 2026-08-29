import { useState } from "react";
import { STATION_ACTIONS, type StationTarget } from "../lib/actions";
import { useCommandStore } from "../context/command";
import { getErrorMessage } from "../lib/errors";

type StationActionProps = {
  stations: StationTarget[];
  onClose: () => void;
};

export function StationActinoModal({ stations, onClose }: StationActionProps) {
  const { register } = useCommandStore();
  const [selectedActionKey, setSelectedActionKey] = useState("");
  const [fieldValues, setFieldValues] = useState<Record<string, string>>({});
  const [fieldErrors, setFieldErrors] = useState<Set<string>>(new Set());
  const [isPending, setIsPending] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);

  const isSingle = stations.length === 1;
  const availableActions = STATION_ACTIONS.filter(
    (a) => a.target === "both" || a.target === (isSingle ? "single" : "multiple"),
  );

  const selectedAction = availableActions.find(
    (a) => a.key === selectedActionKey,
  );

  const handleActionChange = (key: string) => {
    setSelectedActionKey(key);
    setFieldValues({});
    setFieldErrors(new Set());
  };

  const handleExecute = async () => {
    if (!selectedAction) return;

    const missing = new Set(
      selectedAction.fields
        .filter((f) => f.required && !fieldValues[f.key]?.trim())
        .map((f) => f.key),
    );
    if (missing.size > 0) {
      setFieldErrors(missing);
      return;
    }

    setIsPending(true);
    setActionError(null);
    try {
      await selectedAction.execute(stations, fieldValues, register);
      onClose();
    } catch (err) {
      setActionError(getErrorMessage(err));
    } finally {
      setIsPending(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
      <div className="bg-surface-800 rounded-xl border border-surface-600 p-6 w-full max-w-md shadow-2xl">
        <h2 className="text-xl font-semibold text-white mb-1">Actions</h2>
        <p className="text-sm text-gray-400 mb-6">
          Target: {stations.length} station(s)
        </p>

        <div className="mb-4">
          <label className="block text-sm text-gray-400 mb-2">Action</label>
          <select
            value={selectedActionKey}
            onChange={(e) => handleActionChange(e.target.value)}
            className="w-full bg-surface-700 border border-surface-500 rounded-lg px-4 py-2.5 text-gray-200 focus:outline-none focus:ring-2 focus:ring-primary-500"
          >
            <option value="">Choose...</option>
            {availableActions.map((action) => (
              <option key={action.key} value={action.key}>
                {action.name}
              </option>
            ))}
          </select>
        </div>

        {selectedAction && (
          <>
            <p className="text-sm text-gray-400 mb-4">
              {selectedAction.description}
            </p>
            {selectedAction.fields.map((field) => {
              const hasError = fieldErrors.has(field.key);
              return (
                <div key={field.key} className="mb-4">
                  <label className="block text-sm text-gray-400 mb-2">
                    {field.label}
                  </label>
                  <input
                    type={field.type}
                    placeholder={field.placeholder}
                    value={fieldValues[field.key] ?? ""}
                    onChange={(e) => {
                      setFieldValues((prev) => ({
                        ...prev,
                        [field.key]: e.target.value,
                      }));
                      if (hasError) {
                        setFieldErrors((prev) => {
                          const next = new Set(prev);
                          next.delete(field.key);
                          return next;
                        });
                      }
                    }}
                    className={`w-full bg-surface-700 rounded-lg px-4 py-2.5 text-gray-200 focus:outline-none focus:ring-2 border ${
                      hasError
                        ? "border-danger-500 focus:ring-danger-500"
                        : "border-surface-500 focus:ring-primary-500"
                    }`}
                  />
                  {hasError && (
                    <p className="text-sm text-danger-500 mt-1">
                      {field.label} is required
                    </p>
                  )}
                </div>
              );
            })}
          </>
        )}

        {actionError && (
          <p className="text-sm text-danger-500 mt-4">{actionError}</p>
        )}

        <div className="flex justify-end gap-3 mt-6">
          <button
            onClick={onClose}
            className="px-4 py-2 text-gray-400 hover:text-white transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={handleExecute}
            disabled={!selectedAction || isPending}
            className={`px-4 py-2 disabled:bg-surface-600 disabled:text-gray-500 text-white rounded-lg transition-colors ${
              selectedAction?.type === "danger"
                ? "bg-danger-500 hover:bg-danger-600"
                : "bg-primary-500 hover:bg-primary-600"
            }`}
          >
            {isPending ? "Executing..." : "Execute"}
          </button>
        </div>
      </div>
    </div>
  );
}
