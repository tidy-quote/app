import type { AuthState, AuthUser } from "../domain/types";

const API_BASE: string | undefined = import.meta.env.VITE_API_BASE;
const TOKEN_KEY = "tidy-quote:auth-token";
const USER_KEY = "tidy-quote:auth-user";

interface AuthApiResponse {
  token: string;
  user: AuthUser;
}

function hasBackend(): boolean {
  return API_BASE !== undefined && API_BASE !== "";
}

async function authRequest(
  path: string,
  email: string,
  password: string
): Promise<AuthApiResponse> {
  const response = await fetch(`${API_BASE}${path}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ email, password }),
  });

  if (!response.ok) {
    const body = await response.json().catch(() => ({}));
    throw new Error(
      (body as { error?: string }).error ?? `Request failed: ${response.status}`
    );
  }

  return response.json() as Promise<AuthApiResponse>;
}

function storeAuth(token: string, user: AuthUser): void {
  localStorage.setItem(TOKEN_KEY, token);
  localStorage.setItem(USER_KEY, JSON.stringify(user));
}

export async function signup(
  email: string,
  password: string
): Promise<AuthState> {
  if (hasBackend()) {
    const result = await authRequest("/api/auth/signup", email, password);
    storeAuth(result.token, result.user);
    return { token: result.token, user: result.user };
  }

  const fakeToken = `mock-token-${Date.now()}`;
  const user: AuthUser = { id: `user-${Date.now()}`, email };
  storeAuth(fakeToken, user);
  return { token: fakeToken, user };
}

export async function login(
  email: string,
  password: string
): Promise<AuthState> {
  if (hasBackend()) {
    const result = await authRequest("/api/auth/login", email, password);
    storeAuth(result.token, result.user);
    return { token: result.token, user: result.user };
  }

  const storedUser = localStorage.getItem(USER_KEY);
  if (!storedUser) {
    throw new Error("No account found. Please sign up first.");
  }

  const user = JSON.parse(storedUser) as AuthUser;
  if (user.email !== email) {
    throw new Error("Invalid credentials");
  }

  const fakeToken = `mock-token-${Date.now()}`;
  localStorage.setItem(TOKEN_KEY, fakeToken);
  return { token: fakeToken, user };
}

export function logout(): void {
  localStorage.removeItem(TOKEN_KEY);
  localStorage.removeItem(USER_KEY);
}

export function getToken(): string | null {
  return localStorage.getItem(TOKEN_KEY);
}

export function getAuthState(): AuthState {
  const token = localStorage.getItem(TOKEN_KEY);
  const userJson = localStorage.getItem(USER_KEY);

  if (!token || !userJson) {
    return { user: null, token: null };
  }

  try {
    const user = JSON.parse(userJson) as AuthUser;
    return { user, token };
  } catch {
    return { user: null, token: null };
  }
}

export function isAuthenticated(): boolean {
  const { token, user } = getAuthState();
  return token !== null && user !== null;
}
