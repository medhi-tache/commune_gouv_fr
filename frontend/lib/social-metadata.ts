import type { Metadata } from "next";

/** Informations publiques de partage : aucune donnée de session n'est exposée. */
export function socialMetadata(
  title: string,
  description: string,
  pathname: string,
): Metadata {
  const image = {
    url: "https://qomune.fr/images/qomune-partage-v1.png",
    width: 1200,
    height: 630,
    alt: "Qomune - Votre commune, en lien direct. Prototype indépendant.",
    type: "image/png",
  };
  return {
    title,
    description,
    metadataBase: new URL("https://qomune.fr"),
    openGraph: {
      type: "website",
      siteName: "Qomune",
      locale: "fr_FR",
      url: new URL(pathname, "https://qomune.fr").href,
      title,
      description,
      images: [image],
    },
    twitter: {
      card: "summary_large_image",
      title,
      description,
      images: [{ url: image.url, alt: image.alt }],
    },
  };
}
