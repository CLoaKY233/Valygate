import { getCurrentUser } from "@/lib/api";

import { AppNav } from "@/components/app-nav";
import { SignOutButton } from "@/components/signout-button";
import { BrandMark } from "@/components/ui";

function initialFor(name: string) {
  return name
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase())
    .join("");
}

export default async function AppLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  const user = await getCurrentUser();

  return (
    <div className="app-shell">
      <aside className="app-nav">
        <div className="app-nav__panel">
          <div className="brand-row">
            <BrandMark />
            <strong>ValyMux</strong>
          </div>

          <AppNav />

          <div className="app-nav__spacer" />

          <div className="app-nav__session">
            <p className="name">{user.name}</p>
            <p className="email">{user.email}</p>
            <div style={{ marginTop: "0.75rem" }}>
              <SignOutButton />
            </div>
          </div>
        </div>
      </aside>

      <main className="app-main">
        <div className="app-main__canvas">
          <div className="app-topbar">
            <div className="app-topbar__user">
              <div className="app-topbar__user-info">
                <strong>{user.name}</strong>
                <span>{user.email}</span>
              </div>
              <div className="avatar">{initialFor(user.name)}</div>
            </div>
          </div>

          <div className="app-content">{children}</div>
        </div>
      </main>
    </div>
  );
}
