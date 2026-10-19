import type { Metadata, Viewport } from "next";
import "./globals.css";


const SITE = "https://qor-arqade-tau.vercel.app";
const DESCRIPTION =
  "ARQADE, the gaming platform powered by Demiurge. Play original games, create new worlds, and explore a sovereign terminal.";

// Phones: the page reaches the screen's rounded edges and notch (globals.css keeps content inside the safe area), and
// the browser's own bars take ARQADE's darkness.
export const viewport: Viewport = {
  width: "device-width",
  initialScale: 1,
  viewportFit: "cover",
  themeColor: "#0d0e11",
};

export const metadata: Metadata = {
  metadataBase: new URL(SITE),
  title: "ARQADE — powered by Demiurge",
  description: DESCRIPTION,
  alternates: { canonical: "/" },
  applicationName: "ARQADE",
  publisher: "Demiurge",
  authors: [{ name: "Demiurge", url: "https://qorsync.dev" }],
  openGraph: {
    type: "website",
    siteName: "ARQADE",
    title: "ARQADE — powered by Demiurge",
    description: DESCRIPTION,
    url: "/",
    images: [{ url: "/portal.png", alt: "ARQADE" }],
  },
  twitter: { card: "summary_large_image", title: "ARQADE — powered by Demiurge", description: DESCRIPTION, images: ["/portal.png"] },
  icons: {
    icon: "/favicon.svg",
    shortcut: "/favicon.svg",
  },
};

// Search engines read ARQADE as one of Demiurge's products, and Demiurge's homepage as its publisher (ADR-100).
const STRUCTURED = {
  "@context": "https://schema.org",
  "@type": "WebApplication",
  name: "ARQADE",
  url: `${SITE}/`,
  description: DESCRIPTION,
  applicationCategory: "GameApplication",
  browserRequirements: "A current web browser",
  publisher: { "@type": "Organization", "@id": "https://qorsync.dev/#organization", name: "Demiurge", url: "https://qorsync.dev/" },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className="antialiased">
        <script type="application/ld+json" dangerouslySetInnerHTML={{ __html: JSON.stringify(STRUCTURED) }} />
        {children}
      </body>
    </html>
  );
}

