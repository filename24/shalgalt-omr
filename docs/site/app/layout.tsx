import './global.css';

// The real <html>/<body> live in app/[lang]/layout.tsx so the lang attribute
// and providers are locale-aware. This root only passes children through.
export default function RootLayout({ children }: { children: React.ReactNode }) {
  return children;
}
