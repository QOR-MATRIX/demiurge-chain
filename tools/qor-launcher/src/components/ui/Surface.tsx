/**
 * The panel surface.
 *
 * Every panel in the launcher is one of these: a translucent fill, a hairline
 * border and a 2px radius. Structure comes from those and from spacing. Panels
 * do not glow or follow the pointer: a choice, since ADR-080 allows effects.
 *
 * `interactive` panels answer hover with a fill and a border, the same way
 * buttons do. `cut` adds the corner registration marks that identify a panel
 * that is selected or can be acted on.
 */

import type { CSSProperties, ReactNode } from 'react';

type Element = 'div' | 'button' | 'article' | 'section' | 'li';

interface Props {
  children: ReactNode;
  as?: Element;
  className?: string;
  style?: CSSProperties;
  /** Corner registration marks. */
  cut?: boolean;
  /** Answer hover with a fill and border. Implies a pointer cursor. */
  interactive?: boolean;
  onClick?: () => void;
  title?: string;
  disabled?: boolean;
  'aria-label'?: string;
  'aria-current'?: 'page' | undefined;
}

export function Surface({
  children,
  as = 'div',
  className = '',
  style,
  cut = false,
  interactive = false,
  onClick,
  title,
  disabled,
  ...aria
}: Props) {
  const classes = [
    'surface',
    cut ? 'cut' : '',
    interactive ? 'surface-interactive' : '',
    className,
  ]
    .filter(Boolean)
    .join(' ');

  const Tag = as as 'div';

  return (
    <Tag
      className={classes}
      style={style}
      onClick={disabled ? undefined : onClick}
      title={title}
      {...(as === 'button' ? { type: 'button' as const, disabled } : {})}
      {...aria}
    >
      {children}
    </Tag>
  );
}
