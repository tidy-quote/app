import { createContext, useContext, useState, useCallback, type ReactNode } from "react";
import type { AuthState, AuthUser } from "../../domain/types";
import * as authService from "../../application/auth";

interface AuthContextValue {
  user: AuthUser | null;
  token: string | null;
  isAuthenticated: boolean;
  login: (email: string, password: string) => Promise<void>;
  signup: (email: string, password: string) => Promise<void>;
  logout: () => void;
}

const AuthContext = createContext<AuthContextValue | null>(null);

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

  const value: AuthContextValue = {
    user: authState.user,
    token: authState.token,
    isAuthenticated: authState.user !== null && authState.token !== null,
    login,
    signup,
    logout,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth(): AuthContextValue {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error("useAuth must be used within an AuthProvider");
  }
  return context;
}
