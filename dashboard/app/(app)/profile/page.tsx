import { revalidatePath } from "next/cache";

import { getCurrentUser } from "@/lib/api";
import { SectionHeader, StatusPill, Surface } from "@/components/ui";

async function updateProfileAction(formData: FormData) {
  "use server";

  const name = String(formData.get("name") ?? "").trim();
  const { getApiBaseUrl } = await import("@/lib/config");
  const { getSessionToken } = await import("@/lib/session");
  const token = await getSessionToken();

  if (!token) return;

  await fetch(`${getApiBaseUrl()}/me`, {
    method: "PATCH",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify({ name }),
    cache: "no-store",
  });

  revalidatePath("/profile");
}

export default async function ProfilePage() {
  const user = await getCurrentUser();

  return (
    <>
      <SectionHeader
        title="Profile"
        description="Manage your account identity and settings."
        actions={
          <StatusPill
            label={user.enabled ? "active" : "disabled"}
            tone={user.enabled ? "success" : "neutral"}
          />
        }
      />

      <div className="panel-grid">
        {/* Account info */}
        <Surface>
          <div className="surface__title-row">
            <div>
              <p className="eyebrow">Account</p>
              <h2>Identity</h2>
            </div>
          </div>
          <dl className="meta-list">
            <div>
              <dt>User ID</dt>
              <dd className="mono">{user.id}</dd>
            </div>
            <div>
              <dt>Email</dt>
              <dd>{user.email}</dd>
            </div>
            <div>
              <dt>Display name</dt>
              <dd>{user.name}</dd>
            </div>
            <div>
              <dt>Account status</dt>
              <dd>
                <StatusPill
                  label={user.enabled ? "active" : "disabled"}
                  tone={user.enabled ? "success" : "neutral"}
                />
              </dd>
            </div>
          </dl>
        </Surface>

        {/* Update name */}
        <Surface>
          <div className="surface__title-row">
            <div>
              <p className="eyebrow">Edit</p>
              <h2>Display name</h2>
            </div>
          </div>
          <form className="panel-form" action={updateProfileAction}>
            <label className="field">
              <span>Name</span>
              <input name="name" defaultValue={user.name} required />
            </label>
            <button type="submit" className="primary-button">
              Save changes
            </button>
          </form>
        </Surface>
      </div>
    </>
  );
}
