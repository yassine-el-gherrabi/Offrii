import SwiftUI

// MARK: - OffriiCard

struct OffriiCard<Content: View>: View {
    let content: Content

    init(@ViewBuilder content: () -> Content) {
        self.content = content()
    }

    var body: some View {
        content
            .padding(OffriiTheme.spacingMD)
            .background(OffriiTheme.card)
            .cornerRadius(OffriiTheme.cornerRadiusLG)
            .shadow(
                color: OffriiTheme.cardShadowColor,
                radius: OffriiTheme.cardShadowRadius,
                x: 0,
                y: OffriiTheme.cardShadowY
            )
    }
}

// MARK: - Card Modifier (alternative API)

struct OffriiCardModifier: ViewModifier {
    func body(content: Content) -> some View {
        content
            .padding(OffriiTheme.spacingMD)
            .background(OffriiTheme.card)
            .cornerRadius(OffriiTheme.cornerRadiusLG)
            .shadow(
                color: OffriiTheme.cardShadowColor,
                radius: OffriiTheme.cardShadowRadius,
                x: 0,
                y: OffriiTheme.cardShadowY
            )
    }
}

extension View {
    func offriiCard() -> some View {
        modifier(OffriiCardModifier())
    }
}

// MARK: - Preview

#if DEBUG
struct OffriiCard_Previews: PreviewProvider {
    static var previews: some View {
        VStack(spacing: OffriiTheme.spacingMD) {
            OffriiCard {
                VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
                    Text("Titre de la carte")
                        .font(OffriiTypography.headline)
                        .foregroundColor(OffriiTheme.text)
                    Text("Description de la carte avec du contenu.")
                        .font(OffriiTypography.body)
                        .foregroundColor(OffriiTheme.textSecondary)
                }
            }

            Text("Utilisation via modifier")
                .font(OffriiTypography.body)
                .foregroundColor(OffriiTheme.text)
                .frame(maxWidth: .infinity, alignment: .leading)
                .offriiCard()
        }
        .padding(OffriiTheme.spacingLG)
        .background(OffriiTheme.cardSurface)
        .previewLayout(.sizeThatFits)
    }
}
#endif
