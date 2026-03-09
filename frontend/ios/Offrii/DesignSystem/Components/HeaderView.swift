import SwiftUI

// MARK: - HeaderView

struct HeaderView: View {
    let title: String
    var subtitle: String?

    var body: some View {
        ZStack {
            // Background
            OffriiTheme.primary
                .ignoresSafeArea(edges: .top)

            // Decorative orbs
            decorativeOrbs

            // Content
            VStack(alignment: .leading, spacing: OffriiTheme.spacingXS) {
                Text(title)
                    .font(OffriiTypography.largeTitle)
                    .foregroundColor(.white)

                if let subtitle {
                    Text(subtitle)
                        .font(OffriiTypography.subheadline)
                        .foregroundColor(.white.opacity(0.8))
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.horizontal, OffriiTheme.spacingLG)
            .padding(.bottom, OffriiTheme.spacingLG)
            .padding(.top, OffriiTheme.spacingXL)
        }
        .frame(minHeight: 140)
    }

    // MARK: - Decorative Elements

    private var decorativeOrbs: some View {
        DecorativeSquares(preset: .header)
    }
}

// MARK: - Preview

#if DEBUG
struct HeaderView_Previews: PreviewProvider {
    static var previews: some View {
        VStack(spacing: 0) {
            HeaderView(
                title: "Mes envies",
                subtitle: "Partagez vos souhaits avec vos proches"
            )

            Spacer()
        }
        .background(OffriiTheme.cardSurface)
        .ignoresSafeArea(edges: .top)
        .previewLayout(.sizeThatFits)
    }
}
#endif
