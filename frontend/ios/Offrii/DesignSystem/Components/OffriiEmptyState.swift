import SwiftUI

// MARK: - OffriiEmptyState

struct OffriiEmptyState: View {
    let icon: String
    let title: String
    let subtitle: String
    var ctaTitle: String?
    var ctaAction: (() -> Void)?

    var body: some View {
        VStack(spacing: OffriiTheme.spacingLG) {
            Image(systemName: icon)
                .font(.system(size: 48))
                .foregroundColor(OffriiTheme.textMuted)

            VStack(spacing: OffriiTheme.spacingSM) {
                Text(title)
                    .font(OffriiTypography.titleSmall)
                    .foregroundColor(OffriiTheme.text)
                    .multilineTextAlignment(.center)

                Text(subtitle)
                    .font(OffriiTypography.body)
                    .foregroundColor(OffriiTheme.textSecondary)
                    .multilineTextAlignment(.center)
            }

            if let ctaTitle, let ctaAction {
                OffriiButton(ctaTitle, variant: .primary, action: ctaAction)
                    .frame(width: 200)
            }
        }
        .padding(OffriiTheme.spacingXXL)
        .frame(maxWidth: .infinity)
    }
}
