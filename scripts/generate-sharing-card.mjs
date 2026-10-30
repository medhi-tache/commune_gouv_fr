/** Génère la carte de partage à partir du logo existant, sans service externe. */
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
const require = createRequire(
  new URL("../frontend/package.json", import.meta.url),
);
const sharp = require("sharp");
const logo = await sharp(
  fileURLToPath(
    new URL("../frontend/public/images/logo-qomune.png", import.meta.url),
  ),
)
  .trim()
  .resize({ width: 345 })
  .png()
  .toBuffer();

// Composition vectorielle : la carte reste nette au format social 1200 × 630.
const svg = `<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="1200" height="630" viewBox="0 0 1200 630">
<rect width="1200" height="630" fill="#f7f8fc"/>
<rect width="400" height="12" fill="#002395"/><rect x="400" width="400" height="12" fill="white"/><rect x="800" width="400" height="12" fill="#ed2939"/>
<image x="64" y="54" width="345" height="96" preserveAspectRatio="xMinYMid meet" xlink:href="data:image/png;base64,${logo.toString("base64")}"/>
<rect x="895" y="64" width="241" height="38" rx="19" fill="#eef1fa" stroke="#dce2f1"/>
<text x="1015" y="89" fill="#002395" text-anchor="middle" font-family="Arial, sans-serif" font-size="17">Prototype indépendant</text>
<text x="64" y="268" fill="#18264b" font-family="Georgia, serif" font-size="66">Votre commune,</text>
<text x="64" y="345" fill="#18264b" font-family="Georgia, serif" font-size="66">en lien direct.</text>
<rect x="66" y="379" width="68" height="5" rx="2.5" fill="#ed2939"/>
<text x="64" y="433" fill="#5b6680" font-family="Arial, sans-serif" font-size="25">Les actualités locales et l’espace des élus,</text>
<text x="64" y="469" fill="#5b6680" font-family="Arial, sans-serif" font-size="25">réunis pour faire vivre le lien communal.</text>
<circle cx="965" cy="327" r="155" fill="#e9edf8"/>
<g transform="translate(817 210)">
  <path d="M18 195L153 123L282 195Z" fill="#c6cfe9"/>
  <rect x="46" y="193" width="212" height="87" rx="3" fill="white"/>
  <path d="M86 160L153 111L220 160Z" fill="#002395"/>
  <rect x="105" y="159" width="97" height="120" fill="#fafbff"/>
  <rect x="137" y="230" width="32" height="50" rx="15" fill="#a3afd0"/>
  <g fill="#a3afd0"><rect x="63" y="211" width="19" height="24"/><rect x="226" y="211" width="19" height="24"/><rect x="120" y="184" width="17" height="23"/><rect x="169" y="184" width="17" height="23"/></g>
  <path d="M153 112V56" stroke="#8493bb" stroke-width="4"/>
  <rect x="155" y="56" width="13" height="25" fill="#002395"/><rect x="168" y="56" width="13" height="25" fill="white"/><rect x="181" y="56" width="13" height="25" fill="#ed2939"/>
  <path d="M22 281H283" stroke="#a8b5d5" stroke-width="3" stroke-linecap="round"/>
</g>
<path d="M64 533H1136" stroke="#d9e0ef"/>
<text x="64" y="579" fill="#002395" font-family="Arial, sans-serif" font-size="24" font-weight="bold">qomune.fr</text>
<text x="1136" y="579" fill="#626b82" text-anchor="end" font-family="Arial, sans-serif" font-size="19">Le lien local</text>
</svg>`;
const destination = fileURLToPath(
  new URL("../frontend/public/images/qomune-partage-v1.png", import.meta.url),
);
await sharp(Buffer.from(svg)).png({ compressionLevel: 9 }).toFile(destination);
console.log("Carte de partage générée : 1200 × 630 pixels.");
