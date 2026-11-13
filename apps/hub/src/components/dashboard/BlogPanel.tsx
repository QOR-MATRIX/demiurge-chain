// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
'use client';

import React from 'react';
import Link from 'next/link';

interface BlogPost {
  id: string;
  title: string;
  excerpt: string;
  author: string;
  authorAvatar?: string;
  thumbnail?: string;
  category: string;
  readTime: number;
  publishedAt: string;
  slug: string;
}

// Blog posts fetched from API or on-chain storage
// Empty by default - populated when blog service is available
const INITIAL_POSTS: BlogPost[] = [];

export function BlogPanel() {
  const [posts, setPosts] = React.useState<BlogPost[]>(INITIAL_POSTS);
  const [loading, setLoading] = React.useState(true);

  React.useEffect(() => {
    async function fetchPosts() {
      try {
        // Fetch from blog API when available
        const response = await fetch('http://127.0.0.1:3000/api/posts?limit=3');
        if (response.ok) {
          const data = await response.json();
          setPosts(data.posts || []);
        }
      } catch {
        // Blog service not available - show empty state
      } finally {
        setLoading(false);
      }
    }
    fetchPosts();
  }, []);

  const getCategoryColor = (category: string) => {
    switch (category.toLowerCase()) {
      case 'tokenomics': return 'bg-neon-cyan/20 text-neon-cyan';
      case 'development': return 'bg-neon-magenta/20 text-neon-magenta';
      case 'identity': return 'bg-neon-green/20 text-neon-green';
      default: return 'bg-gray-700 text-gray-300';
    }
  };

  const formatDate = (dateStr: string) => {
    return new Date(dateStr).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
    });
  };

  return (
    <div className="glass-panel rounded-xl p-6">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-bold text-white">Blog</h3>
        <a 
          href="https://qorsync.dev/blog" 
          target="_blank"
          rel="noopener noreferrer"
          className="text-xs text-neon-cyan hover:underline"
        >
          View All →
        </a>
      </div>

      <div className="space-y-4">
        {loading ? (
          <div className="text-center py-6 text-gray-500">Loading posts...</div>
        ) : posts.length === 0 ? (
          <div className="text-center py-6 text-gray-500">
            <p>No blog posts yet.</p>
            <a 
              href="https://qorsync.dev/blog" 
              target="_blank" 
              rel="noopener noreferrer"
              className="text-neon-cyan hover:underline text-sm mt-2 inline-block"
            >
              Visit the blog →
            </a>
          </div>
        ) : null}
        {posts.map((post) => (
          <Link
            key={post.id}
            href={`/blog/${post.slug}`}
            className="block glass-panel p-4 rounded-lg hover:chroma-glow transition-all group"
          >
            <div className="flex items-start gap-3">
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 mb-2">
                  <span className={`text-xs px-2 py-0.5 rounded-full ${getCategoryColor(post.category)}`}>
                    {post.category}
                  </span>
                  <span className="text-xs text-gray-500">
                    {post.readTime} min read
                  </span>
                </div>
                <h4 className="font-semibold text-white group-hover:text-neon-cyan transition-colors line-clamp-1">
                  {post.title}
                </h4>
                <p className="text-sm text-gray-400 mt-1 line-clamp-2">
                  {post.excerpt}
                </p>
                <div className="flex items-center justify-between mt-2">
                  <span className="text-xs text-gray-500">
                    By {post.author}
                  </span>
                  <span className="text-xs text-gray-500">
                    {formatDate(post.publishedAt)}
                  </span>
                </div>
              </div>
            </div>
          </Link>
        ))}
      </div>
    </div>
  );
}
