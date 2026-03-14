import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { derived, writable } from "svelte/store";

const GROK_STATE_EVENT = "app://grok-surface-state";

type GrokSurfacePhase = "started" | "finished" | "blocked" | "auth" | "recovering";
export type GrokSessionStatus = "loading" | "signed-in" | "signing-in" | "attention" | "error";

interface GrokSurfacePayload {
  phase: GrokSurfacePhase;
  url: string;
  message?: string;
  isAuthFlow: boolean;
}

interface GrokSessionState {
  status: GrokSessionStatus;
  label: string;
  message: string;
  url: string;
}

const defaultState: GrokSessionState = {
  status: "loading",
  label: "Loading",
  message: "Loading Grok inside the embedded surface.",
  url: "https://grok.com/",
};

const { subscribe, set } = writable<GrokSessionState>(defaultState);

let initialized = false;
let unlistenState: UnlistenFn | null = null;

export const grokSession = {
  subscribe,
};

export const grokSessionTone = derived(grokSession, ($grokSession) => $grokSession.status);

export async function initializeGrokSessionState(): Promise<void> {
  if (initialized) {
    return;
  }

  initialized = true;
  unlistenState = await listen<GrokSurfacePayload>(GROK_STATE_EVENT, (event) => {
    set(mapPayloadToSessionState(event.payload));
  });
}

export function disposeGrokSessionState(): void {
  unlistenState?.();
  unlistenState = null;
  initialized = false;
  set(defaultState);
}

function mapPayloadToSessionState(payload: GrokSurfacePayload): GrokSessionState {
  const message = payload.message ?? defaultState.message;
  const normalizedMessage = message.toLowerCase();

  if (payload.phase === "blocked" && (normalizedMessage.includes("rejected") || normalizedMessage.includes("malformed"))) {
    return {
      status: "error",
      label: "Error",
      message,
      url: payload.url,
    };
  }

  switch (payload.phase) {
    case "auth":
      return {
        status: "signing-in",
        label: "Signing In",
        message,
        url: payload.url,
      };
    case "blocked":
      return {
        status: "attention",
        label: "Attention",
        message,
        url: payload.url,
      };
    case "finished":
      return {
        status: "signed-in",
        label: "Signed In",
        message,
        url: payload.url,
      };
    case "started":
    case "recovering":
    default:
      return {
        status: "loading",
        label: "Loading",
        message,
        url: payload.url,
      };
  }
}
