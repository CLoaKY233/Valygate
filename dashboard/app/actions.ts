"use server";

import { revalidatePath } from "next/cache";
import { redirect } from "next/navigation";

import {
  ApiError,
  createProvider,
  createVirtualKey,
  deleteProvider,
  deleteVirtualKey,
  signIn,
  signOut,
  signUp,
  syncProvider,
  updateProvider,
  updateVirtualKey,
} from "@/lib/api";
import type { FieldState, ProviderUpdateInput, VirtualKeyUpdateInput } from "@/lib/types";

function parseList(value: FormDataEntryValue | null) {
  return String(value ?? "")
    .split(/[\n,]/)
    .map((item) => item.trim())
    .filter(Boolean);
}

function parseMultiSelectList(values: FormDataEntryValue[]) {
  return values.map((v) => String(v).trim()).filter(Boolean);
}

function parseRoutes(formData: FormData) {
  const aliases = formData.getAll("routeModelAlias").map((value) => String(value).trim());
  const providerIds = formData
    .getAll("routeProviderCredentialId")
    .map((value) => String(value).trim());

  const seen = new Set<string>();
  return aliases
    .map((modelAlias, index) => ({
      model_alias: modelAlias,
      provider_credential_id: providerIds[index] ?? "",
    }))
    .filter((route) => {
      if (!route.model_alias || !route.provider_credential_id) return false;
      if (seen.has(route.model_alias)) return false;
      seen.add(route.model_alias);
      return true;
    });
}

function toIsoDateTime(value: string) {
  if (!value) {
    return null;
  }

  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) {
    return null;
  }

  return parsed.toISOString();
}

function handleError(error: unknown): FieldState {
  if (error instanceof ApiError) {
    return { error: error.message };
  }

  if (error instanceof Error) {
    return { error: error.message };
  }

  return { error: "Unexpected error." };
}

export async function signInAction(_: FieldState, formData: FormData): Promise<FieldState> {
  try {
    await signIn(String(formData.get("email") ?? ""), String(formData.get("password") ?? ""));
  } catch (error) {
    return handleError(error);
  }

  redirect("/");
}

export async function signUpAction(_: FieldState, formData: FormData): Promise<FieldState> {
  try {
    await signUp(
      String(formData.get("name") ?? ""),
      String(formData.get("email") ?? ""),
      String(formData.get("password") ?? ""),
    );
  } catch (error) {
    return handleError(error);
  }

  redirect("/");
}

export async function signOutAction() {
  await signOut();
  redirect("/signin");
}

export async function createProviderAction(
  _: FieldState,
  formData: FormData,
): Promise<FieldState> {
  try {
    await createProvider({
      provider: "google-genai",
      label: String(formData.get("label") ?? "").trim(),
      api_key: String(formData.get("apiKey") ?? "").trim(),
      tags: parseList(formData.get("tags")),
    });
  } catch (error) {
    return handleError(error);
  }

  revalidatePath("/providers");
  revalidatePath("/");
  revalidatePath("/models");
  return { success: "Provider credential created. Model sync has started in the background." };
}

export async function updateProviderAction(providerId: string, formData: FormData) {
  const input: ProviderUpdateInput = {
    label: String(formData.get("label") ?? "").trim(),
    tags: parseList(formData.get("tags")),
    enabled: formData.get("enabled") === "on",
  };

  const apiKey = String(formData.get("apiKey") ?? "").trim();
  if (apiKey) {
    input.api_key = apiKey;
  }

  await updateProvider(providerId, input);
  revalidatePath("/providers");
  revalidatePath(`/providers/${providerId}`);
  revalidatePath("/");
}

export async function deleteProviderAction(providerId: string) {
  await deleteProvider(providerId);
  revalidatePath("/providers");
  revalidatePath("/");
  redirect("/providers");
}

export async function syncProviderAction(providerId: string) {
  await syncProvider(providerId);
  revalidatePath("/providers");
  revalidatePath(`/providers/${providerId}`);
  revalidatePath("/");
}

export async function createVirtualKeyAction(
  _: FieldState,
  formData: FormData,
): Promise<FieldState> {
  try {
    const created = await createVirtualKey({
      name: String(formData.get("name") ?? "").trim(),
      // Create with no model scoping — user configures this after sync on the detail page
      allowed_models: [],
      model_routes: [],
      tags: parseList(formData.get("tags")),
      expires_at: toIsoDateTime(String(formData.get("expiresAt") ?? "")),
    });

    revalidatePath("/virtual-keys");
    revalidatePath("/");

    return {
      success: "Virtual key created. Copy your key, then configure model scopes below.",
      rawKey: created.raw_key,
      createdKeyId: created.key.id,
    };
  } catch (error) {
    return handleError(error);
  }
}

export async function updateVirtualKeyAction(keyId: string, formData: FormData) {
  const model_routes = parseRoutes(formData);
  // allowed_models must match route aliases exactly — derive it from routes
  const allowed_models = model_routes.map((r) => r.model_alias);

  const input: VirtualKeyUpdateInput = {
    name: String(formData.get("name") ?? "").trim(),
    allowed_models,
    model_routes,
    tags: parseList(formData.get("tags")),
    expires_at: toIsoDateTime(String(formData.get("expiresAt") ?? "")),
    enabled: formData.get("enabled") === "on",
  };

  await updateVirtualKey(keyId, input);
  revalidatePath("/virtual-keys");
  revalidatePath(`/virtual-keys/${keyId}`);
  revalidatePath("/");
}

export async function updateVirtualKeyFormAction(
  keyId: string,
  _: FieldState,
  formData: FormData,
): Promise<FieldState> {
  // Validate for duplicate model aliases before hitting the API
  const aliases = formData
    .getAll("routeModelAlias")
    .map((v) => String(v).trim())
    .filter(Boolean);
  const uniqueAliases = new Set(aliases);
  if (uniqueAliases.size < aliases.length) {
    return { error: "Each model alias can only appear once in the route list. Remove duplicate routes before saving." };
  }

  try {
    await updateVirtualKeyAction(keyId, formData);
  } catch (error) {
    if (error instanceof ApiError && error.status === 500) {
      return { error: "The backend rejected this update. Check that each model alias is unique and that all routes reference valid provider credentials." };
    }
    return handleError(error);
  }

  return { success: "Virtual key updated." };
}

export async function deleteVirtualKeyAction(keyId: string) {
  await deleteVirtualKey(keyId);
  revalidatePath("/virtual-keys");
  revalidatePath("/");
  redirect("/virtual-keys");
}
