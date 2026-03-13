import SwiftUI

// MARK: - SSO Provider

enum SSOProvider {
    case google
    case apple

    var label: String {
        switch self {
        case .google:
            return NSLocalizedString("auth.continueWithGoogle", comment: "")
        case .apple:
            return NSLocalizedString("auth.continueWithApple", comment: "")
        }
    }
}

// MARK: - SSOButton

struct SSOButton: View {
    let provider: SSOProvider
    var isLoading: Bool = false
    let action: () -> Void

    @State private var isPressed = false

    private var backgroundColor: Color {
        switch provider {
        case .apple: return .black
        case .google: return .white
        }
    }

    private var foregroundColor: Color {
        switch provider {
        case .apple: return .white
        case .google: return .black
        }
    }

    var body: some View {
        Button(action: action) {
            ZStack {
                if isLoading {
                    ProgressView()
                        .tint(foregroundColor)
                } else {
                    // Centered label
                    Text(provider.label)
                        .font(OffriiTypography.subheadline)
                        .fontWeight(.medium)

                    // Left-aligned icon
                    HStack {
                        providerIcon
                        Spacer()
                    }
                    .padding(.leading, OffriiTheme.spacingBase)
                }
            }
            .foregroundColor(foregroundColor)
            .frame(maxWidth: .infinity)
            .frame(height: 48)
            .background(backgroundColor)
            .cornerRadius(OffriiTheme.cornerRadiusMD)
            .overlay(
                RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusMD)
                    .strokeBorder(provider == .google ? OffriiTheme.border : .clear, lineWidth: 1)
            )
        }
        .buttonStyle(.plain)
        .scaleEffect(isPressed ? 0.97 : 1.0)
        .animation(OffriiAnimation.micro, value: isPressed)
        .pressEvents {
            isPressed = true
        } onRelease: {
            isPressed = false
        }
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
        case .apple:
            Image(systemName: "apple.logo")
                .font(.system(size: 18, weight: .medium))
                .foregroundColor(.white)
        }
    }
}
