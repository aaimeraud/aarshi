use crate::config::is_rtl;

pub const MESSAGES_MARKER: &str = "aarshi:messages:end";
pub const PAGES_MARKER: &str = "aarshi:pages:end";
pub const IMPORTS_MARKER: &str = "aarshi:imports:end";
pub const SECTIONS_MARKER: &str = "aarshi:sections:end";

/// Replaces each `__KEY__` placeholder in a template.
fn fill(template: &str, values: &[(&str, &str)]) -> String {
    values
        .iter()
        .fold(template.to_string(), |text, (key, value)| {
            text.replace(&format!("__{key}__"), value)
        })
}

pub fn site(url: &str) -> String {
    fill("export const SITE_URL = \"__URL__\";\n", &[("URL", url)])
}

pub fn i18n_config(languages: &[String]) -> String {
    let list = quoted_list(languages);
    let directions = languages
        .iter()
        .map(|l| format!("  {l}: \"{}\",", if is_rtl(l) { "rtl" } else { "ltr" }))
        .collect::<Vec<_>>()
        .join("\n");
    fill(
        r#"export const languages = [__LIST__] as const;

export type Locale = (typeof languages)[number];

export const defaultLocale: Locale = "__DEFAULT__";

export const direction: Record<Locale, "ltr" | "rtl"> = {
__DIRECTIONS__
};

export const isLocale = (value: string): value is Locale => {
  return (languages as readonly string[]).includes(value);
};

/** Public URL of a page within a locale, e.g. pagePath("fr", "/about") === "/fr/about". */
export const pagePath = (locale: Locale, path: string): string => {
  return path === "/" ? `/${locale}` : `/${locale}${path}`;
};
"#,
        &[
            ("LIST", &list),
            ("DEFAULT", &languages[0]),
            ("DIRECTIONS", &directions),
        ],
    )
}

pub fn i18n_index(languages: &[String]) -> String {
    let default = &languages[0];
    let imports = languages
        .iter()
        .map(|l| format!("import {{ messages as {l} }} from \"./dictionaries/{l}\";"))
        .collect::<Vec<_>>()
        .join("\n");
    let map = languages.join(", ");
    fill(
        r#"import type { Metadata } from "next";
import { notFound } from "next/navigation";

import { SITE_URL } from "@/lib/site";

import { isLocale, languages, pagePath, type Locale } from "./config";
__IMPORTS__

export * from "./config";

export type MessageKey = keyof typeof __DEFAULT__;

const messages = { __MAP__ } satisfies Record<Locale, Record<MessageKey, string>>;

export const t = (locale: Locale, key: MessageKey): string => {
  return messages[locale][key];
};

/** Reads the `[locale]` route param. Unknown languages get a 404. */
export const resolveLocale = async (params: Promise<{ locale: string }>): Promise<Locale> => {
  const { locale } = await params;
  if (!isLocale(locale)) {
    notFound();
  }
  return locale;
};

export const pageMetadata = (locale: Locale, path: string): Metadata => {
  return {
    alternates: {
      canonical: `${SITE_URL}${pagePath(locale, path)}`,
      languages: Object.fromEntries(
        languages.map((l) => [l, `${SITE_URL}${pagePath(l, path)}`]),
      ),
    },
  };
};
"#,
        &[("IMPORTS", &imports), ("DEFAULT", default), ("MAP", &map)],
    )
}

pub fn dictionary(language: &str, default: &str) -> String {
    if language == default {
        return format!("export const messages = {{\n  // {MESSAGES_MARKER}\n}};\n");
    }
    fill(
        r#"import type { messages as base } from "./__DEFAULT__";

export const messages: Record<keyof typeof base, string> = {
  // __MARKER__
};
"#,
        &[("DEFAULT", default), ("MARKER", MESSAGES_MARKER)],
    )
}

pub fn sitemap() -> String {
    fill(
        r#"import type { MetadataRoute } from "next";

import { languages, pagePath } from "@/lib/i18n/config";
import { SITE_URL } from "@/lib/site";

const pages = [
  "/",
  // __MARKER__
];

const sitemap = (): MetadataRoute.Sitemap => {
  return languages.flatMap((locale) =>
    pages.map((path) => ({
      url: `${SITE_URL}${pagePath(locale, path)}`,
      lastModified: new Date(),
    })),
  );
};

export default sitemap;
"#,
        &[("MARKER", PAGES_MARKER)],
    )
}

pub fn robots() -> String {
    r#"import type { MetadataRoute } from "next";

import { SITE_URL } from "@/lib/site";

const robots = (): MetadataRoute.Robots => {
  return {
    rules: { userAgent: "*", allow: "/" },
    sitemap: `${SITE_URL}/sitemap.xml`,
  };
};

export default robots;
"#
    .to_string()
}

/// One root layout for every language: `lang` and `dir` follow the `[locale]` segment,
/// so switching language is a client-side navigation.
pub fn layout() -> String {
    r#"import type { ReactNode } from "react";

import "@/app/globals.css";
import LanguageSwitcher from "@/components/language-switcher";
import { resolveLocale } from "@/lib/i18n";
import { direction, languages } from "@/lib/i18n/config";

export const generateStaticParams = () => {
  return languages.map((locale) => ({ locale }));
};

const RootLayout = async ({
  children,
  params,
}: {
  children: ReactNode;
  params: Promise<{ locale: string }>;
}) => {
  const locale = await resolveLocale(params);
  return (
    <html lang={locale} dir={direction[locale]}>
      <body>
        <header className="flex justify-end p-4">
          <LanguageSwitcher locale={locale} />
        </header>
        {children}
      </body>
    </html>
  );
};

export default RootLayout;
"#
    .to_string()
}

/// Language picker. Client-side links, so switching does not reload the page.
pub fn language_switcher() -> String {
    r#""use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useState } from "react";

import { languages, type Locale } from "@/lib/i18n/config";

const languageName = (code: string, locale: string): string => {
  return new Intl.DisplayNames([locale], { type: "language" }).of(code) ?? code;
};

const LanguageSwitcher = ({ locale }: { locale: Locale }) => {
  const pathname = usePathname();
  const [open, setOpen] = useState(false);
  // "/fr/about" keeps the page part "about", so the same page opens in the other language.
  const page = pathname.split("/").slice(2).join("/");
  const hrefFor = (code: Locale) => (page ? `/${code}/${page}` : `/${code}`);

  return (
    <div className="relative">
      <button
        type="button"
        aria-expanded={open}
        aria-haspopup="true"
        onClick={() => setOpen((value) => !value)}
        className="rounded-md border px-3 py-1.5 text-sm"
      >
        {languageName(locale, locale)}
      </button>
      {open && (
        <ul className="absolute right-0 mt-2 min-w-40 rounded-md border bg-white p-1 text-sm shadow dark:bg-neutral-900">
          {languages.map((code) => (
            <li key={code}>
              <Link
                href={hrefFor(code)}
                hrefLang={code}
                aria-current={code === locale ? "true" : undefined}
                onClick={() => setOpen(false)}
                className="block rounded px-3 py-1.5 hover:bg-neutral-100 dark:hover:bg-neutral-800"
              >
                {languageName(code, code)}
              </Link>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
};

export default LanguageSwitcher;
"#
    .to_string()
}

pub fn view(page: &str) -> String {
    let component = crate::project::to_pascal_case(page);
    fill(
        r#"import { t, type Locale } from "@/lib/i18n";
// __IMPORTS__

const __COMPONENT__View = ({ locale }: { locale: Locale }) => {
  return (
    <main>
      <h1>{t(locale, "__PAGE__.title")}</h1>
      {/* __SECTIONS__ */}
    </main>
  );
};

export default __COMPONENT__View;
"#,
        &[
            ("IMPORTS", IMPORTS_MARKER),
            ("COMPONENT", &component),
            ("PAGE", page),
            ("SECTIONS", SECTIONS_MARKER),
        ],
    )
}

pub fn route(page: &str, route_path: &str) -> String {
    let component = crate::project::to_pascal_case(page);
    fill(
        r#"import type { Metadata } from "next";

import { pageMetadata, resolveLocale } from "@/lib/i18n";
import __COMPONENT__View from "@/views/__PAGE__/__PAGE__.view";

type Props = { params: Promise<{ locale: string }> };

export const generateMetadata = async ({ params }: Props): Promise<Metadata> => {
  return pageMetadata(await resolveLocale(params), "__PATH__");
};

const Page = async ({ params }: Props) => {
  const locale = await resolveLocale(params);
  return <__COMPONENT__View locale={locale} />;
};

export default Page;
"#,
        &[
            ("COMPONENT", &component),
            ("PAGE", page),
            ("PATH", route_path),
        ],
    )
}

pub fn section(page: &str, section: &str) -> String {
    let component = crate::project::to_pascal_case(section);
    fill(
        r#"import { t, type Locale } from "@/lib/i18n";

export const __COMPONENT__Section = ({ locale }: { locale: Locale }) => {
  return (
    <section>
      <h2>{t(locale, "__PAGE__.__SECTION__.title")}</h2>
    </section>
  );
};
"#,
        &[
            ("COMPONENT", &component),
            ("PAGE", page),
            ("SECTION", section),
        ],
    )
}

fn quoted_list(items: &[String]) -> String {
    items
        .iter()
        .map(|i| format!("\"{i}\""))
        .collect::<Vec<_>>()
        .join(", ")
}
