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

export function pagePath(locale: Locale, path: string): string {
  if (locale === defaultLocale) {
    return path;
  }
  return path === "/" ? `/${locale}` : `/${locale}${path}`;
}
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

import { SITE_URL } from "@/lib/site";

import { languages, pagePath, type Locale } from "./config";
__IMPORTS__

export * from "./config";

export type MessageKey = keyof typeof __DEFAULT__;

const messages = { __MAP__ } satisfies Record<Locale, Record<MessageKey, string>>;

export function t(locale: Locale, key: MessageKey): string {
  return messages[locale][key];
}

export function pageMetadata(locale: Locale, path: string): Metadata {
  return {
    alternates: {
      canonical: `${SITE_URL}${pagePath(locale, path)}`,
      languages: Object.fromEntries(
        languages.map((l) => [l, `${SITE_URL}${pagePath(l, path)}`]),
      ),
    },
  };
}
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

export default function sitemap(): MetadataRoute.Sitemap {
  return languages.flatMap((locale) =>
    pages.map((path) => ({
      url: `${SITE_URL}${pagePath(locale, path)}`,
      lastModified: new Date(),
    })),
  );
}
"#,
        &[("MARKER", PAGES_MARKER)],
    )
}

pub fn robots() -> String {
    r#"import type { MetadataRoute } from "next";

import { SITE_URL } from "@/lib/site";

export default function robots(): MetadataRoute.Robots {
  return {
    rules: { userAgent: "*", allow: "/" },
    sitemap: `${SITE_URL}/sitemap.xml`,
  };
}
"#
    .to_string()
}

pub fn layout(locale: &str) -> String {
    let dir = if is_rtl(locale) { "rtl" } else { "ltr" };
    fill(
        r#"import type { ReactNode } from "react";

import "@/app/globals.css";

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="__LOCALE__" dir="__DIR__">
      <body>{children}</body>
    </html>
  );
}
"#,
        &[("LOCALE", locale), ("DIR", dir)],
    )
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

pub fn route(locale: &str, page: &str, route_path: &str) -> String {
    let component = crate::project::to_pascal_case(page);
    fill(
        r#"import type { Metadata } from "next";

import { pageMetadata } from "@/lib/i18n";
import __COMPONENT__View from "@/views/__PAGE__/__PAGE__.view";

export const metadata: Metadata = pageMetadata("__LOCALE__", "__PATH__");

export default function Page() {
  return <__COMPONENT__View locale="__LOCALE__" />;
}
"#,
        &[
            ("COMPONENT", &component),
            ("PAGE", page),
            ("LOCALE", locale),
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
