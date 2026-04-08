import { Shield, KeyRound, Server } from "lucide-react";

import { BrandMark } from "@/components/ui";
import { FadeIn, SlideIn } from "@/components/motion";

export default function PublicLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <div className="flex min-h-screen">
      <section className="relative hidden overflow-hidden bg-gradient-to-br from-slate-950 via-indigo-950 to-slate-900 lg:flex lg:w-1/2 xl:w-[55%]">
        {/* Subtle grid pattern overlay */}
        <div
          className="pointer-events-none absolute inset-0 opacity-[0.04]"
          style={{
            backgroundImage:
              "linear-gradient(rgba(255,255,255,.1) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,.1) 1px, transparent 1px)",
            backgroundSize: "64px 64px",
          }}
        />

        {/* Radial glow */}
        <div className="pointer-events-none absolute -top-1/4 left-1/2 h-[600px] w-[600px] -translate-x-1/2 rounded-full bg-indigo-500/10 blur-3xl" />

        <div className="relative z-10 flex flex-1 flex-col justify-between p-10 text-white">
          <FadeIn>
            <div className="flex items-center gap-2.5">
              <BrandMark />
              <span className="text-[0.95rem] font-semibold tracking-[-0.03em]">ValyMux</span>
            </div>
          </FadeIn>

          <SlideIn direction="up" delay={0.1}>
            <div className="max-w-md space-y-4">
              <p className="text-[0.64rem] font-semibold uppercase tracking-[0.2em] text-indigo-300/70">
                LLM Gateway
              </p>
              <h1 className="text-[2rem] font-bold leading-[1.15] tracking-[-0.04em]">
                Unified LLM Gateway
              </h1>
              <p className="text-[0.9rem] leading-relaxed text-white/55">
                Secure provider credentials, issue scoped virtual keys, and
                monitor your model catalog from one unified control plane.
              </p>
            </div>
          </SlideIn>

          <FadeIn delay={0.25}>
            <div className="grid max-w-md grid-cols-3 gap-3">
              {[
                {
                  icon: Shield,
                  title: "Encrypted Vault",
                  body: "AES-256-GCM encrypted storage for every credential.",
                },
                {
                  icon: KeyRound,
                  title: "Scoped Keys",
                  body: "Model-scoped keys independent of real secrets.",
                },
                {
                  icon: Server,
                  title: "Provider Isolation",
                  body: "Rotate providers without disrupting consumers.",
                },
              ].map(({ icon: Icon, title, body }) => (
                <div
                  key={title}
                  className="rounded-md border border-white/[0.06] bg-white/[0.04] p-3.5 backdrop-blur-sm transition-colors hover:bg-white/[0.06]"
                >
                  <Icon size={16} className="mb-2.5 text-indigo-300/60" />
                  <h3 className="text-[0.78rem] font-semibold tracking-[-0.02em]">
                    {title}
                  </h3>
                  <p className="mt-1 text-[0.7rem] leading-[1.45] text-white/40">
                    {body}
                  </p>
                </div>
              ))}
            </div>
          </FadeIn>
        </div>
      </section>

      <section className="flex flex-1 items-center justify-center bg-background p-6">
        <div className="w-full max-w-sm">
          {children}
        </div>
      </section>
    </div>
  );
}
