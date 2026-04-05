import { AuthForm } from "@/components/auth-form";
import { signInAction } from "@/app/actions";

export default function SignInPage() {
  return <AuthForm mode="signin" action={signInAction} />;
}
