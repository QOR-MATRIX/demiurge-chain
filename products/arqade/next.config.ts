import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  // `next dev` would otherwise write its own AGENTS.md and CLAUDE.md here on every run. The repository's rules for
  // assistants are the root AGENTS.md alone (AGENTS.md section 2), and a second set beside the code would contradict it.
  agentRules: false,
};

export default nextConfig;
