import SwiftUI

struct ComingSoonView: View {
    let icon: String
    let featureName: String

    var body: some View {
        ZStack {
            OffriiTheme.cardSurface.ignoresSafeArea()

            VStack(spacing: 0) {
                HeaderView(title: featureName)

                Spacer()

                VStack(spacing: OffriiTheme.spacingLG) {
                    Image(systemName: icon)
                        .font(.system(size: 64))
                        .foregroundStyle(OffriiTheme.textMuted)

                    Text("comingSoon.title")
                        .font(OffriiTypography.title2)
                        .foregroundStyle(OffriiTheme.text)

                    Text("comingSoon.subtitle")
                        .font(OffriiTypography.body)
                        .foregroundStyle(OffriiTheme.textSecondary)
                        .multilineTextAlignment(.center)
                        .padding(.horizontal, OffriiTheme.spacingXL)
                }

                Spacer()
            }
        }
    }
}

#Preview {
    ComingSoonView(icon: "person.2.fill", featureName: "Cercles")
}
