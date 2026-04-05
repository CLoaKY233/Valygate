import { signOutAction } from "@/app/actions";

export function SignOutButton() {
  return (
    <form action={signOutAction}>
      <button type="submit" className="ghost-button">
        Sign out
      </button>
    </form>
  );
}
