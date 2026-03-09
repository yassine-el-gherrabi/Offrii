import SwiftUI

struct LegalView: View {
    var showPrivacy: Bool = false

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: OffriiTheme.spacingMD) {
                if showPrivacy {
                    privacyContent
                } else {
                    legalContent
                }
            }
            .padding(OffriiTheme.spacingLG)
        }
        .background(OffriiTheme.cardSurface.ignoresSafeArea())
        .navigationTitle(
            showPrivacy
            ? NSLocalizedString("profile.privacy", comment: "")
            : NSLocalizedString("profile.legal", comment: "")
        )
        .navigationBarTitleDisplayMode(.inline)
    }

    // MARK: - Legal Content

    private var legalContent: some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingMD) {
            sectionTitle("Mentions légales")

            sectionBody("""
            Offrii est une application mobile éditée à titre personnel.

            Hébergement : Les données sont hébergées sur des serveurs sécurisés situés en Union Européenne.

            Contact : Pour toute question, vous pouvez nous contacter via l'application.

            Propriété intellectuelle : L'ensemble des éléments composant l'application Offrii (textes, images, logiciels) sont protégés par le droit de la propriété intellectuelle.
            """)
        }
    }

    // MARK: - Privacy Content

    private var privacyContent: some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingMD) {
            sectionTitle("Politique de confidentialité")

            sectionBody("""
            Dernière mise à jour : Mars 2026

            Données collectées :
            • Email et nom d'affichage (pour votre compte)
            • Liste de souhaits (pour le service)
            • Token de notification push (pour les rappels)

            Utilisation des données :
            • Fournir le service de listes de souhaits partagées
            • Envoyer des rappels si vous les activez
            • Aucune donnée n'est vendue à des tiers

            Conservation :
            • Vos données sont conservées tant que votre compte est actif
            • Vous pouvez exporter ou supprimer vos données à tout moment

            Droits :
            • Accès, rectification, suppression de vos données
            • Export de vos données au format JSON
            • Opposition au traitement
            """)
        }
    }

    // MARK: - Helpers

    private func sectionTitle(_ title: String) -> some View {
        Text(title)
            .font(OffriiTypography.title)
            .foregroundColor(OffriiTheme.text)
    }

    private func sectionBody(_ text: String) -> some View {
        Text(text)
            .font(OffriiTypography.body)
            .foregroundColor(OffriiTheme.textSecondary)
            .lineSpacing(4)
    }
}
