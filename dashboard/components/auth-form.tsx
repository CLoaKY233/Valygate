"use client";

import Link from "next/link";
import { useActionState } from "react";
import { Loader2, ArrowRight } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { ScaleIn } from "@/components/motion";

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

  const isSignUp = mode === "signup";

  return (
    <ScaleIn>
      <Card className="border-0 shadow-none lg:border lg:border-border/60 lg:shadow-sm">
        <CardHeader className="pb-6 text-center">
          <CardTitle className="text-[1.35rem] font-semibold tracking-[-0.04em]">
            {isSignUp ? "Create account" : "Welcome back"}
          </CardTitle>
          <CardDescription className="text-[0.84rem] text-muted-foreground">
            {isSignUp
              ? "Set up your ValyMux control-plane access."
              : "Sign in to your dashboard."}
          </CardDescription>
        </CardHeader>

        <form action={formAction}>
          <CardContent className="space-y-4 pb-6">
            {isSignUp && (
              <div className="space-y-2">
                <Label htmlFor="name" className="text-sm font-medium">
                  Name
                </Label>
                <Input
                  id="name"
                  name="name"
                  type="text"
                  placeholder="Ada Lovelace"
                  autoComplete="name"
                  required
                  className="rounded-md"
                />
              </div>
            )}

            <div className="space-y-2">
              <Label htmlFor="email" className="text-sm font-medium">
                Email
              </Label>
              <Input
                id="email"
                name="email"
                type="email"
                placeholder="you@company.com"
                autoComplete="email"
                required
                className="rounded-md"
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="password" className="text-sm font-medium">
                Password
              </Label>
              <Input
                id="password"
                name="password"
                type="password"
                placeholder={isSignUp ? "Create a strong password" : "Your password"}
                autoComplete={isSignUp ? "new-password" : "current-password"}
                required
                className="rounded-md"
              />
            </div>

            {state.error && (
              <div className="rounded-md border border-destructive/20 bg-destructive/5 px-3 py-2.5">
                <p className="text-[0.8rem] leading-snug text-destructive">
                  {state.error}
                </p>
              </div>
            )}
            {state.success && (
              <div className="rounded-md border border-emerald-500/20 bg-emerald-500/5 px-3 py-2.5">
                <p className="text-[0.8rem] leading-snug text-emerald-600 dark:text-emerald-400">
                  {state.success}
                </p>
              </div>
            )}
          </CardContent>

          <CardFooter className="flex-col gap-4 pt-0">
            <Button
              type="submit"
              className="w-full rounded-md"
              disabled={pending}
            >
              {pending ? (
                <>
                  <Loader2 className="mr-2 size-4 animate-spin" />
                  Working...
                </>
              ) : (
                <>
                  {isSignUp ? "Create account" : "Sign in"}
                  <ArrowRight className="ml-2 size-4" />
                </>
              )}
            </Button>

            <p className="text-center text-[0.8rem] text-muted-foreground">
              {isSignUp ? "Already have an account?" : "Need an account?"}{" "}
              <Link
                href={isSignUp ? "/signin" : "/signup"}
                className="font-medium text-foreground underline underline-offset-4 transition-colors hover:text-foreground/70"
              >
                {isSignUp ? "Sign in" : "Create one"}
              </Link>
            </p>
          </CardFooter>
        </form>
      </Card>
    </ScaleIn>
  );
}
