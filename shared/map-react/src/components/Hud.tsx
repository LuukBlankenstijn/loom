import { useState } from "react";

type ButtonProps = {
  label: string;
  color: string;
  onClick: () => void;
};

function HudButton({ label, color, onClick }: ButtonProps) {
  return (
    <button
      type="button"
      onClick={onClick}
      style={{
        background: "rgba(255,255,255,0.05)",
        color,
        border: `1px solid ${color}66`,
        borderRadius: 4,
        padding: "8px 12px",
        width: "100%",
        textAlign: "center",
        fontSize: 14,
        cursor: "pointer",
      }}
    >
      {label}
    </button>
  );
}

export type HudProps = {
  mode: "view" | "edit";
  onToggleMode: () => void;
  onDelete: () => void;
  onClearSelection: () => void;
  onDuplicate: () => void;
  onRotate: () => void;
  onAddDoor: () => void;
  onAddSeat: () => void;
  onSave: () => void;
  saving?: boolean;
  error?: string | null;
  onClearError?: () => void;
};

export function Hud({
  mode,
  onToggleMode,
  onDelete,
  onClearSelection,
  onDuplicate,
  onRotate,
  onAddDoor,
  onAddSeat,
  onSave,
  saving,
  error,
  onClearError,
}: HudProps) {
  const [collapsed, setCollapsed] = useState(true);
  const editing = mode === "edit";

  const baseStyle: React.CSSProperties = {
    position: "absolute",
    top: 20,
    right: 20,
    background: "#262626",
    color: "white",
    border: "1px solid rgba(255,255,255,0.2)",
    borderRadius: 8,
    padding: 15,
    width: collapsed ? undefined : 220,
    fontFamily: "system-ui, sans-serif",
    pointerEvents: "auto",
  };

  const divider = (
    <div style={{ height: 1, background: "rgba(255,255,255,0.5)", margin: "8px 0" }} />
  );

  return (
    <>
      {error && (
        <div
          style={{
            position: "absolute",
            top: 0,
            left: 0,
            right: 0,
            background: "rgba(204,25,25,0.85)",
            color: "white",
            padding: 10,
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
            pointerEvents: "auto",
            fontFamily: "system-ui, sans-serif",
            fontSize: 14,
          }}
        >
          <span>Error: {error}</span>
          {onClearError && (
            <button
              type="button"
              onClick={onClearError}
              style={{
                background: "rgba(255,255,255,0.1)",
                color: "white",
                border: "none",
                padding: "2px 8px",
                cursor: "pointer",
              }}
            >
              Close
            </button>
          )}
        </div>
      )}
      <div style={baseStyle}>
        {collapsed ? (
          <button
            type="button"
            onClick={() => setCollapsed(false)}
            style={{
              background: "none",
              border: "none",
              color: "white",
              cursor: "pointer",
              fontSize: 16,
            }}
          >
            &lt;
          </button>
        ) : (
          <>
            <div style={{ display: "flex", alignItems: "center", marginBottom: 4 }}>
              <span style={{ fontSize: 16, flex: 1 }}>Menu</span>
              <button
                type="button"
                onClick={() => setCollapsed(true)}
                style={{
                  background: "none",
                  border: "none",
                  color: "white",
                  cursor: "pointer",
                  fontSize: 16,
                }}
              >
                v
              </button>
            </div>
            {divider}
            <HudButton
              label={editing ? "Mode: Editing" : "Mode: Viewing"}
              color={editing ? "#00ff00" : "#b3b3b3"}
              onClick={onToggleMode}
            />
            {editing && (
              <>
                {divider}
                <div style={{ display: "flex", flexDirection: "column", gap: 5 }}>
                  <HudButton label="Delete Selected" color="#ff4d4d" onClick={onDelete} />
                  <HudButton label="Clear Selection" color="#ffcc33" onClick={onClearSelection} />
                  <HudButton label="Duplicate Selection" color="#33ccff" onClick={onDuplicate} />
                  <HudButton label="Rotate Selection" color="#33ccff" onClick={onRotate} />
                </div>
                {divider}
                <div style={{ display: "flex", flexDirection: "column", gap: 5 }}>
                  <HudButton label="New Door" color="#00ff00" onClick={onAddDoor} />
                  <HudButton label="New Seat" color="#00ff00" onClick={onAddSeat} />
                </div>
                {divider}
                <HudButton
                  label={saving ? "Saving…" : "Save"}
                  color="#00ff00"
                  onClick={saving ? () => {} : onSave}
                />
              </>
            )}
          </>
        )}
      </div>
    </>
  );
}
