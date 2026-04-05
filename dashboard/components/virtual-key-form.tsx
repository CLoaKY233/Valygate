"use client";

import { useActionState, useState } from "react";

import type { FieldState, Provider, VirtualKey, Model } from "@/lib/types";
import type { SelectOption } from "./multi-select";
import { CopyButton } from "./copy-button";

const initialState: FieldState = {};

type RouteRow = {
  modelAlias: string;
  providerCredentialId: string;
};

function buildInitialRoutes(key?: VirtualKey): RouteRow[] {
  if (!key || key.model_routes.length === 0) {
    return [{ modelAlias: "", providerCredentialId: "" }];
  }

  return key.model_routes.map((route) => ({
    modelAlias: route.model_alias,
    providerCredentialId: route.provider_credential_id,
  }));
}

export function VirtualKeyEditor({
  providers,
  action,
  mode,
  keyRecord,
  models = [],
}: {
  providers: Provider[];
  action: (state: FieldState, formData: FormData) => Promise<FieldState>;
  mode: "create" | "edit";
  keyRecord?: VirtualKey;
  models?: Model[];
}) {
  const [state, formAction, pending] = useActionState(action, initialState);
  const [routes, setRoutes] = useState<RouteRow[]>(buildInitialRoutes(keyRecord));

  const modelOptions: SelectOption[] = models.map((m) => ({
    value: m.alias,
    label: m.display_name,
    description: m.alias,
  }));

  return (
    <form action={formAction} className="panel-form">
      {keyRecord ? <input type="hidden" name="keyId" value={keyRecord.id} /> : null}

      <div className="panel-form__grid">
        <label className="field">
          <span>Name</span>
          <input
            name="name"
            type="text"
            placeholder="Production key"
            defaultValue={keyRecord?.name ?? ""}
            required
          />
        </label>

        <label className="field">
          <span>Expires at</span>
          <input
            name="expiresAt"
            type="datetime-local"
            defaultValue={keyRecord?.expires_at?.slice(0, 16) ?? ""}
          />
        </label>
      </div>

      <label className="field">
        <span>Tags</span>
        <input
          name="tags"
          type="text"
          defaultValue={keyRecord?.tags.join(", ") ?? ""}
          placeholder="prod, analytics, backend"
        />
      </label>

      {mode === "edit" ? (
        <label className="field field--checkbox">
          <input
            name="enabled"
            type="checkbox"
            defaultChecked={keyRecord?.enabled ?? true}
          />
          <span>Key is enabled for gateway traffic</span>
        </label>
      ) : null}

      {/* Model scoping — only shown in edit mode */}
      {mode === "edit" && (
        <>
          <div className="route-editor">
            <div className="route-editor__header">
              <div>
                <h3>Model scope &amp; routing</h3>
                <p style={{ margin: "0.25rem 0 0", fontSize: "0.8125rem", color: "var(--text-soft)" }}>
                  Each route scopes the key to that model and routes it to a provider. Leave empty for unrestricted access.
                </p>
              </div>
              <button
                type="button"
                className="secondary-button"
                onClick={() =>
                  setRoutes((current) => [...current, { modelAlias: "", providerCredentialId: "" }])
                }
              >
                Add route
              </button>
            </div>

            <div className="route-editor__rows">
              {routes.map((route, index) => (
                <div className="route-editor__row" key={`${route.modelAlias}-${index}`}>
                  <label className="field">
                    <span>Model alias</span>
                    {modelOptions.length > 0 ? (
                      <select name="routeModelAlias" defaultValue={route.modelAlias}>
                        <option value="">Select model</option>
                        {modelOptions.map((opt) => (
                          <option key={opt.value} value={opt.value}>
                            {opt.label}
                          </option>
                        ))}
                      </select>
                    ) : (
                      <input
                        name="routeModelAlias"
                        type="text"
                        defaultValue={route.modelAlias}
                        placeholder="google-genai/gemini-2.5-flash"
                      />
                    )}
                  </label>

                  <label className="field">
                    <span>Provider credential</span>
                    <select
                      name="routeProviderCredentialId"
                      defaultValue={route.providerCredentialId}
                    >
                      <option value="">Select credential</option>
                      {providers.map((provider) => (
                        <option key={provider.id} value={provider.id}>
                          {provider.label} · {provider.provider}
                        </option>
                      ))}
                    </select>
                  </label>

                  <button
                    type="button"
                    className="ghost-button"
                    style={{ alignSelf: "flex-end" }}
                    onClick={() =>
                      setRoutes((current) =>
                        current.length === 1
                          ? [{ modelAlias: "", providerCredentialId: "" }]
                          : current.filter((_, rowIndex) => rowIndex !== index),
                      )
                    }
                  >
                    Remove
                  </button>
                </div>
              ))}
            </div>
          </div>
        </>
      )}

      {state.error ? <p className="form-message form-message--error">{state.error}</p> : null}
      {state.success ? <p className="form-message form-message--success">{state.success}</p> : null}

      {state.rawKey ? (
        <div className="secret-panel">
          <div>
            <h3>Store this key — shown once only</h3>
            <p>
              The backend only returns the raw key once. Copy it now before navigating away.
            </p>
          </div>
          <code>{state.rawKey}</code>
          <CopyButton value={state.rawKey} />
          {state.createdKeyId ? (
            <a
              href={`/virtual-keys/${state.createdKeyId}`}
              className="secondary-button"
              style={{ alignSelf: "flex-start" }}
            >
              Configure model scopes →
            </a>
          ) : null}
        </div>
      ) : null}

      <button type="submit" className="primary-button" disabled={pending}>
        {pending
          ? "Saving…"
          : mode === "create"
            ? "Create virtual key"
            : "Save changes"}
      </button>
    </form>
  );
}
