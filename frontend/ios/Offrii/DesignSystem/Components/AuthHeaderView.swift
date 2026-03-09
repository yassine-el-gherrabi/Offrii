import SwiftUI

// MARK: - AuthHeaderView

struct AuthHeaderView: View {
    var body: some View {
        ZStack {
            // Background
            OffriiTheme.primary
                .ignoresSafeArea(edges: .top)

            // Decorative orbs
            decorativeOrbs

            // Centered content
            VStack(spacing: OffriiTheme.spacingSM) {
                // Logo placeholder
                RoundedRectangle(cornerRadius: 16)
                    .fill(Color.white)
                    .frame(width: 60, height: 60)
                    .overlay(
                        Image(systemName: "gift.fill")
                            .font(.system(size: 28))
                            .foregroundColor(OffriiTheme.primary)
                    )

                // App name
                Text("Offrii")
                    .font(OffriiTypography.title)
                    .foregroundColor(.white)
            }
            .padding(.top, OffriiTheme.spacingLG)
            .padding(.bottom, OffriiTheme.spacingXL)
        }
        .frame(minHeight: 200)
    }

    // MARK: - Decorative Orbs

    private var decorativeOrbs: some View {
        GeometryReader { geometry in
            let width = geometry.size.width
            let height = geometry.size.height

            Circle()
                .fill(Color.white.opacity(0.08))
                .frame(width: 120, height: 120)
                .offset(x: width * 0.7, y: -height * 0.15)

            Circle()
                .fill(Color.white.opacity(0.06))
                .frame(width: 80, height: 80)
                .offset(x: -width * 0.08, y: height * 0.55)

            Circle()
                .fill(Color.white.opacity(0.10))
                .frame(width: 50, height: 50)
                .offset(x: width * 0.55, y: height * 0.65)
        }
    }
}

// MARK: - Preview

#if DEBUG
struct AuthHeaderView_Previews: PreviewProvider {
    static var previews: some View {
        VStack(spacing: 0) {
            AuthHeaderView()
            Spacer()
        }
        .background(OffriiTheme.cardSurface)
        .ignoresSafeArea(edges: .top)
        .previewLayout(.sizeThatFits)
    }
}
#endif
