import type { Metadata } from "next";

import "./globals.css";

export const metadata: Metadata = {
  title: "ValyMux Dashboard",
  description: "Control plane for your ValyMux LLM gateway.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
