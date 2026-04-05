"use client";

import Link from "next/link";
import { useActionState } from "react";

import type { FieldState } from "@/lib/types";

const initialState: FieldState = {};

export function AuthForm({
  mode,
  action,
}: {
  mode: "signin" | "signup";
  action: (state: FieldState, formData: FormData) => Promise<FieldState>;
}) {
  const [state, formAction, pending] = useActionState(action, initialState);

  return (
    <div>
      <div className="auth-card__header">
        <h2>{mode === "signup" ? "Create account" : "Welcome back"}</h2>
        <p>
          {mode === "signup"
            ? "Set up your ValyMux control-plane access."
            : "Sign in to your dashboard."}
        </p>
      </div>

      <form action={formAction} className="auth-form">
        {mode === "signup" ? (
          <label className="field">
            <span>Name</span>
            <input name="name" type="text" placeholder="Ada Lovelace" required />
          </label>
        ) : null}

        <label className="field">
          <span>Email</span>
          <input name="email" type="email" placeholder="you@company.com" required />
        </label>

        <label className="field">
          <span>Password</span>
          <input
            name="password"
            type="password"
            placeholder={mode === "signup" ? "Create a strong password" : "Your password"}
            required
          />
        </label>

        {state.error ? <p className="form-message form-message--error">{state.error}</p> : null}
        {state.success ? <p className="form-message form-message--success">{state.success}</p> : null}

        <button className="primary-button primary-button--wide" type="submit" disabled={pending}>
          {pending
            ? "Working…"
            : mode === "signup"
              ? "Create account"
              : "Sign in"}
        </button>

        <p className="auth-form__switch">
          {mode === "signup" ? "Already have an account?" : "Need an account?"}{" "}
          <Link href={mode === "signup" ? "/signin" : "/signup"}>
            {mode === "signup" ? "Sign in" : "Create one"}
          </Link>
        </p>
      </form>
    </div>
  );
}
