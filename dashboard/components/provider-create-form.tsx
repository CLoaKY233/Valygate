"use client";

import { useActionState } from "react";

import type { FieldState } from "@/lib/types";

const initialState: FieldState = {};

export function ProviderCreateForm({
  action,
}: {
  action: (state: FieldState, formData: FormData) => Promise<FieldState>;
}) {
  const [state, formAction, pending] = useActionState(action, initialState);

  return (
    <form action={formAction} className="panel-form">
      <div className="panel-form__grid">
        <label className="field">
          <span>Provider</span>
          <select name="provider" defaultValue="google-genai">
            <option value="google-genai">Google GenAI</option>
          </select>
        </label>

        <label className="field">
          <span>Label</span>
          <input name="label" type="text" placeholder="Production Gemini" required />
        </label>
      </div>

      <label className="field">
        <span>API key</span>
        <input name="apiKey" type="password" placeholder="AIza…" required />
      </label>

      <label className="field">
        <span>Tags</span>
        <input name="tags" type="text" placeholder="prod, primary" />
      </label>

      {state.error ? <p className="form-message form-message--error">{state.error}</p> : null}
      {state.success ? <p className="form-message form-message--success">{state.success}</p> : null}

      <button type="submit" className="primary-button" disabled={pending}>
        {pending ? "Connecting…" : "Add provider"}
      </button>
    </form>
  );
}
