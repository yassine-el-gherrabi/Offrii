import SwiftUI

// MARK: - EntraideWelcomeSheet

struct EntraideWelcomeSheet: View {
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        VStack(spacing: OffriiTheme.spacingLG) {
            Spacer().frame(height: OffriiTheme.spacingSM)

            // Hero icon
            Image(systemName: "hand.raised.fill")
                .font(.system(size: 48))
                .foregroundColor(OffriiTheme.primary)
                .frame(width: 88, height: 88)
                .background(OffriiTheme.primary.opacity(0.1))
                .clipShape(Circle())

            // Title
            Text(NSLocalizedString("entraide.welcome.title", comment: ""))
                .font(.system(size: 24, weight: .bold))
                .foregroundColor(OffriiTheme.text)
                .multilineTextAlignment(.center)

            // Explanation points
            VStack(alignment: .leading, spacing: OffriiTheme.spacingBase) {
                welcomePoint(
                    icon: "megaphone.fill",
                    text: NSLocalizedString("entraide.welcome.point1", comment: "")
                )
                welcomePoint(
                    icon: "heart.fill",
                    text: NSLocalizedString("entraide.welcome.point2", comment: "")
                )
                welcomePoint(
                    icon: "shield.checkered",
                    text: NSLocalizedString("entraide.welcome.point3", comment: "")
                )
            }
            .padding(.horizontal, OffriiTheme.spacingBase)

            Spacer()

            // CTA
            OffriiButton(
                NSLocalizedString("entraide.welcome.cta", comment: "")
            ) {
                dismiss()
            }
            .padding(.horizontal, OffriiTheme.spacingLG)
            .padding(.bottom, OffriiTheme.spacingBase)
        }
        .padding(.top, OffriiTheme.spacingLG)
    }

    private func welcomePoint(icon: String, text: String) -> some View {
        HStack(alignment: .top, spacing: OffriiTheme.spacingSM) {
            Image(systemName: icon)
                .font(.system(size: 16))
                .foregroundColor(OffriiTheme.primary)
                .frame(width: 28, height: 28)
                .background(OffriiTheme.primary.opacity(0.08))
                .clipShape(Circle())

            Text(text)
                .font(.system(size: 15))
                .foregroundColor(OffriiTheme.textSecondary)
                .fixedSize(horizontal: false, vertical: true)
        }
    }
}
