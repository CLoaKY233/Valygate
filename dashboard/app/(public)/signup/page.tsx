import { AuthForm } from "@/components/auth-form";
import { signUpAction } from "@/app/actions";

export default function SignUpPage() {
  return <AuthForm mode="signup" action={signUpAction} />;
}
