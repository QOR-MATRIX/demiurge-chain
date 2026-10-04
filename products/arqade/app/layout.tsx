import type { Metadata } from "next";
import "./globals.css";


export const metadata: Metadata = {
  title: "DEMIURGE — The Sovereign Arcade",
  description: "Enter the Demiurge universe. Play original games, create new worlds, and explore a sovereign terminal.",
  icons: {
    icon: "/favicon.svg",
    shortcut: "/favicon.svg",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className="antialiased">{children}</body>
    </html>
  );
}

