import Link from 'next/link';
import { i18n } from '@/lib/i18n';

export function generateStaticParams() {
  return i18n.languages.map((lang) => ({ lang }));
}

const copy = {
  mn: {
    tagline: 'Офлайн ажилладаг OMR засалтын IDE — нутагшсан, нууцлалтай.',
    dev: 'Хөгжүүлэгчийн баримт',
    user: 'Хэрэглэгчийн гарын авлага',
    devDesc: 'Архитектур, бүтээх заавар, HTTP API лавлагаа.',
    userDesc: 'Суулгах, шалгалт засах, үр дүн гаргах.',
  },
  en: {
    tagline: 'A local-first, offline OMR grading IDE.',
    dev: 'Developer Docs',
    user: 'User Manual',
    devDesc: 'Architecture, build guide, and the HTTP API reference.',
    userDesc: 'Install, grade exams, and export results.',
  },
} as const;

export default async function Home(props: {
  params: Promise<{ lang: string }>;
}) {
  const { lang } = await props.params;
  const t = lang === 'en' ? copy.en : copy.mn;
  const prefix = `/${lang}`;

  return (
    <main className="flex flex-1 flex-col items-center justify-center gap-8 px-6 py-24 text-center">
      <div className="space-y-3">
        <h1 className="text-4xl font-bold tracking-tight">shalgalt-omr</h1>
        <p className="text-fd-muted-foreground max-w-xl">{t.tagline}</p>
      </div>
      <div className="grid w-full max-w-2xl gap-4 sm:grid-cols-2">
        <Link
          href={`${prefix}/docs/dev`}
          className="rounded-lg border border-fd-border bg-fd-card p-6 text-left transition-colors hover:bg-fd-accent"
        >
          <h2 className="mb-1 text-lg font-semibold">{t.dev}</h2>
          <p className="text-sm text-fd-muted-foreground">{t.devDesc}</p>
        </Link>
        <Link
          href={`${prefix}/docs/user`}
          className="rounded-lg border border-fd-border bg-fd-card p-6 text-left transition-colors hover:bg-fd-accent"
        >
          <h2 className="mb-1 text-lg font-semibold">{t.user}</h2>
          <p className="text-sm text-fd-muted-foreground">{t.userDesc}</p>
        </Link>
      </div>
    </main>
  );
}
