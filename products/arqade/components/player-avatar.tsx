"use client";
// A player's QOR ID avatar (ADR-079): the picture QOR ID keeps, its still frame when the player asks for less motion,
// and the first letter of the name when there is no picture or it cannot be loaded. The level bubble sits on it.
import Image from "next/image";
import { useEffect, useState } from "react";

export type PlayerAvatarProps = {
  /** The name: the letter when there is no picture, and what a screen reader says. */
  username: string;
  /** The picture's full address, as this site's `/api/auth/me` gives it, or null for none. */
  avatarUrl?: string | null;
  /** Width and height in pixels. */
  size?: number;
  /** The level (ADR-078), shown in the bubble when given. */
  level?: number | null;
};

/** Whether the player asked for less motion. Follows a change while the page is open. */
function useLessMotion(): boolean {
  const [less, setLess] = useState(false);
  useEffect(() => {
    const query = window.matchMedia("(prefers-reduced-motion: reduce)");
    const follow = () => setLess(query.matches);
    follow();
    query.addEventListener("change", follow);
    return () => query.removeEventListener("change", follow);
  }, []);
  return less;
}

export function PlayerAvatar({ username, avatarUrl, size = 34, level }: PlayerAvatarProps) {
  const [failed, setFailed] = useState<string | null>(null);
  const lessMotion = useLessMotion();
  // QOR ID serves an animated picture's first frame at `/still` (ADR-079).
  const src = avatarUrl ? (lessMotion ? `${avatarUrl}/still` : avatarUrl) : null;
  const letter = (username.trim()[0] ?? "?").toUpperCase();

  return (
    <span className="avatar player-avatar-wrap" style={{ width: size, height: size }}>
      {src && failed !== src ? (
        // Served as QOR ID sends it: re-encoded there already (ADR-079), and an animated picture must stay animated.
        <Image src={src} alt="" width={size} height={size} unoptimized className="player-avatar-img" onError={() => setFailed(src)} />
      ) : (
        <span className="player-avatar-letter" aria-hidden="true" style={{ fontSize: Math.round(size * 0.44) }}>
          {letter}
        </span>
      )}
      {level != null && (
        <span className="level-bubble" aria-label={`Level ${level}`}>
          {level}
        </span>
      )}
    </span>
  );
}
