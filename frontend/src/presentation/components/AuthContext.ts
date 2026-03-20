import { createContext } from "react";
import type { AuthUser, SubscriptionStatus } from "../../domain/types";

export interface AuthContextValue {
  user: AuthUser | null;
  token: string | null;
  isAuthenticated: boolean;
  subscriptionStatus: SubscriptionStatus;
  login: (email: string, password: string) => Promise<void>;
  signup: (email: string, password: string) => Promise<void>;
  logout: () => void;
  refreshSubscription: () => Promise<void>;
}

export const AuthContext = createContext<AuthContextValue | null>(null);
