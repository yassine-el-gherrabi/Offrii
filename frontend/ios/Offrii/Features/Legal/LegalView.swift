import SwiftUI

struct LegalView: View {
    var showPrivacy: Bool = false

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: OffriiTheme.spacingBase) {
                if showPrivacy {
                    privacyContent
                } else {
                    legalContent
                }
            }
            .padding(OffriiTheme.spacingLG)
        }
        .background(OffriiTheme.surface.ignoresSafeArea())
        .navigationTitle(
            showPrivacy
            ? NSLocalizedString("profile.privacy", comment: "")
            : NSLocalizedString("profile.legal", comment: "")
        )
        .navigationBarTitleDisplayMode(.inline)
    }

    // MARK: - Legal Content

    private var legalContent: some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingBase) {
            sectionTitle("Mentions l\u{00E9}gales")

            sectionBody("""
            Offrii est une application mobile \u{00E9}dit\u{00E9}e \u{00E0} titre personnel.

            H\u{00E9}bergement : Les donn\u{00E9}es sont h\u{00E9}berg\u{00E9}es sur des serveurs s\u{00E9}curis\u{00E9}s situ\u{00E9}s en Union Europ\u{00E9}enne.

            Contact : Pour toute question, vous pouvez nous contacter via l'application.

            Propri\u{00E9}t\u{00E9} intellectuelle : L'ensemble des \u{00E9}l\u{00E9}ments composant l'application Offrii \
            (textes, images, logiciels) sont prot\u{00E9}g\u{00E9}s par le droit de la propri\u{00E9}t\u{00E9} intellectuelle.
            """)
        }
    }

    // MARK: - Privacy Content

    private var privacyContent: some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingBase) {
            sectionTitle("Politique de confidentialit\u{00E9}")

            sectionBody("""
            Derni\u{00E8}re mise \u{00E0} jour : Mars 2026

            Donn\u{00E9}es collect\u{00E9}es :
            \u{2022} Email et nom d'affichage (pour votre compte)
            \u{2022} Liste de souhaits (pour le service)
            \u{2022} Token de notification push (pour les rappels)

            Utilisation des donn\u{00E9}es :
            \u{2022} Fournir le service de listes de souhaits partag\u{00E9}es
            \u{2022} Envoyer des rappels si vous les activez
            \u{2022} Aucune donn\u{00E9}e n'est vendue \u{00E0} des tiers

            Conservation :
            \u{2022} Vos donn\u{00E9}es sont conserv\u{00E9}es tant que votre compte est actif
            \u{2022} Vous pouvez exporter ou supprimer vos donn\u{00E9}es \u{00E0} tout moment

            Droits :
            \u{2022} Acc\u{00E8}s, rectification, suppression de vos donn\u{00E9}es
            \u{2022} Export de vos donn\u{00E9}es au format JSON
            \u{2022} Opposition au traitement
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
