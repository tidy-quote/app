import { useState, useCallback, type ReactNode } from "react";
import type { AuthState } from "../../domain/types";
import * as authService from "../../application/auth";
import { AuthContext } from "./AuthContext";

interface AuthProviderProps {
  children: ReactNode;
}

export function AuthProvider({ children }: AuthProviderProps): React.JSX.Element {
  const [authState, setAuthState] = useState<AuthState>(() => authService.getAuthState());

  const login = useCallback(async (email: string, password: string): Promise<void> => {
    const state = await authService.login(email, password);
    setAuthState(state);
  }, []);

  const signup = useCallback(async (email: string, password: string): Promise<void> => {
    const state = await authService.signup(email, password);
    setAuthState(state);
  }, []);

  const logout = useCallback((): void => {
    authService.logout();
    setAuthState({ user: null, token: null });
  }, []);

  return (
    <AuthContext.Provider
      value={{
        user: authState.user,
        token: authState.token,
        isAuthenticated: authState.user !== null && authState.token !== null,
        login,
        signup,
        logout,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}
