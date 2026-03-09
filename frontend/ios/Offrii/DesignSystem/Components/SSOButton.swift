import SwiftUI

// MARK: - SSO Provider

enum SSOProvider {
    case google
    case facebook
    case apple

    var label: String {
        switch self {
        case .google:
            return NSLocalizedString("auth.continueWithGoogle", comment: "")
        case .facebook:
            return NSLocalizedString("auth.continueWithFacebook", comment: "")
        case .apple:
            return NSLocalizedString("auth.continueWithApple", comment: "")
        }
    }
}

// MARK: - SSOButton

struct SSOButton: View {
    let provider: SSOProvider
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: OffriiTheme.spacingSM) {
                providerIcon
                Text(provider.label)
                    .font(OffriiTypography.subheadline)
                    .fontWeight(.medium)
            }
            .foregroundColor(OffriiTheme.text)
            .frame(maxWidth: .infinity)
            .frame(height: 48)
            .background(Color(hex: "#F2F2F7"))
            .cornerRadius(OffriiTheme.cornerRadiusMD)
        }
        .buttonStyle(.plain)
    }

    // MARK: - Provider Icon

    @ViewBuilder
    private var providerIcon: some View {
        switch provider {
        case .google:
            Image("google-logo")
                .resizable()
                .scaledToFit()
                .frame(width: 20, height: 20)
        case .facebook:
            Image("facebook-logo")
                .resizable()
                .scaledToFit()
                .frame(width: 20, height: 20)
        case .apple:
            Image(systemName: "apple.logo")
                .font(.system(size: 18, weight: .medium))
                .foregroundColor(OffriiTheme.text)
        }
    }
}

// MARK: - Preview

#if DEBUG
struct SSOButton_Previews: PreviewProvider {
    static var previews: some View {
        HStack(spacing: OffriiTheme.spacingSM) {
            SSOButton(provider: .google) {}
            SSOButton(provider: .facebook) {}
        }
        .padding(OffriiTheme.spacingLG)
        .background(Color.white)
        .previewLayout(.sizeThatFits)
    }
}
#endif
