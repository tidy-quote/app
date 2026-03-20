import { useState, useCallback, useSyncExternalStore, type ReactNode } from "react";
import type { AuthState, SubscriptionStatus } from "../../domain/types";
import * as authService from "../../application/auth";
import { getSubscriptionStatus } from "../../application/api";
import { AuthContext } from "./AuthContext";

interface AuthProviderProps {
  children: ReactNode;
}

let subscriptionState: SubscriptionStatus = "unknown";
let subscriptionListeners: Array<() => void> = [];

function getSubscriptionSnapshot(): SubscriptionStatus {
  return subscriptionState;
}

function setSubscription(status: SubscriptionStatus): void {
  subscriptionState = status;
  for (const listener of subscriptionListeners) {
    listener();
  }
}

function subscribeSubscription(listener: () => void): () => void {
  subscriptionListeners = [...subscriptionListeners, listener];
  return () => {
    subscriptionListeners = subscriptionListeners.filter((l) => l !== listener);
  };
}

async function fetchAndSetSubscription(): Promise<void> {
  try {
    const sub = await getSubscriptionStatus();
    setSubscription(sub.status === "active" ? "active" : "inactive");
  } catch {
    setSubscription("unknown");
  }
}

// Fetch on initial load if there's a token
if (authService.getAuthState().token) {
  fetchAndSetSubscription();
}

export function AuthProvider({ children }: AuthProviderProps): React.JSX.Element {
  const [authState, setAuthState] = useState<AuthState>(() => authService.getAuthState());
  const subscriptionStatus = useSyncExternalStore(subscribeSubscription, getSubscriptionSnapshot);

  const refreshSubscription = useCallback(async (): Promise<void> => {
    await fetchAndSetSubscription();
  }, []);

  const login = useCallback(async (email: string, password: string): Promise<void> => {
    const state = await authService.login(email, password);
    setAuthState(state);
    fetchAndSetSubscription();
  }, []);

  const signup = useCallback(async (email: string, password: string): Promise<void> => {
    const state = await authService.signup(email, password);
    setAuthState(state);
    fetchAndSetSubscription();
  }, []);

  const logout = useCallback((): void => {
    authService.logout();
    setAuthState({ user: null, token: null });
    setSubscription("unknown");
  }, []);

  return (
    <AuthContext.Provider
      value={{
        user: authState.user,
        token: authState.token,
        isAuthenticated: authState.user !== null && authState.token !== null,
        subscriptionStatus,
        login,
        signup,
        logout,
        refreshSubscription,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}
