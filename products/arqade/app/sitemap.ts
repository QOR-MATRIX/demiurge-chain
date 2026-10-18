import type { MetadataRoute } from "next";

// ARQADE is one page; the homepage of the project it belongs to is https://qorsync.dev (ADR-100).
export default function sitemap(): MetadataRoute.Sitemap {
  return [{ url: "https://qor-arqade-tau.vercel.app/", changeFrequency: "weekly", priority: 1 }];
}
