import { ReactNode } from "react";
import { Link } from "react-router-dom";

interface Crumb {
  label: string;
  to?: string;
}

interface Props {
  title: ReactNode;
  subtitle?: ReactNode;
  crumbs?: Crumb[];
  actions?: ReactNode;
  meta?: ReactNode;
}

export function PageHeader({ title, subtitle, crumbs, actions, meta }: Props) {
  return (
    <header className="px-6 py-5 border-b border-border1 bg-bg0/80 backdrop-blur-md sticky top-0 z-10">
      {crumbs && crumbs.length > 0 && (
        <nav className="text-xs text-fg2 mb-1.5 flex items-center gap-1.5">
          {crumbs.map((c, i) => (
            <span key={i} className="flex items-center gap-1.5">
              {c.to ? (
                <Link to={c.to} className="hover:text-fg0 transition-colors">
                  {c.label}
                </Link>
              ) : (
                <span>{c.label}</span>
              )}
              {i < crumbs.length - 1 && <span className="text-fg2/60">/</span>}
            </span>
          ))}
        </nav>
      )}
      <div className="flex items-baseline justify-between gap-4 flex-wrap">
        <div className="min-w-0">
          <h1 className="text-fg0 text-xl font-medium tracking-tight truncate">{title}</h1>
          {subtitle && <p className="text-fg1 text-sm mt-0.5">{subtitle}</p>}
        </div>
        {actions && <div className="flex items-center gap-2 flex-wrap">{actions}</div>}
      </div>
      {meta && <div className="mt-3 flex items-center gap-4 flex-wrap text-xs text-fg2">{meta}</div>}
    </header>
  );
}
