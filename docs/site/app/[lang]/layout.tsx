import { RootProvider } from 'fumadocs-ui/provider/next';
import { Inter } from 'next/font/google';
import type { ReactNode } from 'react';
import { i18n } from '@/lib/i18n';
import { mnTranslations, locales } from '@/lib/layout.shared';
import type { Translations } from 'fumadocs-ui/i18n';

const inter = Inter({ subsets: ['latin'] });

export function generateStaticParams() {
  return i18n.languages.map((lang) => ({ lang }));
}

export default async function LangLayout({
  params,
  children,
}: {
  params: Promise<{ lang: string }>;
  children: ReactNode;
}) {
  const { lang } = await params;
  const translations: Partial<Translations> | undefined =
    lang === 'mn' ? mnTranslations : undefined;

  return (
    <html lang={lang} className={inter.className} suppressHydrationWarning>
      <body className="flex flex-col min-h-screen">
        <RootProvider
          i18n={{
            locale: lang,
            locales,
            translations,
          }}
          search={{
            // Static export: the search index is fetched once and queried
            // in-browser via the flexsearch static client.
            options: {
              type: 'static',
            },
          }}
        >
          {children}
        </RootProvider>
      </body>
    </html>
  );
}
