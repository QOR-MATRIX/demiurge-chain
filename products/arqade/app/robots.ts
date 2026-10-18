import type { MetadataRoute } from "next";

// Crawl the arcade; the API is for the page itself (ADR-100).
export default function robots(): MetadataRoute.Robots {
  return {
    rules: [{ userAgent: "*", allow: "/", disallow: "/api/" }],
    sitemap: "https://qor-arqade-tau.vercel.app/sitemap.xml",
  };
}
