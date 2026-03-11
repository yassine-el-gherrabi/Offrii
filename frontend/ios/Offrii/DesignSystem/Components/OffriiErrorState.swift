import SwiftUI

// MARK: - OffriiErrorState

struct OffriiErrorState: View {
    let message: String
    var retryAction: (() -> Void)?

    var body: some View {
        VStack(spacing: OffriiTheme.spacingLG) {
            Image(systemName: "exclamationmark.triangle.fill")
                .font(.system(size: 48))
                .foregroundColor(OffriiTheme.warning)

            Text(message)
                .font(OffriiTypography.body)
                .foregroundColor(OffriiTheme.textSecondary)
                .multilineTextAlignment(.center)

            if let retryAction {
                OffriiButton(
                    NSLocalizedString("common.retry", comment: ""),
                    variant: .secondary,
                    action: retryAction
                )
                .frame(width: 160)
            }
        }
        .padding(OffriiTheme.spacingXXL)
        .frame(maxWidth: .infinity)
    }
}
