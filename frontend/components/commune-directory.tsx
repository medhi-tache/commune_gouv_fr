"use client";
import CommunePicker from "./commune-picker";
/** La recherche interroge le référentiel sans charger toutes les communes dans le navigateur. */
export default function CommuneDirectory() {
  return (
    <div className="commune-directory">
      <CommunePicker
        label="Rechercher la fiche d’une commune"
        value={null}
        allowAll={false}
        onChange={(commune) => {
          if (commune)
            window.location.assign(
              `/communes/${encodeURIComponent(commune.id)}`,
            );
        }}
      />
    </div>
  );
}
