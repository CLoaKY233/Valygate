import "server-only";

import { redirect } from "next/navigation";

import { getApiBaseUrl } from "./config";
import { clearSessionToken, getSessionToken, setSessionToken } from "./session";
import type {
  ApiErrorPayload,
  AuthResponse,
  CreateVirtualKeyResponse,
  Model,
  Provider,
  ProviderInput,
  ProviderUpdateInput,
  User,
  VirtualKey,
  VirtualKeyInput,
  VirtualKeyUpdateInput,
} from "./types";

export class ApiError extends Error {
  status: number;

  constructor(status: number, message: string) {
    super(message);
    this.status = status;
  }
}

async function parseError(response: Response) {
  const payload = (await response.json().catch(() => null)) as ApiErrorPayload | null;
  return payload?.error?.message ?? response.statusText ?? "Request failed";
}

async function request<T>(path: string, init: RequestInit = {}, token?: string | null) {
  const headers = new Headers(init.headers);
  headers.set("Accept", "application/json");

  if (init.body && !headers.has("Content-Type")) {
    headers.set("Content-Type", "application/json");
  }

  if (token) {
    headers.set("Authorization", `Bearer ${token}`);
  }

  const response = await fetch(`${getApiBaseUrl()}${path}`, {
    ...init,
    headers,
    cache: "no-store",
  });

  if (!response.ok) {
    throw new ApiError(response.status, await parseError(response));
  }

  if (response.status === 202 || response.status === 204) {
    return null as T;
  }

  const contentType = response.headers.get("content-type") ?? "";
  if (!contentType.includes("application/json")) {
    throw new ApiError(
      response.status,
      `API returned non-JSON response (${contentType || "no content-type"}). ` +
        `Check that VALYMUX_API_BASE_URL points to the Rust API, not the Next.js server.`,
    );
  }

  return (await response.json()) as T;
}

async function authenticatedRequest<T>(path: string, init: RequestInit = {}) {
  const token = await getSessionToken();
  if (!token) {
    redirect("/signin");
  }

  try {
    return await request<T>(path, init, token);
  } catch (error) {
    if (error instanceof ApiError && error.status === 401) {
      // Clear the stale cookie via a Route Handler (cookies() can't be modified
      // during rendering in a Server Component — only in Actions/Route Handlers).
      redirect("/api/auth/clear");
    }

    throw error;
  }
}

export async function signIn(email: string, password: string) {
  const payload = await request<AuthResponse>("/auth/signin", {
    method: "POST",
    body: JSON.stringify({ email, password }),
  });

  await setSessionToken(payload.token);
  return payload;
}

export async function signUp(name: string, email: string, password: string) {
  const payload = await request<AuthResponse>("/auth/signup", {
    method: "POST",
    body: JSON.stringify({ name, email, password }),
  });

  await setSessionToken(payload.token);
  return payload;
}

export async function signOut() {
  await clearSessionToken();
}

export async function getCurrentUser() {
  return authenticatedRequest<User>("/me");
}

export async function listProviders() {
  return authenticatedRequest<Provider[]>("/providers");
}

export async function getProvider(providerId: string) {
  return authenticatedRequest<Provider>(`/providers/${providerId}`);
}

export async function createProvider(input: ProviderInput) {
  return authenticatedRequest<Provider>("/providers", {
    method: "POST",
    body: JSON.stringify(input),
  });
}

export async function updateProvider(providerId: string, input: ProviderUpdateInput) {
  return authenticatedRequest<Provider>(`/providers/${providerId}`, {
    method: "PATCH",
    body: JSON.stringify(input),
  });
}

export async function deleteProvider(providerId: string) {
  return authenticatedRequest<Provider>(`/providers/${providerId}`, {
    method: "DELETE",
  });
}

export async function syncProvider(providerId: string) {
  return authenticatedRequest<null>(`/providers/${providerId}/sync`, {
    method: "POST",
  });
}

export async function listProviderModels(providerId: string) {
  return authenticatedRequest<Model[]>(`/providers/${providerId}/models`);
}

export async function listModels() {
  return authenticatedRequest<Model[]>("/models");
}

export async function getModel(alias: string) {
  return authenticatedRequest<Model>(`/models/${alias}`);
}

export async function listVirtualKeys() {
  return authenticatedRequest<VirtualKey[]>("/virtual-keys");
}

export async function getVirtualKey(keyId: string) {
  return authenticatedRequest<VirtualKey>(`/virtual-keys/${keyId}`);
}

export async function createVirtualKey(input: VirtualKeyInput) {
  return authenticatedRequest<CreateVirtualKeyResponse>("/virtual-keys", {
    method: "POST",
    body: JSON.stringify(input),
  });
}

export async function updateVirtualKey(keyId: string, input: VirtualKeyUpdateInput) {
  return authenticatedRequest<VirtualKey>(`/virtual-keys/${keyId}`, {
    method: "PATCH",
    body: JSON.stringify(input),
  });
}

export async function deleteVirtualKey(keyId: string) {
  return authenticatedRequest<VirtualKey>(`/virtual-keys/${keyId}`, {
    method: "DELETE",
  });
}
