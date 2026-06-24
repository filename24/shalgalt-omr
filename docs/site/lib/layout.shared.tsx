import type { BaseLayoutProps } from 'fumadocs-ui/layouts/shared';
import type { Translations } from 'fumadocs-ui/i18n';

// Minimal Mongolian chrome strings. English uses the built-in defaults.
// Keys are the descriptive identifiers from fumadocs-ui's `Translations` map.
export const mnTranslations: Partial<Translations> = {
  'Search(search dialog)': 'Хайх',
  'Search(search trigger)': 'Хайх',
  'No results found(search dialog)': 'Илэрц олдсонгүй',
  'On this page(table of contents)': 'Энэ хуудсанд',
  'No Headings(table of contents)': 'Гарчиг алга',
  'Table of Contents(inline table of contents)': 'Гарчиг',
  'Last updated on(page footer)': 'Сүүлд шинэчилсэн',
  'Choose a language(language switcher)': 'Хэл сонгох',
  'Next Page(pagination)': 'Дараах',
  'Previous Page(pagination)': 'Өмнөх',
  'Edit on GitHub(edit page)': 'GitHub дээр засах',
  'Page Not Found(404 page)': 'Хуудас олдсонгүй',
  'Back to Home(404 page)': 'Нүүр хуудас руу буцах',
};

// Locale metadata shown in the language switcher.
export const locales = [
  { name: 'Монгол', locale: 'mn' },
  { name: 'English', locale: 'en' },
];

export function baseOptions(locale: string): BaseLayoutProps {
  return {
    i18n: true,
    nav: {
      title: 'shalgalt-omr',
      url: `/${locale}`,
    },
    githubUrl: 'https://github.com/filename24/shalgalt-omr',
  };
}
