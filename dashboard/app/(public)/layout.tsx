import { BrandMark } from "@/components/ui";

export default function PublicLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <div className="public-shell">
      <section className="public-shell__hero">
        <div className="public-shell__hero-card">
          <div className="public-shell__hero-top stack">
            <div className="brand-row">
              <BrandMark />
              <strong>ValyMux</strong>
            </div>
            <div style={{ marginTop: "2.5rem" }}>
              <p className="eyebrow">LLM Gateway</p>
              <h1>Operate your models with precision.</h1>
              <p style={{ marginTop: "1rem", lineHeight: 1.7 }}>
                Secure provider credentials, issue scoped virtual keys, and monitor your model
                catalog from one unified control plane.
              </p>
            </div>
          </div>

          <div className="public-shell__hero-bottom" style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "1rem" }}>
            <div style={{
              padding: "1.25rem",
              borderRadius: "var(--radius-lg)",
              background: "rgba(255,255,255,0.5)",
              backdropFilter: "blur(8px)",
            }}>
              <p className="eyebrow">Encrypted vault</p>
              <h3 style={{ margin: "0.375rem 0 0.5rem", fontSize: "0.9375rem", fontWeight: 700 }}>
                Provider keys at rest
              </h3>
              <p style={{ margin: 0, fontSize: "0.8125rem", color: "var(--text-soft)" }}>
                AES-256-GCM encrypted storage for all your API credentials.
              </p>
            </div>
            <div style={{
              padding: "1.25rem",
              borderRadius: "var(--radius-lg)",
              background: "rgba(255,255,255,0.5)",
              backdropFilter: "blur(8px)",
            }}>
              <p className="eyebrow">Scoped access</p>
              <h3 style={{ margin: "0.375rem 0 0.5rem", fontSize: "0.9375rem", fontWeight: 700 }}>
                Virtual key isolation
              </h3>
              <p style={{ margin: 0, fontSize: "0.8125rem", color: "var(--text-soft)" }}>
                Issue model-scoped keys independently of your real provider secrets.
              </p>
            </div>
          </div>
        </div>
      </section>

      <section className="public-shell__auth">
        <div className="auth-card">{children}</div>
      </section>
    </div>
  );
}
